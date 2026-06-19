#![allow(clippy::collapsible_if)]

//! High-level equivalent-circuit search facade.
//!
//! This module wraps the lower-level evolutionary search engine and exposes a
//! report-oriented API used by CLI workflows.

use super::ecm_candidate::RANDLES_SEED_CIRCUIT;
use super::ecm_evolution::{EcmEvolutionConfig, run_ecm_evolution};
use super::ecm_scoring::CandidateFitResult;
use super::reporting::format_fitted_circuit_composition;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EcmSearchConfig {
    /// Evolutionary search hyperparameters.
    pub evolution: EcmEvolutionConfig,
    /// Maximum number of ranked candidates preserved in reports.
    pub max_ranked_results: usize,
}

impl Default for EcmSearchConfig {
    fn default() -> Self {
        Self {
            evolution: EcmEvolutionConfig::default(),
            max_ranked_results: 12,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RankedEcmCandidate {
    /// 1-based rank in sorted search results.
    pub rank: usize,
    /// Canonical circuit expression string.
    pub circuit_string: String,
    /// Weighted chi-square objective value.
    pub chi_square: f64,
    /// Bayesian information criterion.
    pub bic: f64,
    /// Weighted RMSE in impedance space.
    pub weighted_rmse: f64,
    /// Number of free parameters in the model.
    pub parameter_count: usize,
    /// Optimized parameter vector.
    pub fitted_parameters: Vec<f64>,
    /// Human-readable parameter names aligned with `fitted_parameters`.
    pub parameter_names: Vec<String>,
    /// Parameter units aligned with `parameter_names`.
    pub parameter_units: Vec<String>,
    /// Fitted real impedance values.
    pub fitted_z_re: Vec<f64>,
    /// Fitted imaginary impedance values.
    pub fitted_z_im: Vec<f64>,
    /// Fitted magnitude values.
    pub fitted_magnitude: Vec<f64>,
    /// Fitted phase values.
    pub fitted_phase: Vec<f64>,
}

impl RankedEcmCandidate {
    fn from_fit(rank: usize, fit: CandidateFitResult) -> Self {
        Self {
            rank,
            circuit_string: fit.circuit_string,
            chi_square: fit.chi_square,
            bic: fit.bic,
            weighted_rmse: fit.weighted_rmse,
            parameter_count: fit.parameter_count,
            fitted_parameters: fit.fitted_parameters,
            parameter_names: fit.parameter_names,
            parameter_units: fit.parameter_units,
            fitted_z_re: fit.fitted_z_re,
            fitted_z_im: fit.fitted_z_im,
            fitted_magnitude: fit.fitted_magnitude,
            fitted_phase: fit.fitted_phase,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EcmSearchReport {
    /// Canonical seed topology used to initialize search populations.
    pub seed_circuit: String,
    /// Number of generations executed.
    pub generations_processed: u64,
    /// Best integer fitness observed during evolution.
    pub best_fitness: i64,
    /// Number of unique candidates evaluated (after cache reuse).
    pub unique_candidates_evaluated: usize,
    /// Ranked candidate list exported to table/CSV/report views.
    pub ranked_candidates: Vec<RankedEcmCandidate>,
}

impl EcmSearchReport {
    /// Render a plain-text markdown-like ranking table.
    pub fn ranking_table(&self) -> String {
        format_ranked_candidates_table(&self.ranked_candidates)
    }

    /// Render CSV ranking output for downstream analysis tools.
    pub fn ranking_csv(&self) -> String {
        format_ranked_candidates_csv(&self.ranked_candidates)
    }

    /// Render run-level summary statistics.
    pub fn summary(&self) -> String {
        format!(
            "Seed Circuit: {}\nGenerations Processed: {}\nBest Fitness: {}\nUnique Candidates Evaluated: {}",
            self.seed_circuit,
            self.generations_processed,
            self.best_fitness,
            self.unique_candidates_evaluated,
        )
    }

    /// Render a full human-readable report including per-candidate details.
    pub fn detailed_report(&self) -> String {
        let mut report = String::new();
        report.push_str(&self.summary());
        report.push_str("\n\n");
        report.push_str(&self.ranking_table());

        if !self.ranked_candidates.is_empty() {
            report.push_str("\n\nCandidate Details\n");
            report.push_str("=================\n");
        }

        for candidate in &self.ranked_candidates {
            report.push_str(&format!(
                "Rank {}: {}\n",
                candidate.rank, candidate.circuit_string
            ));
            report.push_str(&format!("  chi^2: {:.6e}\n", candidate.chi_square));
            report.push_str(&format!("  BIC: {:.6e}\n", candidate.bic));
            report.push_str(&format!(
                "  Weighted RMSE: {:.6e}\n",
                candidate.weighted_rmse
            ));
            report.push_str(&format!(
                "  Parameter Count: {}\n",
                candidate.parameter_count
            ));

            match format_fitted_circuit_composition(
                &candidate.circuit_string,
                &candidate.fitted_parameters,
                "  ",
            ) {
                Ok(composition_report) => {
                    report.push_str(&composition_report);
                    report.push('\n');
                }
                Err(error) => {
                    report.push_str(&format!("  Circuit Composition: unavailable ({error})\n"));
                }
            }

            report.push('\n');
        }

        report.trim_end().to_string()
    }

    /// Export the detailed plain-text report to disk.
    pub fn export_detailed_report<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create report directory {}: {error}",
                        parent.display()
                    )
                })?;
            }
        }

        fs::write(path, self.detailed_report())
            .map_err(|error| format!("failed to write {}: {error}", path.display()))
    }

    /// Export the ranking CSV to disk.
    pub fn export_ranking_csv<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to create report directory {}: {error}",
                        parent.display()
                    )
                })?;
            }
        }

        fs::write(path, self.ranking_csv())
            .map_err(|error| format!("failed to write {}: {error}", path.display()))
    }
}

/// Convenience wrapper that runs search with default configuration.
pub fn discover_equivalent_circuits(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Result<EcmSearchReport, String> {
    discover_equivalent_circuits_with_config(
        frequencies,
        z_real,
        z_imag,
        phase_deg,
        &EcmSearchConfig::default(),
    )
}

/// Run equivalent-circuit discovery using explicit search configuration.
pub fn discover_equivalent_circuits_with_config(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
    config: &EcmSearchConfig,
) -> Result<EcmSearchReport, String> {
    let outcome = run_ecm_evolution(frequencies, z_real, z_imag, phase_deg, &config.evolution)?;

    let limit = config
        .max_ranked_results
        .min(outcome.evaluated_candidates.len());

    let ranked_candidates = outcome
        .evaluated_candidates
        .into_iter()
        .take(limit)
        .enumerate()
        .map(|(index, fit)| RankedEcmCandidate::from_fit(index + 1, fit))
        .collect();

    Ok(EcmSearchReport {
        seed_circuit: RANDLES_SEED_CIRCUIT.to_string(),
        generations_processed: outcome.generations_processed,
        best_fitness: outcome.best_fitness,
        unique_candidates_evaluated: outcome.unique_candidates_evaluated,
        ranked_candidates,
    })
}

/// Format ranked candidates as a compact plain-text table.
pub fn format_ranked_candidates_table(ranked_candidates: &[RankedEcmCandidate]) -> String {
    let mut lines = Vec::with_capacity(ranked_candidates.len() + 2);
    lines.push("Rank | Circuit String | chi^2 | BIC | Parameter Count".to_string());
    lines.push("---- | -------------- | ----- | --- | ---------------".to_string());

    for candidate in ranked_candidates {
        lines.push(format!(
            "{} | {} | {:.6e} | {:.6e} | {}",
            candidate.rank,
            candidate.circuit_string,
            candidate.chi_square,
            candidate.bic,
            candidate.parameter_count,
        ));
    }

    lines.join("\n")
}

/// Format ranked candidates as CSV.
pub fn format_ranked_candidates_csv(ranked_candidates: &[RankedEcmCandidate]) -> String {
    let mut lines = Vec::with_capacity(ranked_candidates.len() + 1);
    lines.push("rank,circuit_string,chi_square,bic,weighted_rmse,parameter_count".to_string());

    for candidate in ranked_candidates {
        lines.push(format!(
            "{},{},{:.6e},{:.6e},{:.6e},{}",
            candidate.rank,
            csv_escape(&candidate.circuit_string),
            candidate.chi_square,
            candidate.bic,
            candidate.weighted_rmse,
            candidate.parameter_count,
        ));
    }

    lines.join("\n")
}

fn csv_escape(value: &str) -> String {
    // RFC4180-style escaping for commas, quotes, and multiline fields.
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EcmSearchConfig, EcmSearchReport, RankedEcmCandidate,
        discover_equivalent_circuits_with_config, format_ranked_candidates_csv,
    };
    use crate::impedance::{
        EcmEvolutionConfig, Impedance, RANDLES_SEED_CIRCUIT, parse_circuit_string,
    };
    use std::f64::consts::PI;

    #[test]
    fn seeded_search_evaluates_randles_data_end_to_end() {
        let circuit = parse_circuit_string(RANDLES_SEED_CIRCUIT).expect("parse synthetic circuit");
        let params = [4.5, 3.2e-5, 0.91, 82.0];
        let frequencies = vec![
            1.0, 2.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1_000.0,
        ];

        let mut z_real = Vec::with_capacity(frequencies.len());
        let mut z_imag = Vec::with_capacity(frequencies.len());
        let mut phase = Vec::with_capacity(frequencies.len());

        for &frequency in &frequencies {
            let impedance = circuit.calculate(2.0 * PI * frequency, &params);
            z_real.push(impedance.re);
            z_imag.push(impedance.im);
            phase.push(impedance.im.atan2(impedance.re).to_degrees());
        }

        let report = discover_equivalent_circuits_with_config(
            &frequencies,
            &z_real,
            &z_imag,
            &phase,
            &EcmSearchConfig {
                evolution: EcmEvolutionConfig {
                    population_size: 8,
                    generation_limit: 1,
                    mutation_rate: 0.25,
                    ..EcmEvolutionConfig::default()
                },
                max_ranked_results: 6,
            },
        )
        .expect("run equivalent circuit search");

        assert!(!report.ranked_candidates.is_empty());
        assert!(
            report
                .ranked_candidates
                .iter()
                .any(|candidate| candidate.circuit_string == RANDLES_SEED_CIRCUIT),
            "expected ranked candidates to include canonical Randles seed `{RANDLES_SEED_CIRCUIT}`"
        );
    }

    #[test]
    fn ranked_candidates_csv_escapes_circuit_strings() {
        let csv = format_ranked_candidates_csv(&[RankedEcmCandidate {
            rank: 1,
            circuit_string: "R0-p(CPE1,R1)".to_string(),
            chi_square: 1.0,
            bic: 2.0,
            weighted_rmse: 3.0,
            parameter_count: 4,
            fitted_parameters: Vec::new(),
            parameter_names: Vec::new(),
            parameter_units: Vec::new(),
            fitted_z_re: Vec::new(),
            fitted_z_im: Vec::new(),
            fitted_magnitude: Vec::new(),
            fitted_phase: Vec::new(),
        }]);

        assert!(csv.contains("\"R0-p(CPE1,R1)\""));
    }

    #[test]
    fn detailed_report_includes_element_counts_and_breakdown() {
        let report = EcmSearchReport {
            seed_circuit: RANDLES_SEED_CIRCUIT.to_string(),
            generations_processed: 3,
            best_fitness: 42,
            unique_candidates_evaluated: 5,
            ranked_candidates: vec![RankedEcmCandidate {
                rank: 1,
                circuit_string: "R0-p(CPE1,R1)".to_string(),
                chi_square: 1.0,
                bic: 2.0,
                weighted_rmse: 3.0,
                parameter_count: 4,
                fitted_parameters: vec![4.5, 3.2e-5, 0.91, 82.0],
                parameter_names: vec![
                    "R_0".to_string(),
                    "Q_1".to_string(),
                    "alpha_1".to_string(),
                    "R_1".to_string(),
                ],
                parameter_units: vec![
                    "Ohm".to_string(),
                    "Ohm^-1 s^alpha".to_string(),
                    "".to_string(),
                    "Ohm".to_string(),
                ],
                fitted_z_re: Vec::new(),
                fitted_z_im: Vec::new(),
                fitted_magnitude: Vec::new(),
                fitted_phase: Vec::new(),
            }],
        };

        let detailed = report.detailed_report();

        assert!(detailed.contains("Element Counts:"));
        assert!(detailed.contains("R (Resistor) = 2"));
        assert!(detailed.contains("CPE (Constant Phase Element) = 1"));
        assert!(detailed.contains("- R0 [R: Resistor]"));
        assert!(detailed.contains("- CPE1 [CPE: Constant Phase Element]"));
    }
}
