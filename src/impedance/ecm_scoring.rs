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
use serde::{Deserialize, Serialize};

/// Scalar objective used consistently by ECM evolution, ranking, and reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EcmRankingCriterion {
    #[default]
    Bic,
    Aic,
    WeightedRmse,
    LegacyPenalizedScore,
}

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
    /// Gaussian-residual AIC using two scalar observations per frequency.
    pub aic: Option<f64>,
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
    let rss = effective_rss(residual_sum_of_squares, n_obs)?;
    if n_obs == 0 || parameter_count == 0 {
        return None;
    }
    let n = n_obs as f64;
    Some(n * (rss / n).ln() + parameter_count as f64 * n.ln())
}

/// Standard Gaussian-residual AIC using independent real and imaginary
/// observations.
pub fn aic(residual_sum_of_squares: f64, parameter_count: usize, n_obs: usize) -> Option<f64> {
    let rss = effective_rss(residual_sum_of_squares, n_obs)?;
    if n_obs == 0 || parameter_count == 0 {
        return None;
    }
    let n = n_obs as f64;
    Some(n * (rss / n).ln() + 2.0 * parameter_count as f64)
}

/// Exact zero RSS is a perfect fit, not a numerical failure. Nonfinite RSS
/// remains unavailable so failed optimizations cannot rank as perfect fits.
pub fn effective_rss(residual_sum_of_squares: f64, n_obs: usize) -> Option<f64> {
    if n_obs == 0 || !residual_sum_of_squares.is_finite() || residual_sum_of_squares < 0.0 {
        return None;
    }
    Some(residual_sum_of_squares.max(f64::EPSILON * n_obs as f64))
}

pub fn ranking_value(result: &CandidateFitResult, criterion: EcmRankingCriterion) -> Option<f64> {
    match criterion {
        EcmRankingCriterion::Bic => result.bic,
        EcmRankingCriterion::Aic => result.aic,
        EcmRankingCriterion::WeightedRmse => Some(result.weighted_rmse),
        EcmRankingCriterion::LegacyPenalizedScore => result.legacy_penalized_score,
    }
    .filter(|value| value.is_finite())
}

/// Available finite objectives sort first, followed by the legacy score and
/// canonical circuit string as deterministic tie-breakers.
pub fn compare_candidates(
    left: &CandidateFitResult,
    right: &CandidateFitResult,
    criterion: EcmRankingCriterion,
) -> std::cmp::Ordering {
    ranking_value(left, criterion)
        .is_none()
        .cmp(&ranking_value(right, criterion).is_none())
        .then_with(|| {
            ranking_value(left, criterion)
                .zip(ranking_value(right, criterion))
                .map_or(std::cmp::Ordering::Equal, |(a, b)| a.total_cmp(&b))
        })
        .then_with(|| {
            left.legacy_penalized_score
                .filter(|v| v.is_finite())
                .unwrap_or(f64::INFINITY)
                .total_cmp(
                    &right
                        .legacy_penalized_score
                        .filter(|v| v.is_finite())
                        .unwrap_or(f64::INFINITY),
                )
        })
        .then_with(|| left.circuit_string.cmp(&right.circuit_string))
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
    let n_obs = 2 * point_count;
    let bic = bic(residual_sum_of_squares, parameter_count, n_obs);
    let aic = aic(residual_sum_of_squares, parameter_count, n_obs);

    Ok(CandidateFitResult {
        circuit_string: circuit_str.to_string(),
        residual_sum_of_squares,
        weighted_residual_sum_of_squares,
        bic,
        aic,
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
            "{} | {} | {} | {} | {} | {}",
            index + 1,
            result.circuit_string,
            format_metric(result.residual_sum_of_squares),
            format_metric_option(result.bic),
            format_metric_option(result.legacy_penalized_score),
            result.parameter_count
        ));
    }

    lines.join("\n")
}

fn format_metric(value: f64) -> String {
    format_metric_option(Some(value))
}

fn format_metric_option(value: Option<f64>) -> String {
    value
        .filter(|value| value.is_finite())
        .map_or_else(|| "".to_string(), |value| format!("{value:.6e}"))
}

#[cfg(test)]
mod tests {
    use super::{
        CandidateFitResult, EcmRankingCriterion, aic, bic, compare_candidates,
        legacy_penalized_score,
    };

    fn candidate(name: &str, bic: Option<f64>, aic: Option<f64>) -> CandidateFitResult {
        CandidateFitResult {
            circuit_string: name.into(),
            residual_sum_of_squares: 1.0,
            weighted_residual_sum_of_squares: Some(1.0),
            bic,
            aic,
            legacy_penalized_score: Some(1.0),
            weighted_rmse: 1.0,
            parameter_count: 1,
            fitted_parameters: Vec::new(),
            parameter_names: Vec::new(),
            parameter_units: Vec::new(),
            fitted_z_re: Vec::new(),
            fitted_z_im: Vec::new(),
            fitted_magnitude: Vec::new(),
            fitted_phase: Vec::new(),
        }
    }

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
        assert!(bic(0.0, 3, 20).unwrap().is_finite());
        assert!(bic(1.0, 0, 20).is_none());
    }

    #[test]
    fn aic_uses_two_scalar_observations_and_perfect_fit_is_finite() {
        let value = aic(2.0, 3, 20).unwrap();
        let expected = 20.0 * (2.0_f64 / 20.0).ln() + 2.0 * 3.0;
        assert!((value - expected).abs() < 1e-12);
        assert!(aic(0.0, 3, 20).unwrap().is_finite());
    }

    #[test]
    fn unavailable_objectives_sort_after_finite_values() {
        let unavailable = candidate("bad", None, None);
        let valid = candidate("good", Some(1.0), Some(1.0));
        assert_eq!(
            compare_candidates(&valid, &unavailable, EcmRankingCriterion::Bic),
            std::cmp::Ordering::Less
        );
    }
}
