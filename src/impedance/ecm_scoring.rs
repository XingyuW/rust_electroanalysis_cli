//! Candidate scoring metrics for equivalent-circuit ranking.
//!
//! This module keeps ranking math (`chi_square`, `BIC`, weighted RMSE)
//! isolated so search strategies can reuse the same evaluation rules.

use super::{
    CircuitNode, PreparedImpedanceData, fit_circuit_with_circuit, parse_circuit_string,
    prepare_impedance_data, validate_input_lengths,
};

/// Full fit output used to rank and report one candidate circuit.
#[derive(Debug, Clone)]
pub struct CandidateFitResult {
    pub circuit_string: String,
    pub chi_square: f64,
    pub bic: f64,
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

/// Weighted chi-square objective used as the primary goodness-of-fit signal.
pub fn chi_square(z_re: &[f64], z_im: &[f64], fitted_z_re: &[f64], fitted_z_im: &[f64]) -> f64 {
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

/// Bayesian Information Criterion used to penalize over-parameterized models.
pub fn bic(chi_square: f64, parameter_count: usize, point_count: usize) -> f64 {
    if point_count == 0 {
        return f64::INFINITY;
    }

    parameter_count as f64 * (point_count as f64).ln() + chi_square
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
) -> Result<CandidateFitResult, String> {
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
) -> Result<CandidateFitResult, String> {
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
) -> Result<CandidateFitResult, String> {
    let parameter_count = circuit.count_total_params();

    let (
        fitted_parameters,
        parameter_names,
        parameter_units,
        fitted_z_re,
        fitted_z_im,
        fitted_magnitude,
        fitted_phase,
    ) = fit_circuit_with_circuit(circuit, frequencies, prepared)?;

    let point_count = frequencies
        .len()
        .min(z_real.len())
        .min(z_imag.len())
        .min(fitted_z_re.len())
        .min(fitted_z_im.len());

    if point_count == 0 {
        return Err("candidate fit did not produce any impedance points".to_string());
    }

    let chi_square = chi_square(z_real, z_imag, &fitted_z_re, &fitted_z_im);
    let weighted_rmse = weighted_rmse(z_real, z_imag, &fitted_z_re, &fitted_z_im);
    let bic = bic(chi_square, parameter_count, point_count);

    Ok(CandidateFitResult {
        circuit_string: circuit_str.to_string(),
        chi_square,
        bic,
        weighted_rmse,
        parameter_count,
        fitted_parameters,
        parameter_names,
        parameter_units,
        fitted_z_re,
        fitted_z_im,
        fitted_magnitude,
        fitted_phase,
    })
}

pub fn format_candidate_ranking_table(results: &[CandidateFitResult]) -> String {
    let mut lines = Vec::with_capacity(results.len() + 2);
    lines.push("Rank | Circuit String | chi^2 | BIC | Parameter Count".to_string());
    lines.push("---- | -------------- | ----- | --- | ---------------".to_string());

    for (index, result) in results.iter().enumerate() {
        lines.push(format!(
            "{} | {} | {:.6e} | {:.6e} | {}",
            index + 1,
            result.circuit_string,
            result.chi_square,
            result.bic,
            result.parameter_count
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::{bic, chi_square};

    #[test]
    fn chi_square_is_zero_for_perfect_fit() {
        let z_re = [1.0, 2.0, 3.0];
        let z_im = [-1.0, -2.0, -3.0];

        assert_eq!(chi_square(&z_re, &z_im, &z_re, &z_im), 0.0);
    }

    #[test]
    fn bic_penalizes_parameter_growth() {
        let base = bic(1.0, 3, 50);
        let more_complex = bic(1.0, 6, 50);

        assert!(more_complex > base);
    }
}
