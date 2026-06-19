#![allow(clippy::too_many_arguments)]

use super::circuits::{CircuitNode, Impedance};
use super::elements::{Constraint, ElementType};
use levenberg_marquardt::LeastSquaresProblem;
use nalgebra::{DMatrix, DVector, Dyn, Owned};
use num_complex::Complex64;
use std::cmp::Ordering;
use std::f64::consts::PI;

struct GuessState {
    /// Estimated solution resistance.
    rs: f64,
    /// Estimated charge-transfer resistance.
    r_ct: f64,
    /// Estimated CPE Q coefficient.
    q_cpe: f64,
    /// Characteristic time constant estimate.
    tau_char: f64,
    /// Estimated generalized-Warburg sigma.
    sigma_gw: f64,
    /// Estimated generalized-Warburg alpha.
    alpha_gw: f64,
    /// Running counter used to assign first R as Rs then Rct-like values.
    r_count: usize,
}

/// Guesses initial parameters for the circuit based on impedance data.
pub fn guess_parameters(
    node: &CircuitNode,
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Vec<f64> {
    let total_params = node.count_total_params();
    let mut params = vec![0.0; total_params];
    let n = frequencies.len();

    if n == 0 {
        return params;
    }

    let (idx_hf, _) = frequencies
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap();
    let (_idx_lf, _) = frequencies
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap();

    // Estimate Rs from the left intercept using the highest-frequency points.
    let mut hf_indices: Vec<usize> = (0..n).collect();
    hf_indices.sort_by(|&lhs, &rhs| {
        frequencies[rhs]
            .partial_cmp(&frequencies[lhs])
            .unwrap_or(Ordering::Equal)
    });
    let hf_window = hf_indices.len().min(5);
    let rs_guess = hf_indices[..hf_window]
        .iter()
        .map(|&idx| z_real[idx])
        .fold(f64::INFINITY, f64::min)
        .abs();

    let (idx_im_peak, _) = z_imag
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap_or((idx_hf, &z_imag[idx_hf]));

    let idx_phase_peak = if phase_deg.len() == n {
        phase_deg
            .iter()
            .enumerate()
            .min_by(|lhs, rhs| lhs.1.partial_cmp(rhs.1).unwrap_or(Ordering::Equal))
            .map(|(idx, _)| idx)
            .unwrap_or(idx_im_peak)
    } else {
        idx_im_peak
    };

    let boundary_margin = (n / 10).max(1);
    let characteristic_idx = if idx_im_peak <= boundary_margin || idx_im_peak + boundary_margin >= n
    {
        idx_phase_peak
    } else {
        idx_im_peak
    };

    let arc_real = (z_real[characteristic_idx] - rs_guess).abs();
    let arc_span = z_real.iter().copied().fold(f64::NEG_INFINITY, f64::max) - rs_guess;
    let rct_guess = if idx_im_peak > boundary_margin && idx_im_peak + boundary_margin < n {
        2.0 * arc_real
    } else {
        arc_real.max(0.15 * arc_span)
    };
    let final_rct_guess = rct_guess.max(1e-6);

    let peak_imag = z_imag[idx_im_peak].abs();
    let mut alpha_guess = if idx_im_peak > boundary_margin && idx_im_peak + boundary_margin < n {
        let ratio = (2.0 * peak_imag / final_rct_guess).max(1e-9);
        (4.0 / PI) * ratio.atan()
    } else {
        0.85
    };
    alpha_guess = alpha_guess.clamp(0.45, 0.98);

    let w_char = (2.0 * PI * frequencies[characteristic_idx].abs()).max(1e-9);
    let q_guess =
        (1.0_f64 / (final_rct_guess * w_char.powf(alpha_guess))).clamp(1e-15_f64, 1e3_f64);

    let tail_idx = frequencies
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .map(|(idx, _)| idx)
        .unwrap_or(characteristic_idx);
    let z_tail = Complex64::new(z_real[tail_idx], z_imag[tail_idx]);
    let mut alpha_gw_guess =
        (z_tail.im.atan2(z_tail.re).abs().to_degrees() / 90.0).clamp(0.25, 0.75);
    if !alpha_gw_guess.is_finite() {
        alpha_gw_guess = 0.5;
    }

    let w_tail = (2.0 * PI * frequencies[tail_idx].abs()).max(1e-9);
    let sigma_guess = (z_tail.norm() * w_tail.powf(alpha_gw_guess)).clamp(1e-12, 1e12);

    let mut state = GuessState {
        rs: rs_guess,
        r_ct: final_rct_guess,
        q_cpe: q_guess,
        tau_char: (1.0 / w_char).clamp(1e-9, 1e9),
        sigma_gw: sigma_guess,
        alpha_gw: alpha_gw_guess,
        r_count: 0,
    };

    fill_guesses(node, &mut params, &mut state);

    params
}

fn fill_guesses(node: &CircuitNode, params: &mut [f64], state: &mut GuessState) {
    // Walk circuit tree and assign element-specific heuristic initials.
    match node {
        CircuitNode::Series(nodes) | CircuitNode::Parallel(nodes) => {
            for n in nodes {
                fill_guesses(n, params, state);
            }
        }
        CircuitNode::Element(etype, idx, _) => {
            match etype {
                ElementType::R => {
                    if state.r_count == 0 {
                        params[*idx] = state.rs;
                    } else {
                        params[*idx] = state.r_ct;
                    }
                    state.r_count += 1;
                }
                ElementType::C => {
                    params[*idx] = 1e-6;
                }
                ElementType::L => {
                    params[*idx] = 1e-6;
                }
                ElementType::W => {
                    params[*idx] = state.sigma_gw;
                }
                ElementType::Cpe => {
                    params[*idx] = state.q_cpe;
                    params[*idx + 1] = 0.9; // Alpha usually close to 1 for CPE
                }
                ElementType::Wo | ElementType::Ws => {
                    params[*idx] = state.r_ct;
                    params[*idx + 1] = state.tau_char;
                }
                ElementType::La => {
                    params[*idx] = 1e-6;
                    params[*idx + 1] = 1.0;
                }
                ElementType::Gw => {
                    params[*idx] = state.sigma_gw;
                    params[*idx + 1] = state.alpha_gw;
                }
                ElementType::G | ElementType::Gs => {
                    params[*idx] = state.r_ct;
                    params[*idx + 1] = state.tau_char;
                    if let ElementType::Gs = etype {
                        params[*idx + 2] = 0.5;
                    }
                }
                ElementType::K | ElementType::Zarc | ElementType::Tlmq => {
                    params[*idx] = state.r_ct;
                    params[*idx + 1] = state.tau_char;
                    if let ElementType::Tlmq = etype {
                        params[*idx + 1] = state.q_cpe;
                    }
                    if etype.param_count() > 2 {
                        params[*idx + 2] = 0.8;
                    }
                }
                ElementType::T => {
                    params[*idx] = state.r_ct;
                    params[*idx + 1] = state.r_ct;
                    params[*idx + 2] = 1.0;
                    params[*idx + 3] = 1.0;
                }
            }
        }
    }
}

/// Struct for Levenberg-Marquardt optimization.
pub struct ImpedanceFitter {
    /// Circuit topology and element definitions.
    pub circuit: CircuitNode,
    /// Frequency-domain sample points (Hz).
    pub frequencies: Vec<f64>,
    /// Measured real impedance values.
    pub z_real_data: Vec<f64>,
    /// Measured imaginary impedance values.
    pub z_imag_data: Vec<f64>,
    /// Per-sample residual weighting factors.
    pub weights: Vec<f64>,
    /// Current optimizer parameter vector (internal transform space).
    pub params: DVector<f64>,
    /// Per-parameter transform constraints.
    pub constraints: Vec<Constraint>,
    /// Per-parameter min/max bounds in physical space.
    pub bounds: Vec<(f64, f64)>,
}

pub(crate) struct BorrowedImpedanceFitter<'a> {
    /// Borrowed circuit topology.
    pub circuit: &'a CircuitNode,
    /// Angular frequencies in rad/s.
    pub omegas: &'a [f64],
    /// Borrowed real impedance observations.
    pub z_real_data: &'a [f64],
    /// Borrowed imaginary impedance observations.
    pub z_imag_data: &'a [f64],
    /// Borrowed sample weights.
    pub weights: &'a [f64],
    /// Current optimizer parameter vector (internal transform space).
    pub params: DVector<f64>,
    /// Per-parameter transform constraints.
    pub constraints: &'a [Constraint],
    /// Per-parameter min/max bounds in physical space.
    pub bounds: &'a [(f64, f64)],
}

// Helper functions for parameter transformation
/// Convert a physical-space parameter into optimizer internal space.
pub fn transform_forward(physical: f64, constraint: Constraint) -> f64 {
    match constraint {
        Constraint::Positive => {
            if physical <= 0.0 {
                -23.0
            } else {
                physical.ln()
            } // Handle non-positive input gracefully
        }
        Constraint::ZeroOne => {
            if physical <= 0.0 {
                -23.0
            } else if physical >= 1.0 {
                23.0
            } else {
                (physical / (1.0 - physical)).ln()
            }
        }
        Constraint::None => physical,
    }
}

/// Convert an optimizer internal-space parameter back to physical space.
pub fn transform_backward(internal: f64, constraint: Constraint) -> f64 {
    match constraint {
        Constraint::Positive => internal.exp(),
        Constraint::ZeroOne => 1.0 / (1.0 + (-internal).exp()),
        Constraint::None => internal,
    }
}

/// Clamp one scalar value into explicit lower/upper bounds.
pub fn clamp_to_bounds(value: f64, bounds: (f64, f64)) -> f64 {
    let (lower, upper) = bounds;
    if !value.is_finite() {
        return lower;
    }
    value.clamp(lower, upper)
}

/// Sanitize a physical parameter vector against constraints and bounds.
pub fn sanitize_physical_params(
    params: &[f64],
    constraints: &[Constraint],
    bounds: &[(f64, f64)],
) -> Vec<f64> {
    params
        .iter()
        .zip(constraints.iter())
        .zip(bounds.iter())
        .map(|((&value, &constraint), &bound)| {
            let sanitized = match constraint {
                Constraint::Positive => value.max(bound.0),
                Constraint::ZeroOne => value.clamp(bound.0, bound.1),
                Constraint::None => value,
            };
            clamp_to_bounds(sanitized, bound)
        })
        .collect()
}

fn internal_to_physical(
    params: &DVector<f64>,
    constraints: &[Constraint],
    bounds: &[(f64, f64)],
) -> Vec<f64> {
    // Apply inverse transform and enforce physical limits per parameter.
    let mut physical = Vec::with_capacity(params.len());

    for ((&internal, &constraint), &bound) in params.iter().zip(constraints.iter()).zip(bounds) {
        let transformed = transform_backward(internal, constraint);
        let sanitized = match constraint {
            Constraint::Positive => transformed.max(bound.0),
            Constraint::ZeroOne => transformed.clamp(bound.0, bound.1),
            Constraint::None => transformed,
        };
        physical.push(clamp_to_bounds(sanitized, bound));
    }

    physical
}

fn residual_vector_for_omegas(
    circuit: &CircuitNode,
    omegas: &[f64],
    z_real_data: &[f64],
    z_imag_data: &[f64],
    weights: &[f64],
    constraints: &[Constraint],
    bounds: &[(f64, f64)],
    params: &DVector<f64>,
) -> DVector<f64> {
    // Build stacked residual vector [Re residuals..., Im residuals...].
    let n_points = omegas.len();
    let mut residuals = DVector::zeros(2 * n_points);
    let physical_params = internal_to_physical(params, constraints, bounds);

    for i in 0..n_points {
        let z_model = circuit.calculate(omegas[i], &physical_params);
        let weight = weights[i].max(1e-12);
        residuals[i] = (z_model.re - z_real_data[i]) / weight;
        residuals[i + n_points] = (z_model.im - z_imag_data[i]) / weight;
    }

    residuals
}

fn residual_vector_for_frequencies(
    circuit: &CircuitNode,
    frequencies: &[f64],
    z_real_data: &[f64],
    z_imag_data: &[f64],
    weights: &[f64],
    constraints: &[Constraint],
    bounds: &[(f64, f64)],
    params: &DVector<f64>,
) -> DVector<f64> {
    // Frequency-based helper that converts Hz -> rad/s before evaluation.
    let n_points = frequencies.len();
    let mut residuals = DVector::zeros(2 * n_points);
    let physical_params = internal_to_physical(params, constraints, bounds);

    for i in 0..n_points {
        let omega = 2.0 * PI * frequencies[i];
        let z_model = circuit.calculate(omega, &physical_params);
        let weight = weights[i].max(1e-12);
        residuals[i] = (z_model.re - z_real_data[i]) / weight;
        residuals[i + n_points] = (z_model.im - z_imag_data[i]) / weight;
    }

    residuals
}

fn numerical_jacobian<F>(base: &DVector<f64>, residual_len: usize, residuals_for: F) -> DMatrix<f64>
where
    F: Fn(&DVector<f64>) -> DVector<f64>,
{
    // Central-difference Jacobian for Levenberg-Marquardt compatibility.
    let n_params = base.len();
    let mut jacobian = DMatrix::zeros(residual_len, n_params);
    let mut plus = base.clone();
    let mut minus = base.clone();

    for j in 0..n_params {
        let base_value = base[j];
        let step = 1e-4 * base_value.abs().max(1.0);

        plus[j] = base_value + step;
        let residuals_plus = residuals_for(&plus);

        minus[j] = base_value - step;
        let residuals_minus = residuals_for(&minus);

        plus[j] = base_value;
        minus[j] = base_value;

        let inv_step = 1.0 / (2.0 * step);
        for i in 0..residual_len {
            jacobian[(i, j)] = (residuals_plus[i] - residuals_minus[i]) * inv_step;
        }
    }

    jacobian
}

impl ImpedanceFitter {
    // Adapter around shared residual construction for owned-frequency storage.
    fn residual_vector_for_internal(&self, params: &DVector<f64>) -> DVector<f64> {
        residual_vector_for_frequencies(
            &self.circuit,
            &self.frequencies,
            &self.z_real_data,
            &self.z_imag_data,
            &self.weights,
            &self.constraints,
            &self.bounds,
            params,
        )
    }
}

impl LeastSquaresProblem<f64, Dyn, Dyn> for ImpedanceFitter {
    type ParameterStorage = Owned<f64, Dyn>;
    type ResidualStorage = Owned<f64, Dyn>;
    type JacobianStorage = Owned<f64, Dyn, Dyn>;

    fn set_params(&mut self, params: &DVector<f64>) {
        self.params = params.clone();
    }

    fn params(&self) -> DVector<f64> {
        self.params.clone()
    }

    fn residuals(&self) -> Option<DVector<f64>> {
        Some(self.residual_vector_for_internal(&self.params))
    }

    fn jacobian(&self) -> Option<DMatrix<f64>> {
        let residual_len = 2 * self.frequencies.len();
        Some(numerical_jacobian(&self.params, residual_len, |params| {
            self.residual_vector_for_internal(params)
        }))
    }
}

impl<'a> BorrowedImpedanceFitter<'a> {
    // Adapter around shared residual construction for borrowed-omega storage.
    fn residual_vector_for_internal(&self, params: &DVector<f64>) -> DVector<f64> {
        residual_vector_for_omegas(
            self.circuit,
            self.omegas,
            self.z_real_data,
            self.z_imag_data,
            self.weights,
            self.constraints,
            self.bounds,
            params,
        )
    }
}

impl LeastSquaresProblem<f64, Dyn, Dyn> for BorrowedImpedanceFitter<'_> {
    type ParameterStorage = Owned<f64, Dyn>;
    type ResidualStorage = Owned<f64, Dyn>;
    type JacobianStorage = Owned<f64, Dyn, Dyn>;

    fn set_params(&mut self, params: &DVector<f64>) {
        self.params = params.clone();
    }

    fn params(&self) -> DVector<f64> {
        self.params.clone()
    }

    fn residuals(&self) -> Option<DVector<f64>> {
        Some(self.residual_vector_for_internal(&self.params))
    }

    fn jacobian(&self) -> Option<DMatrix<f64>> {
        let residual_len = 2 * self.omegas.len();
        Some(numerical_jacobian(&self.params, residual_len, |params| {
            self.residual_vector_for_internal(params)
        }))
    }
}

/// Solves the Linear Kramers-Kronig validation.
pub fn lin_kk_solver(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    c: f64,
    max_m: usize,
) -> (usize, f64, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = frequencies.len();

    let mut best_m = 0;
    let mut best_mu = 1.0;
    let mut best_z_fit_re = vec![0.0; n];
    let mut best_z_fit_im = vec![0.0; n];
    let mut best_res_real = vec![0.0; n];
    let mut best_res_imag = vec![0.0; n];

    for m in 3..=max_m {
        let min_f = frequencies.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_f = frequencies.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let tau_min = 1.0 / (2.0 * PI * max_f);
        let tau_max = 1.0 / (2.0 * PI * min_f);

        let mut taus = Vec::with_capacity(m);
        if m > 1 {
            for k in 0..m {
                let log_tau =
                    tau_min.ln() + (k as f64 / (m as f64 - 1.0)) * (tau_max / tau_min).ln();
                taus.push(log_tau.exp());
            }
        } else {
            taus.push(tau_min);
        }

        let n_params = m + 1;
        let mut a_mat = DMatrix::<f64>::zeros(2 * n, n_params);
        let mut b_vec = DVector::<f64>::zeros(2 * n);

        for i in 0..n {
            let w = 2.0 * PI * frequencies[i];

            a_mat[(i, 0)] = 1.0;
            a_mat[(i + n, 0)] = 0.0;

            b_vec[i] = z_real[i];
            b_vec[i + n] = z_imag[i];

            for k in 0..m {
                let tau = taus[k];
                let denom = 1.0 + (w * tau).powi(2);

                a_mat[(i, k + 1)] = 1.0 / denom;
                a_mat[(i + n, k + 1)] = -(w * tau) / denom;
            }
        }

        let svd = a_mat.clone().svd(true, true);
        let x = svd
            .solve(&b_vec, 1e-9)
            .unwrap_or_else(|_| DVector::zeros(n_params));

        let r_k = x.rows(1, m);
        let mut sum_pos = 0.0;
        let mut sum_neg = 0.0;
        for &val in r_k.iter() {
            if val >= 0.0 {
                sum_pos += val.abs();
            } else {
                sum_neg += val.abs();
            }
        }

        let mu = if sum_neg > 1e-12 {
            1.0 - sum_pos / sum_neg
        } else {
            -1.0
        };

        let ax = &a_mat * &x;
        let mut z_fit_re = Vec::with_capacity(n);
        let mut z_fit_im = Vec::with_capacity(n);
        for i in 0..n {
            z_fit_re.push(ax[i]);
            z_fit_im.push(ax[i + n]);
        }

        best_m = m;
        best_mu = mu;
        best_res_real = z_real
            .iter()
            .zip(z_fit_re.iter())
            .map(|(a, b)| a - b)
            .collect();
        best_res_imag = z_imag
            .iter()
            .zip(z_fit_im.iter())
            .map(|(a, b)| a - b)
            .collect();
        best_z_fit_re = z_fit_re;
        best_z_fit_im = z_fit_im;

        if mu < c {
            break;
        }
    }

    (
        best_m,
        best_mu,
        best_z_fit_re,
        best_z_fit_im,
        best_res_real,
        best_res_imag,
    )
}
