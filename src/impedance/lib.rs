//! Equivalent-circuit modeling, fitting, and search primitives.
//!
//! This module exports the scientific core used by both direct fitting and
//! evolutionary circuit discovery workflows.

use levenberg_marquardt::LevenbergMarquardt;
use nalgebra::DVector;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::f64::consts::PI;

use crate::domain::FittingError;

pub mod circuit_models;
pub mod circuits;
pub mod ecm_candidate;
pub mod ecm_evolution;
pub mod ecm_scoring;
pub mod ecm_search;
pub mod elements;
pub mod fitting;
pub mod pinn_optimizer;
pub mod reporting;

pub use crate::results::CircuitFitResult;
pub use circuit_models::{
    CircuitModelContext, CircuitModelResolver, CircuitModelRule, DEFAULT_CIRCUIT_MODEL_CONFIG_PATH,
    DEFAULT_EIS_CIRCUIT_MODEL, FitRankingMetric, ModelSelectionConfig,
};
pub use circuits::{CircuitNode, Impedance, parse_circuit_string};
pub use ecm_candidate::{
    CircuitCandidate, CircuitGenome, CircuitTopology, LeafKind, RANDLES_SEED_CIRCUIT,
    candidate_from_genome, genome_from_candidate, genome_from_topology, seed_candidates,
    topology_from_genome,
};
pub use ecm_evolution::{EcmEvolutionConfig, EcmEvolutionOutcome, run_ecm_evolution};
pub use ecm_scoring::{
    CandidateFitResult, EcmRankingCriterion, aic, bic, compare_candidates,
    format_candidate_ranking_table, legacy_penalized_score,
};
pub use ecm_search::{
    EcmSearchConfig, EcmSearchReport, RankedEcmCandidate, discover_equivalent_circuits,
    discover_equivalent_circuits_with_config, format_ranked_candidates_table,
};
pub use elements::{Constraint, ElementType};
pub use fitting::{
    ImpedanceFitter, clamp_to_bounds, guess_parameters, lin_kk_solver, local_covariance,
    sanitize_physical_params, transform_backward, transform_forward,
};
pub use pinn_optimizer::{PinnConfig, PinnOptimizer, PinnResult, compute_aic, compute_bic};
pub use reporting::{
    CircuitCompositionReport, CircuitElementBreakdown, CircuitElementCount,
    CircuitElementParameterValue, describe_fitted_circuit, format_circuit_composition_report,
    format_circuit_fit_report, format_fitted_circuit_composition,
};

pub type LinKkResult = (usize, f64, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>);

/// Additive diagnostics surrounding the backward-compatible fit result.
#[derive(Debug, Clone)]
pub struct DetailedCircuitFit {
    pub legacy_result: CircuitFitResult,
    pub covariance: Option<Vec<Vec<f64>>>,
    pub condition_number: Option<f64>,
    pub jacobian_rank: Option<usize>,
}

pub fn fit_circuit_detailed(
    circuit_str: &str,
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Result<DetailedCircuitFit, FittingError> {
    let legacy_result = fit_circuit(circuit_str, frequencies, z_real, z_imag, phase_deg)?;
    let circuit = parse_circuit_string(circuit_str)?;
    let weights = z_real
        .iter()
        .zip(z_imag)
        .map(|(re, im)| re.hypot(*im).max(1.0))
        .collect::<Vec<_>>();
    let (covariance, condition_number, jacobian_rank) = local_covariance(
        &circuit,
        frequencies,
        z_real,
        z_imag,
        &legacy_result.fitted_parameters,
        &weights,
    );
    Ok(DetailedCircuitFit {
        legacy_result,
        covariance,
        condition_number,
        jacobian_rank,
    })
}

/// Fit a circuit expression to measured impedance data.
///
/// Inputs:
/// - `circuit_str`: textual circuit expression (for example `R0-p(CPE1,R1)`).
/// - `frequencies`, `z_real`, `z_imag`, `phase_deg`: measured series.
///
/// Outputs:
/// - optimized parameter values plus rendered fitted series.
///
/// Side effects:
/// - none; this function is deterministic for a given input.
///
/// Errors:
/// - parse errors, invalid-length input vectors, or optimizer failure.
pub fn fit_circuit(
    circuit_str: &str,
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Result<CircuitFitResult, FittingError> {
    validate_input_lengths(frequencies, z_real, z_imag, phase_deg)?;

    let prepared = prepare_impedance_data(frequencies, z_real, z_imag, phase_deg)?;
    let circuit = parse_circuit_string(circuit_str)?;
    fit_circuit_with_circuit(&circuit, frequencies, &prepared)
}

pub(crate) fn fit_circuit_with_circuit(
    circuit: &CircuitNode,
    frequencies: &[f64],
    prepared: &PreparedImpedanceData,
) -> Result<CircuitFitResult, FittingError> {
    let constraints = circuit.get_constraints();
    let bounds = circuit.get_bounds();

    let base_guess = sanitize_physical_params(
        &guess_parameters(
            circuit,
            &prepared.frequencies,
            &prepared.z_real,
            &prepared.z_imag,
            &prepared.phase_deg,
        ),
        &constraints,
        &bounds,
    );
    let initial_candidates = build_initial_guesses(circuit, &base_guess, &constraints, &bounds);

    let fit_work = prepared.omegas.len() * circuit.count_total_params();
    let evaluated_guesses: Vec<(usize, Vec<f64>, f64)> =
        if initial_candidates.len() > 1 && fit_work >= 256 {
            initial_candidates
                .into_par_iter()
                .enumerate()
                .map(|(index, guess)| {
                    let (fitted_params, score) =
                        optimize_initial_guess(circuit, prepared, &constraints, &bounds, guess);
                    (index, fitted_params, score)
                })
                .collect()
        } else {
            initial_candidates
                .into_iter()
                .enumerate()
                .map(|(index, guess)| {
                    let (fitted_params, score) =
                        optimize_initial_guess(circuit, prepared, &constraints, &bounds, guess);
                    (index, fitted_params, score)
                })
                .collect()
        };

    let best_result = select_best_result(evaluated_guesses);
    let fitted_params_physical = best_result
        .map(|(_, params, _)| params)
        .ok_or_else(|| FittingError::optimizer("optimizer failed to produce a fit"))?;

    let param_names = circuit.get_param_names();
    let param_units = circuit.get_param_units();
    let mut fitted_real = Vec::with_capacity(frequencies.len());
    let mut fitted_imag = Vec::with_capacity(frequencies.len());
    let mut fitted_mag = Vec::with_capacity(frequencies.len());
    let mut fitted_phase = Vec::with_capacity(frequencies.len());

    for &f in frequencies {
        let omega = 2.0 * PI * f;
        let z = circuit.calculate(omega, &fitted_params_physical);

        fitted_real.push(z.re);
        fitted_imag.push(z.im);
        fitted_mag.push(z.norm());
        fitted_phase.push(z.im.atan2(z.re).to_degrees());
    }

    Ok(CircuitFitResult {
        fitted_parameters: fitted_params_physical,
        parameter_names: param_names,
        parameter_units: param_units,
        fitted_z_re: fitted_real,
        fitted_z_im: fitted_imag,
        fitted_magnitude: fitted_mag,
        fitted_phase,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedImpedanceData {
    pub frequencies: Vec<f64>,
    pub omegas: Vec<f64>,
    pub z_real: Vec<f64>,
    pub z_imag: Vec<f64>,
    pub phase_deg: Vec<f64>,
    pub weights: Vec<f64>,
}

pub(crate) fn prepare_impedance_data(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Result<PreparedImpedanceData, FittingError> {
    let mut rows = Vec::new();

    for idx in 0..frequencies.len() {
        let f = frequencies[idx];
        let re = z_real[idx];
        let im = z_imag[idx];
        let phase = if phase_deg.is_empty() {
            im.atan2(re).to_degrees()
        } else {
            phase_deg[idx]
        };

        if f.is_finite() && f > 0.0 && re.is_finite() && im.is_finite() && phase.is_finite() {
            rows.push((f, re, im, phase));
        }
    }

    if rows.len() < 3 {
        return Err(FittingError::invalid_input(
            "not enough valid impedance points after preprocessing",
        ));
    }

    rows.sort_by(|lhs, rhs| {
        rhs.0
            .partial_cmp(&lhs.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut prepared_frequencies = Vec::with_capacity(rows.len());
    let mut prepared_omegas = Vec::with_capacity(rows.len());
    let mut prepared_real = Vec::with_capacity(rows.len());
    let mut prepared_imag = Vec::with_capacity(rows.len());
    let mut prepared_phase = Vec::with_capacity(rows.len());
    let mut prepared_weights = Vec::with_capacity(rows.len());

    for (frequency, real, imag, phase) in rows {
        prepared_frequencies.push(frequency);
        prepared_omegas.push(2.0 * PI * frequency);
        prepared_real.push(real);
        prepared_imag.push(imag);
        prepared_phase.push(phase);
        prepared_weights.push(real.hypot(imag).max(1.0));
    }

    Ok(PreparedImpedanceData {
        frequencies: prepared_frequencies,
        omegas: prepared_omegas,
        z_real: prepared_real,
        z_imag: prepared_imag,
        phase_deg: prepared_phase,
        weights: prepared_weights,
    })
}

#[cfg(test)]
fn build_modulus_weights(z_real: &[f64], z_imag: &[f64]) -> Vec<f64> {
    z_real
        .iter()
        .zip(z_imag.iter())
        .map(|(&re, &im)| re.hypot(im).max(1.0))
        .collect()
}

fn build_initial_guesses(
    circuit: &CircuitNode,
    base_guess: &[f64],
    constraints: &[Constraint],
    bounds: &[(f64, f64)],
) -> Vec<Vec<f64>> {
    let param_names = circuit.get_param_names();
    let mut candidates = vec![base_guess.to_vec()];

    let mut alpha_indices = Vec::new();
    let mut q_indices = Vec::new();
    let mut r_indices = Vec::new();

    for (idx, name) in param_names.iter().enumerate() {
        if name.starts_with("alpha_") || name.starts_with("gamma_") {
            alpha_indices.push(idx);
        } else if name.starts_with("Q_") || name.starts_with("Qs_") {
            q_indices.push(idx);
        } else if name.starts_with("R_") || name.starts_with("Rion_") || name.starts_with("R_G_") {
            r_indices.push(idx);
        }
    }

    if !alpha_indices.is_empty() || !q_indices.is_empty() || !r_indices.is_empty() {
        let mut low_alpha = base_guess.to_vec();
        for &idx in &alpha_indices {
            low_alpha[idx] = clamp_to_bounds(0.7, bounds[idx]);
        }
        for &idx in &q_indices {
            low_alpha[idx] = clamp_to_bounds(base_guess[idx] * 0.25, bounds[idx]);
        }
        candidates.push(sanitize_physical_params(&low_alpha, constraints, bounds));

        let mut high_alpha = base_guess.to_vec();
        for &idx in &alpha_indices {
            high_alpha[idx] = clamp_to_bounds(0.9, bounds[idx]);
        }
        for &idx in &q_indices {
            high_alpha[idx] = clamp_to_bounds(base_guess[idx] * 4.0, bounds[idx]);
        }
        candidates.push(sanitize_physical_params(&high_alpha, constraints, bounds));

        let mut lower_resistance = base_guess.to_vec();
        for (rank, &idx) in r_indices.iter().enumerate() {
            let scale = if rank == 0 { 0.8 } else { 0.5 };
            lower_resistance[idx] = clamp_to_bounds(base_guess[idx] * scale, bounds[idx]);
        }
        candidates.push(sanitize_physical_params(
            &lower_resistance,
            constraints,
            bounds,
        ));

        let mut higher_resistance = base_guess.to_vec();
        for (rank, &idx) in r_indices.iter().enumerate() {
            let scale = if rank == 0 { 1.2 } else { 2.0 };
            higher_resistance[idx] = clamp_to_bounds(base_guess[idx] * scale, bounds[idx]);
        }
        candidates.push(sanitize_physical_params(
            &higher_resistance,
            constraints,
            bounds,
        ));
    }

    candidates
}

fn optimize_initial_guess(
    circuit: &CircuitNode,
    prepared: &PreparedImpedanceData,
    constraints: &[Constraint],
    bounds: &[(f64, f64)],
    guess: Vec<f64>,
) -> (Vec<f64>, f64) {
    let initial_params_internal: Vec<f64> = guess
        .iter()
        .zip(constraints.iter())
        .map(|(&p, &c)| transform_forward(p, c))
        .collect();

    let fitter = fitting::BorrowedImpedanceFitter {
        circuit,
        omegas: &prepared.omegas,
        z_real_data: &prepared.z_real,
        z_imag_data: &prepared.z_imag,
        weights: &prepared.weights,
        params: DVector::from_vec(initial_params_internal),
        constraints,
        bounds,
    };

    let solver = LevenbergMarquardt::new()
        .with_ftol(1e-10)
        .with_xtol(1e-10)
        .with_gtol(1e-10)
        .with_patience(400)
        .with_stepbound(50.0);

    let (result, report) = solver.minimize(fitter);
    let fitted_params_physical = sanitize_physical_params(
        &result
            .params
            .iter()
            .zip(constraints.iter())
            .map(|(&p, &c)| transform_backward(p, c))
            .collect::<Vec<_>>(),
        constraints,
        bounds,
    );

    (fitted_params_physical, report.objective_function)
}

fn select_best_result(
    evaluated_guesses: Vec<(usize, Vec<f64>, f64)>,
) -> Option<(usize, Vec<f64>, f64)> {
    evaluated_guesses.into_iter().min_by(|lhs, rhs| {
        lhs.2
            .partial_cmp(&rhs.2)
            .unwrap_or(Ordering::Equal)
            .then_with(|| lhs.0.cmp(&rhs.0))
    })
}

pub fn ignore_below_x(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut f_out = Vec::new();
    let mut z_real_out = Vec::new();
    let mut z_imag_out = Vec::new();

    for i in 0..frequencies.len() {
        if z_imag[i] < 0.0 {
            f_out.push(frequencies[i]);
            z_real_out.push(z_real[i]);
            z_imag_out.push(z_imag[i]);
        }
    }
    (f_out, z_real_out, z_imag_out)
}

pub fn crop_frequencies(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    freqmin: Option<f64>,
    freqmax: Option<f64>,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut f_out = Vec::new();
    let mut z_real_out = Vec::new();
    let mut z_imag_out = Vec::new();

    let min_f = freqmin.unwrap_or(0.0);
    let max_f = freqmax.unwrap_or(f64::INFINITY);

    for i in 0..frequencies.len() {
        if frequencies[i] >= min_f && frequencies[i] <= max_f {
            f_out.push(frequencies[i]);
            z_real_out.push(z_real[i]);
            z_imag_out.push(z_imag[i]);
        }
    }
    (f_out, z_real_out, z_imag_out)
}

pub fn lin_kk(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    c: f64,
    max_m: usize,
) -> Result<LinKkResult, FittingError> {
    validate_input_lengths(frequencies, z_real, z_imag, &[])?;
    Ok(lin_kk_solver(frequencies, z_real, z_imag, c, max_m))
}

pub(crate) fn validate_input_lengths(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Result<(), FittingError> {
    if frequencies.is_empty() {
        return Err(FittingError::invalid_input("frequencies cannot be empty"));
    }

    if frequencies.len() != z_real.len() || frequencies.len() != z_imag.len() {
        return Err(FittingError::invalid_input(format!(
            "frequencies, z_real, and z_imag must have the same length; got {}, {}, {}",
            frequencies.len(),
            z_real.len(),
            z_imag.len()
        )));
    }

    if !phase_deg.is_empty() && frequencies.len() != phase_deg.len() {
        return Err(FittingError::invalid_input(format!(
            "phase_deg must have the same length as frequencies when provided; got {} and {}",
            phase_deg.len(),
            frequencies.len()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        Impedance, build_modulus_weights, crop_frequencies, fit_circuit, ignore_below_x,
        parse_circuit_string, prepare_impedance_data,
    };
    use num_complex::Complex64;
    use std::f64::consts::PI;

    fn weighted_rmse(z_real: &[f64], z_imag: &[f64], fit_real: &[f64], fit_imag: &[f64]) -> f64 {
        let weights = build_modulus_weights(z_real, z_imag);
        let mut sum = 0.0;
        let mut count = 0.0;

        for idx in 0..z_real.len() {
            let weight = weights[idx];
            let re = (fit_real[idx] - z_real[idx]) / weight;
            let im = (fit_imag[idx] - z_imag[idx]) / weight;
            sum += re * re + im * im;
            count += 2.0;
        }

        (sum / count).sqrt()
    }

    #[test]
    fn filters_negative_imaginary_values() {
        let frequencies = vec![10.0, 20.0, 30.0];
        let z_real = vec![1.0, 2.0, 3.0];
        let z_imag = vec![0.5, -0.25, -0.75];

        let (filtered_f, filtered_re, filtered_im) = ignore_below_x(&frequencies, &z_real, &z_imag);

        assert_eq!(filtered_f, vec![20.0, 30.0]);
        assert_eq!(filtered_re, vec![2.0, 3.0]);
        assert_eq!(filtered_im, vec![-0.25, -0.75]);
    }

    #[test]
    fn crops_frequency_range() {
        let frequencies = vec![1.0, 10.0, 100.0];
        let z_real = vec![4.0, 5.0, 6.0];
        let z_imag = vec![-4.0, -5.0, -6.0];

        let (filtered_f, filtered_re, filtered_im) =
            crop_frequencies(&frequencies, &z_real, &z_imag, Some(5.0), Some(50.0));

        assert_eq!(filtered_f, vec![10.0]);
        assert_eq!(filtered_re, vec![5.0]);
        assert_eq!(filtered_im, vec![-5.0]);
    }

    #[test]
    fn preprocesses_impedance_points_by_frequency() {
        let prepared = prepare_impedance_data(
            &[1.0, 100.0, 10.0],
            &[3.0, 1.0, 2.0],
            &[-3.0, -1.0, -2.0],
            &[-45.0, -5.0, -20.0],
        )
        .expect("prepare data");

        assert_eq!(prepared.frequencies, vec![100.0, 10.0, 1.0]);
        assert_eq!(prepared.z_real, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn fits_real_eis_dataset_with_bounded_weighted_error() {
        let csv = std::fs::read_to_string(
            "/Users/xingyuwang/ProjectOngoing/rust_plots/data/EIS/20260312/20260312_QD_EIS (0.1M).csv",
        )
        .expect("read dataset");

        let mut freq = Vec::new();
        let mut z_real = Vec::new();
        let mut z_imag = Vec::new();
        let mut phase = Vec::new();

        let mut header_seen = false;
        for line in csv.lines() {
            if !header_seen {
                if line.starts_with("Freq/Hz") {
                    header_seen = true;
                }
                continue;
            }

            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<_> = line.split(',').map(|field| field.trim()).collect();
            if parts.len() < 5 {
                continue;
            }

            freq.push(parts[0].parse::<f64>().expect("freq"));
            z_real.push(parts[1].parse::<f64>().expect("z real"));
            z_imag.push(parts[2].parse::<f64>().expect("z imag"));
            phase.push(parts[4].parse::<f64>().expect("phase"));
        }

        let fit =
            fit_circuit("R0-p(CPE1,R1)", &freq, &z_real, &z_imag, &phase).expect("fit dataset");

        let error = weighted_rmse(&z_real, &z_imag, &fit.fitted_z_re, &fit.fitted_z_im);
        assert!(error < 0.35, "weighted RMSE too high: {error}");
    }

    #[test]
    fn cpe_matches_ideal_capacitor_when_alpha_is_one() {
        let circuit = parse_circuit_string("CPE1").expect("parse CPE circuit");
        let omega = 2.0 * PI * 1_000.0;
        let q = 2.5e-6;

        let z_cpe = circuit.calculate(omega, &[q, 1.0]);
        let z_cap = Complex64::new(0.0, -1.0 / (omega * q));

        assert!((z_cpe.re - z_cap.re).abs() < 1e-9);
        assert!((z_cpe.im - z_cap.im).abs() < 1e-9);
    }

    #[test]
    fn randles_parallel_branch_has_expected_apex_for_rc_limit() {
        let circuit = parse_circuit_string("R0-p(CPE1,R1)").expect("parse Randles circuit");
        let rs = 5.0;
        let rct = 50.0;
        let c = 2.0e-5;
        let omega_peak = 1.0 / (rct * c);
        let z = circuit.calculate(omega_peak, &[rs, c, 1.0, rct]);

        assert!((z.re - (rs + 0.5 * rct)).abs() < 1e-6);
        assert!((z.im + 0.5 * rct).abs() < 1e-6);
    }

    #[test]
    fn generalized_warburg_outperforms_fixed_warburg_for_ism_dataset() {
        let fixture = std::path::Path::new(
            "/Users/xingyuwang/ProjectOngoing/rust_plots/data/EIS/20260312/20260312_QD-Li-ISM-2_EIS (0.1M).csv",
        );
        if !fixture.exists() {
            eprintln!(
                "skipping external ISM comparison fixture: {}",
                fixture.display()
            );
            return;
        }
        let data =
            crate::data_file::chi_file::EISData::parse_file(fixture).expect("parse li dataset");

        let baseline_fit = data
            .fit_circuit_for_model("R0-p(CPE1,R1)")
            .expect("fit baseline model");
        let warburg_fit = data
            .fit_circuit_for_model("R0-p(CPE1,R1)-W2")
            .expect("fit fixed warburg model");
        let generalized_fit = data
            .fit_circuit_for_model("R0-p(CPE1,R1)-Gw2")
            .expect("fit generalized warburg model");

        let baseline_metrics = data.fit_metrics(&baseline_fit);
        let warburg_metrics = data.fit_metrics(&warburg_fit);
        let generalized_metrics = data.fit_metrics(&generalized_fit);

        assert!(
            generalized_metrics.weighted_rmse < warburg_metrics.weighted_rmse,
            "expected generalized Warburg to beat fixed Warburg: {:?} vs {:?}",
            generalized_metrics,
            warburg_metrics,
        );
        assert!(
            generalized_metrics.weighted_rmse < baseline_metrics.weighted_rmse,
            "expected generalized Warburg to beat baseline: {:?} vs {:?}",
            generalized_metrics,
            baseline_metrics,
        );
        assert!(
            generalized_metrics.weighted_rmse < 0.08,
            "expected Li tail fit weighted RMSE below 0.08, got {}",
            generalized_metrics.weighted_rmse,
        );
        assert!(
            generalized_metrics.aic < warburg_metrics.aic,
            "expected generalized Warburg to have lower AIC than fixed Warburg",
        );
    }
}
