//! Candidate scoring metrics for equivalent-circuit ranking.
//!
//! BIC is computed under an explicit Gaussian residual assumption. Each
//! complex impedance point contributes two independent scalar observations
//! (real and imaginary residuals), so `n_obs = 2 * n_frequency_points`.
//! Modulus-normalized residuals are retained only as a legacy ranking signal;
//! they are not a statistically calibrated chi-square.

use super::{
    CircuitNode, PreparedImpedanceData, fit_circuit_with_circuit, parse_circuit_string,
    prepare_impedance_data, validate_input_lengths,
};
use crate::domain::FittingError;

/// Full fit output used to rank and report one candidate circuit.
#[derive(Debug, Clone)]
pub struct CandidateFitResult {
    pub circuit_string: String,
    /// Unweighted sum of squared real and imaginary scalar residuals.
    pub residual_sum_of_squares: f64,
    /// Legacy modulus-normalized sum of squared residuals. This is not a
    /// chi-square because measurement variances are not supplied.
    pub weighted_residual_sum_of_squares: Option<f64>,
    /// Gaussian-residual BIC, or `None` when its inputs are invalid.
    pub bic: Option<f64>,
    /// Former modulus-normalized score, retained under an explicit name.
    pub legacy_penalized_score: Option<f64>,
    pub weighted_rmse: f64,
    pub parameter_count: usize,
    pub fitted_parameters: Vec<f64>,
    pub parameter_names: Vec<String>,
    pub parameter_units: Vec<String>,
    pub fitted_z_re: Vec<f64>,
    pub fitted_z_im: Vec<f64>,
    pub fitted_magnitude: Vec<f64>,
    pub fitted_phase: Vec<f64>,
}

/// Former modulus-normalized residual objective.
pub fn legacy_penalized_score(
    z_re: &[f64],
    z_im: &[f64],
    fitted_z_re: &[f64],
    fitted_z_im: &[f64],
) -> f64 {
    z_re.iter()
        .zip(z_im.iter())
        .zip(fitted_z_re.iter().zip(fitted_z_im.iter()))
        .map(|((&exp_re, &exp_im), (&fit_re, &fit_im))| {
            let residual_re = exp_re - fit_re;
            let residual_im = exp_im - fit_im;
            let exp_modulus = exp_re.hypot(exp_im).max(1e-12);
            (residual_re * residual_re + residual_im * residual_im) / exp_modulus
        })
        .sum()
}

/// Standard BIC for independent Gaussian scalar residuals.
///
/// `n_obs` must count scalar residuals, not complex frequency points.
pub fn bic(residual_sum_of_squares: f64, parameter_count: usize, n_obs: usize) -> Option<f64> {
    if n_obs == 0
        || parameter_count == 0
        || !residual_sum_of_squares.is_finite()
        || residual_sum_of_squares <= 0.0
    {
        return None;
    }
    let n = n_obs as f64;
    Some(n * (residual_sum_of_squares / n).ln() + parameter_count as f64 * n.ln())
}

/// Weighted RMSE in complex-impedance space.
pub fn weighted_rmse(z_re: &[f64], z_im: &[f64], fitted_z_re: &[f64], fitted_z_im: &[f64]) -> f64 {
    let count = z_re
        .len()
        .min(z_im.len())
        .min(fitted_z_re.len())
        .min(fitted_z_im.len());

    if count == 0 {
        return 0.0;
    }

    let weighted_sse: f64 = z_re
        .iter()
        .zip(z_im.iter())
        .zip(fitted_z_re.iter().zip(fitted_z_im.iter()))
        .take(count)
        .map(|((&exp_re, &exp_im), (&fit_re, &fit_im))| {
            let weight = exp_re.hypot(exp_im).max(1.0);
            ((fit_re - exp_re) / weight).powi(2) + ((fit_im - exp_im) / weight).powi(2)
        })
        .sum();

    (weighted_sse / (2.0 * count as f64)).sqrt()
}

pub fn score_circuit(
    circuit_str: &str,
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Result<CandidateFitResult, FittingError> {
    let circuit = parse_circuit_string(circuit_str)?;
    validate_input_lengths(frequencies, z_real, z_imag, phase_deg)?;
    let prepared = prepare_impedance_data(frequencies, z_real, z_imag, phase_deg)?;
    score_parsed_circuit(
        circuit_str,
        &circuit,
        frequencies,
        z_real,
        z_imag,
        &prepared,
    )
}

#[allow(dead_code)]
pub(crate) fn score_circuit_with_prepared(
    circuit_str: &str,
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    prepared: &PreparedImpedanceData,
) -> Result<CandidateFitResult, FittingError> {
    let circuit = parse_circuit_string(circuit_str)?;
    score_parsed_circuit(circuit_str, &circuit, frequencies, z_real, z_imag, prepared)
}

fn score_parsed_circuit(
    circuit_str: &str,
    circuit: &CircuitNode,
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    prepared: &PreparedImpedanceData,
) -> Result<CandidateFitResult, FittingError> {
    let parameter_count = circuit.count_total_params();

    let fit = fit_circuit_with_circuit(circuit, frequencies, prepared)?;
    let fitted_z_re = fit.fitted_z_re;
    let fitted_z_im = fit.fitted_z_im;
    let fitted_magnitude = fit.fitted_magnitude;
    let fitted_phase = fit.fitted_phase;

    let point_count = frequencies
        .len()
        .min(z_real.len())
        .min(z_imag.len())
        .min(fitted_z_re.len())
        .min(fitted_z_im.len());

    if point_count == 0 {
        return Err(FittingError::invalid_input(
            "candidate fit did not produce any impedance points",
        ));
    }

    let residual_sum_of_squares = z_real
        .iter()
        .zip(z_imag)
        .zip(fitted_z_re.iter().zip(&fitted_z_im))
        .take(point_count)
        .map(|((&re, &im), (&fit_re, &fit_im))| (re - fit_re).powi(2) + (im - fit_im).powi(2))
        .sum::<f64>();
    let legacy_penalized_score = legacy_penalized_score(z_real, z_imag, &fitted_z_re, &fitted_z_im);
    let weighted_residual_sum_of_squares = Some(legacy_penalized_score);
    let weighted_rmse = weighted_rmse(z_real, z_imag, &fitted_z_re, &fitted_z_im);
    let bic = bic(residual_sum_of_squares, parameter_count, 2 * point_count);

    Ok(CandidateFitResult {
        circuit_string: circuit_str.to_string(),
        residual_sum_of_squares,
        weighted_residual_sum_of_squares,
        bic,
        legacy_penalized_score: Some(legacy_penalized_score),
        weighted_rmse,
        parameter_count,
        fitted_parameters: fit.fitted_parameters,
        parameter_names: fit.parameter_names,
        parameter_units: fit.parameter_units,
        fitted_z_re,
        fitted_z_im,
        fitted_magnitude,
        fitted_phase,
    })
}

pub fn format_candidate_ranking_table(results: &[CandidateFitResult]) -> String {
    let mut lines = Vec::with_capacity(results.len() + 2);
    lines.push("Rank | Circuit String | RSS | BIC | Legacy Score | Parameter Count".to_string());
    lines.push("---- | -------------- | --- | --- | ------------ | ---------------".to_string());

    for (index, result) in results.iter().enumerate() {
        lines.push(format!(
            "{} | {} | {:.6e} | {:.6e} | {:.6e} | {}",
            index + 1,
            result.circuit_string,
            result.residual_sum_of_squares,
            result.bic.unwrap_or(f64::NAN),
            result.legacy_penalized_score.unwrap_or(f64::NAN),
            result.parameter_count
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{bic, legacy_penalized_score};

    #[test]
    fn legacy_score_is_zero_for_perfect_fit() {
        let z_re = [1.0, 2.0, 3.0];
        let z_im = [-1.0, -2.0, -3.0];

        assert_eq!(legacy_penalized_score(&z_re, &z_im, &z_re, &z_im), 0.0);
    }

    #[test]
    fn bic_penalizes_parameter_growth() {
        let base = bic(1.0, 3, 100).unwrap();
        let more_complex = bic(1.0, 6, 100).unwrap();

        assert!(more_complex > base);
    }

    #[test]
    fn bic_uses_two_scalar_observations_per_complex_point() {
        let value = bic(2.0, 3, 20).unwrap();
        let expected = 20.0 * (2.0_f64 / 20.0).ln() + 3.0 * 20.0_f64.ln();
        assert!((value - expected).abs() < 1e-12);
        assert!(bic(0.0, 3, 20).is_none());
        assert!(bic(1.0, 0, 20).is_none());
    }
}
