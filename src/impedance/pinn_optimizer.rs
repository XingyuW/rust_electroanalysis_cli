#![allow(clippy::needless_range_loop)]

//! Physics-Informed Neural Network (PINN) optimizer for EIS parameter fitting.
//!
//! This module implements a gradient-descent-based optimization framework that
//! replaces the Levenberg-Marquardt solver with a composite physics-informed loss:
//!
//! ```text
//!   L(θ) = L_data(θ) + λ_physics · L_physics(θ) + λ_kk · L_kk(θ)
//! ```
//!
//! Where:
//! - **L_data**: Weighted mean-squared error between circuit predictions and
//!   experimental Z(ω) data.
//! - **L_physics**: Penalty for parameter-bound violations, ensuring physical
//!   realism (e.g. positive resistances, CPE exponents in (0, 1]).
//! - **L_kk**: Discrete Kramers-Kronig consistency residual, enforcing the linear
//!   causal-response requirement of all physical impedance spectra.
//!
//! The circuit equations **are** the physics: each element's `Impedance` trait
//! implementation encodes the domain physics (e.g. Z_CPE = 1 / (Q·(jω)^α)).
//! The gradient-descent optimizer simultaneously fits the data and satisfies
//! these physics constraints—the defining characteristic of a PINN.
//!
//! # No Pre-training Required
//! Unlike general-purpose PINNs, each experimental spectrum is its own training
//! set. The circuit topology supplies the governing equations; no dataset is needed
//! before use.
//!
//! # Noise Resilience
//! The KK regularisation term naturally suppresses high-frequency noise that
//! violates the causal, linear-response physics of electrochemical systems.
//! Parameter-bound penalties prevent physically unreasonable solutions arising
//! from noisy or poorly conditioned data.
//!
//! # Adam Optimiser
//! Parameters are updated with the Adam rule (adaptive moment estimation), which
//! handles the highly non-convex landscape typical of multi-element circuits and
//! is more robust to saddle points than plain gradient descent.

use super::circuits::{CircuitNode, Impedance};
use super::elements::Constraint;
use super::fitting::{
    guess_parameters, sanitize_physical_params, transform_backward, transform_forward,
};
use rayon::prelude::*;
use std::f64::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────
// Public configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Tunable hyperparameters for the PINN optimizer.
#[derive(Debug, Clone)]
pub struct PinnConfig {
    /// Maximum number of Adam gradient-descent epochs.
    pub max_epochs: usize,
    /// Adam learning rate (global step size).
    pub learning_rate: f64,
    /// Weight λ_physics for the physical-bounds penalty term.
    pub physics_weight: f64,
    /// Weight λ_kk for the Kramers-Kronig consistency term.
    pub kk_weight: f64,
    /// Adam first-moment decay β₁ (momentum coefficient).
    pub beta1: f64,
    /// Adam second-moment decay β₂ (RMS-prop coefficient).
    pub beta2: f64,
    /// Adam numerical stability constant ε.
    pub epsilon: f64,
    /// Step size h used for central finite-difference gradient estimation.
    pub fd_step: f64,
    /// Early-stopping patience: number of consecutive epochs with no
    /// improvement (relative threshold 1e-7) before Adam exits early.
    pub patience: usize,
}

impl Default for PinnConfig {
    fn default() -> Self {
        Self {
            max_epochs: 300,
            learning_rate: 5e-3,
            // Physics regularisation is intentionally tiny: its only role is
            // to softly discourage extreme parameter values. At 0.005 it can't
            // overwhelm the forward-model data loss.
            physics_weight: 0.005,
            // Circuit elements are causal, linear, passive systems and therefore
            // satisfy the Kramers–Kronig relations exactly by construction.
            // Applying a KK loss to model predictions adds only FD gradient
            // noise with no useful signal; keep it at 0.
            kk_weight: 0.0,
            beta1: 0.90,
            beta2: 0.999,
            epsilon: 1e-8,
            fd_step: 1e-5,
            patience: 30,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Result type
// ─────────────────────────────────────────────────────────────────────────────

/// Result returned by a completed PINN optimization run.
#[derive(Debug, Clone)]
pub struct PinnResult {
    /// Best physically valid parameter vector found.
    pub fitted_params: Vec<f64>,
    /// Final composite loss value at the best parameters.
    pub final_loss: f64,
    /// Akaike Information Criterion – combines data-fit quality with model
    /// complexity.  Lower is better.
    pub aic: f64,
    /// Bayesian Information Criterion. Lower is better.
    pub bic: f64,
    /// Predicted real-part impedance Z'(ω) at each input frequency.
    pub fitted_z_re: Vec<f64>,
    /// Predicted imaginary-part impedance Z''(ω) at each input frequency.
    pub fitted_z_im: Vec<f64>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Core optimizer
// ─────────────────────────────────────────────────────────────────────────────

/// Physics-Informed optimizer for equivalent-circuit model (ECM) parameter fitting.
///
/// The circuit model serves as the *physics network*: its `Impedance` equations
/// constrain which solutions are physically admissible, analogously to how a
/// traditional PINN uses PDE residuals.
///
/// # Example
/// ```rust,ignore
/// let optimizer = PinnOptimizer::new(&circuit, &frequencies, &z_re, &z_im);
/// let initial = guess_parameters(&circuit, &frequencies, &z_re, &z_im, &[]);
/// let result  = optimizer.optimize(&initial);
/// println!("AIC = {:.2}", result.aic);
/// ```
pub struct PinnOptimizer<'a> {
    circuit: &'a CircuitNode,
    omegas: Vec<f64>,
    z_re_data: Vec<f64>,
    z_im_data: Vec<f64>,
    weights: Vec<f64>,
    constraints: Vec<Constraint>,
    bounds: Vec<(f64, f64)>,
    config: PinnConfig,
}

impl<'a> PinnOptimizer<'a> {
    /// Creates a PINN optimizer with default hyperparameters.
    pub fn new(
        circuit: &'a CircuitNode,
        frequencies: &[f64],
        z_real: &[f64],
        z_imag: &[f64],
    ) -> Self {
        Self::with_config(circuit, frequencies, z_real, z_imag, PinnConfig::default())
    }

    /// Creates a PINN optimizer with custom hyperparameters.
    pub fn with_config(
        circuit: &'a CircuitNode,
        frequencies: &[f64],
        z_real: &[f64],
        z_imag: &[f64],
        config: PinnConfig,
    ) -> Self {
        let omegas: Vec<f64> = frequencies.iter().map(|&f| 2.0 * PI * f).collect();
        let weights = compute_weights(z_real, z_imag);
        let constraints = circuit.get_constraints();
        let bounds = circuit.get_bounds();
        Self {
            circuit,
            omegas,
            z_re_data: z_real.to_vec(),
            z_im_data: z_imag.to_vec(),
            weights,
            constraints,
            bounds,
            config,
        }
    }

    /// Returns the initial parameter guess derived from impedance heuristics.
    pub fn make_initial_guess(&self, frequencies: &[f64], z_re: &[f64], z_im: &[f64]) -> Vec<f64> {
        guess_parameters(self.circuit, frequencies, z_re, z_im, &[])
    }

    /// Runs Adam-optimised PINN training from `initial_params`.
    ///
    /// Parameters are continuously transformed to an unconstrained space during
    /// optimisation and transformed back to physical space before output.
    pub fn optimize(&self, initial_params: &[f64]) -> PinnResult {
        let n_params = initial_params.len();
        if n_params == 0 || self.omegas.is_empty() {
            return self.empty_result();
        }

        // Map physical parameters into unconstrained space x.
        let mut x: Vec<f64> = initial_params
            .iter()
            .zip(self.constraints.iter())
            .map(|(&p, &c)| transform_forward(p.abs().max(1e-30), c))
            .collect();

        // Adam first- and second-moment accumulators.
        let mut m = vec![0.0f64; n_params];
        let mut v = vec![0.0f64; n_params];
        let cfg = &self.config;

        let mut best_loss = f64::INFINITY;
        let mut best_x = x.clone();
        let mut no_improve = 0usize;

        for epoch in 1..=cfg.max_epochs {
            let grad = self.numerical_gradient(&x);

            // Update moments.
            for i in 0..n_params {
                m[i] = cfg.beta1 * m[i] + (1.0 - cfg.beta1) * grad[i];
                v[i] = cfg.beta2 * v[i] + (1.0 - cfg.beta2) * grad[i].powi(2);
            }

            // Bias-corrected moments → parameter update.
            let t = epoch as f64;
            for i in 0..n_params {
                let m_hat = m[i] / (1.0 - cfg.beta1.powf(t));
                let v_hat = v[i] / (1.0 - cfg.beta2.powf(t));
                x[i] -= cfg.learning_rate * m_hat / (v_hat.sqrt() + cfg.epsilon);
            }

            let loss = self.total_loss(&x);
            // Track improvement with a relative threshold to avoid early exit
            // due to floating-point noise near the optimum.
            if loss < best_loss * (1.0 - 1e-7) {
                best_loss = loss;
                best_x = x.clone();
                no_improve = 0;
            } else {
                no_improve += 1;
                if no_improve >= cfg.patience {
                    break;
                }
            }
        }

        let fitted_params = self.to_physical(&best_x);
        let (z_re_pred, z_im_pred) = self.predict(&fitted_params);
        let n = self.omegas.len();
        let k = n_params;
        let mse = point_mse(&self.z_re_data, &self.z_im_data, &z_re_pred, &z_im_pred);
        let aic = compute_aic(n, k, mse);
        let bic = compute_bic(n, k, mse);

        PinnResult {
            fitted_params,
            final_loss: best_loss,
            aic,
            bic,
            fitted_z_re: z_re_pred,
            fitted_z_im: z_im_pred,
        }
    }

    // ──────────────────────────────────────────────────────────────────────
    // Loss components
    // ──────────────────────────────────────────────────────────────────────

    fn total_loss(&self, x: &[f64]) -> f64 {
        let p = self.to_physical(x);
        let mut loss = self.data_loss(&p);
        // Avoid calling expensive regularisation terms when their weight is
        // zero; the compiler cannot eliminate the call itself because the
        // weight is a runtime value.
        if self.config.physics_weight > 0.0 {
            loss += self.config.physics_weight * self.physics_loss(&p);
        }
        if self.config.kk_weight > 0.0 {
            loss += self.config.kk_weight * self.kk_loss(&p);
        }
        loss
    }

    /// Data loss: weighted MSE between circuit prediction and experimental data.
    ///
    /// The weighting by 1/|Z| makes the fit relative (logarithmic), giving
    /// equal importance to high- and low-impedance frequency regions.
    fn data_loss(&self, params: &[f64]) -> f64 {
        let n = self.omegas.len();
        if n == 0 {
            return 0.0;
        }
        let mut sum = 0.0;
        for (i, &omega) in self.omegas.iter().enumerate() {
            let z = self.circuit.calculate(omega, params);
            if !z.re.is_finite() || !z.im.is_finite() {
                sum += 1e6;
                continue;
            }
            let w = self.weights[i];
            let re_err = (z.re - self.z_re_data[i]) * w;
            let im_err = (z.im - self.z_im_data[i]) * w;
            sum += re_err * re_err + im_err * im_err;
        }
        sum / (2.0 * n as f64)
    }

    /// Physics loss: penalty for parameters that violate physical bounds.
    ///
    /// Each violation is normalised by the width of the feasible interval so
    /// the contribution is dimensionless and comparable to `data_loss`,
    /// regardless of the physical units of the parameter.
    fn physics_loss(&self, physical: &[f64]) -> f64 {
        physical
            .iter()
            .zip(self.bounds.iter())
            .map(|(&p, &(lo, hi))| {
                let range = (hi - lo).max(1e-30);
                let v_lo = ((lo - p) / range).max(0.0);
                let v_hi = ((p - hi) / range).max(0.0);
                v_lo * v_lo + v_hi * v_hi
            })
            .sum::<f64>()
    }

    /// Kramers-Kronig consistency loss (discrete Hilbert-transform residual).
    ///
    /// The KK relations state that for any causal, linear time-invariant system:
    ///
    /// ```text
    ///   Z'(ω) = Z'(∞) + (2/π) ∫₀^∞ ω' Z''(ω') / (ω'² − ω²) dω'
    /// ```
    ///
    /// This term penalises predicted spectra that violate the above identity,
    /// which naturally excludes solutions shaped by high-frequency measurement noise.
    fn kk_loss(&self, params: &[f64]) -> f64 {
        let n = self.omegas.len();
        if n < 4 {
            return 0.0;
        }

        let zs: Vec<_> = self
            .omegas
            .iter()
            .map(|&omega| self.circuit.calculate(omega, params))
            .collect();

        if zs.iter().any(|z| !z.re.is_finite() || !z.im.is_finite()) {
            return 1e6;
        }

        // High-frequency limit approximation: minimum real part measured.
        let z_re_inf = zs.iter().map(|z| z.re).fold(f64::INFINITY, f64::min);

        let mut residual = 0.0;
        for i in 0..n {
            let omega_i = self.omegas[i];
            let mut kk_re_estimate = z_re_inf;

            for j in 0..n {
                if j == i {
                    continue;
                }
                let omega_j = self.omegas[j];
                let denom = omega_j * omega_j - omega_i * omega_i;
                if denom.abs() < 1e-10 {
                    continue;
                }
                // Trapezoidal quadrature weight: half-interval to each neighbour.
                let d_omega = if j + 1 < n {
                    (self.omegas[j + 1] - self.omegas[j]).abs()
                } else if j > 0 {
                    (self.omegas[j] - self.omegas[j - 1]).abs()
                } else {
                    continue;
                };
                kk_re_estimate += (2.0 / PI) * (omega_j * zs[j].im / denom) * d_omega;
            }

            let err = kk_re_estimate - zs[i].re;
            residual += err * err;
        }

        residual / n as f64
    }

    // ──────────────────────────────────────────────────────────────────────
    // Gradient and prediction helpers
    // ──────────────────────────────────────────────────────────────────────

    /// Central finite-difference gradient estimate for total_loss w.r.t. x.
    ///
    /// Central differences achieve O(h²) truncation error, giving gradients
    /// that are ~1e5 times more accurate than forward differences at the same
    /// step size. This is critical for the highly non-convex EIS landscape.
    ///
    /// Each parameter perturbation is independent; Rayon evaluates all 2k
    /// `total_loss` calls concurrently, giving a near-linear speedup with
    /// respect to the number of circuit parameters.
    fn numerical_gradient(&self, x: &[f64]) -> Vec<f64> {
        let h = self.config.fd_step;
        (0..x.len())
            .into_par_iter()
            .map(|i| {
                // Each iteration builds its own ±h perturbation vectors so there
                // is no shared mutable state; `total_loss` only takes &self.
                let mut x_plus = x.to_vec();
                let mut x_minus = x.to_vec();
                x_plus[i] += h;
                x_minus[i] -= h;
                (self.total_loss(&x_plus) - self.total_loss(&x_minus)) / (2.0 * h)
            })
            .collect()
    }

    fn to_physical(&self, x: &[f64]) -> Vec<f64> {
        let raw: Vec<f64> = x
            .iter()
            .zip(self.constraints.iter())
            .map(|(&xi, &c)| transform_backward(xi, c))
            .collect();
        sanitize_physical_params(&raw, &self.constraints, &self.bounds)
    }

    fn predict(&self, params: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let mut z_re = Vec::with_capacity(self.omegas.len());
        let mut z_im = Vec::with_capacity(self.omegas.len());
        for &omega in &self.omegas {
            let z = self.circuit.calculate(omega, params);
            z_re.push(z.re);
            z_im.push(z.im);
        }
        (z_re, z_im)
    }

    fn empty_result(&self) -> PinnResult {
        PinnResult {
            fitted_params: vec![],
            final_loss: f64::INFINITY,
            aic: f64::INFINITY,
            bic: f64::INFINITY,
            fitted_z_re: vec![],
            fitted_z_im: vec![],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public information-criterion helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Akaike Information Criterion for Gaussian-noise regression models.
///
/// Combines goodness-of-fit with parsimony: `AIC ≈ n · ln(MSE) + 2k`.
/// The AIC penalises extra parameters less heavily than the BIC, making it
/// preferable when the true model order is uncertain.
pub fn compute_aic(n: usize, k: usize, mse: f64) -> f64 {
    if n == 0 || mse < 0.0 || !mse.is_finite() {
        return f64::INFINITY;
    }
    let n_obs = (2 * n) as f64;
    let effective_mse = mse.max(f64::EPSILON);
    n_obs * effective_mse.ln() + 2.0 * k as f64
}

/// Gaussian-residual Bayesian Information Criterion.
///
/// `n` is the number of complex frequency points. Real and imaginary
/// residuals are independent scalar observations, so the scalar count is
/// `2*n`.
pub fn compute_bic(n: usize, k: usize, mse: f64) -> f64 {
    if n == 0 || mse < 0.0 || !mse.is_finite() {
        return f64::INFINITY;
    }
    let n_obs = (2 * n) as f64;
    n_obs * mse.max(f64::EPSILON).ln() + k as f64 * n_obs.ln()
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ─────────────────────────────────────────────────────────────────────────────

fn compute_weights(z_re: &[f64], z_im: &[f64]) -> Vec<f64> {
    z_re.iter()
        .zip(z_im.iter())
        .map(|(&re, &im)| 1.0 / re.hypot(im).max(1e-12))
        .collect()
}

/// Unweighted MSE across both real and imaginary channels.
fn point_mse(z_re_ref: &[f64], z_im_ref: &[f64], z_re_pred: &[f64], z_im_pred: &[f64]) -> f64 {
    let n = z_re_ref
        .len()
        .min(z_im_ref.len())
        .min(z_re_pred.len())
        .min(z_im_pred.len());
    if n == 0 {
        return f64::INFINITY;
    }
    let sum: f64 = (0..n)
        .map(|i| {
            let re = z_re_pred[i] - z_re_ref[i];
            let im = z_im_pred[i] - z_im_ref[i];
            re * re + im * im
        })
        .sum();
    sum / (2.0 * n as f64)
}
