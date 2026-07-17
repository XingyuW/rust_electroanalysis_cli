//! Durable, uncertainty-aware EIS fit artifacts.

use crate::data_file::EISData;
use crate::domain::AnalysisProvenance;
use crate::impedance::{CircuitNode, ElementType};
use crate::results::CircuitFitResult;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EisMeasuredData {
    pub frequency_hz: Vec<f64>,
    pub z_real_ohm: Vec<f64>,
    pub z_imag_ohm: Vec<f64>,
    pub magnitude_ohm: Vec<f64>,
    pub phase_deg: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EisFittedData {
    pub z_real_ohm: Vec<f64>,
    pub z_imag_ohm: Vec<f64>,
    pub magnitude_ohm: Vec<f64>,
    pub phase_deg: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EisFittedParameter {
    pub name: String,
    pub element_id: String,
    pub element_type: String,
    pub semantic_role: Option<String>,
    pub unit: String,
    pub value: f64,
    pub standard_error: Option<f64>,
    pub lower_bound: Option<f64>,
    pub upper_bound: Option<f64>,
    pub at_bound: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EisFitStatistics {
    pub valid_frequency_points: usize,
    pub fitted_parameters: usize,
    pub rss_real: Option<f64>,
    pub rss_imag: Option<f64>,
    pub combined_rss: Option<f64>,
    pub weighted_rss: Option<f64>,
    pub rmse: Option<f64>,
    pub weighted_rmse: Option<f64>,
    pub mae: Option<f64>,
    pub reduced_chi_squared: Option<f64>,
    pub aic: Option<f64>,
    pub aicc: Option<f64>,
    pub bic: Option<f64>,
    pub parameter_covariance: Option<Vec<Vec<f64>>>,
    pub condition_number: Option<f64>,
    pub jacobian_rank: Option<usize>,
    pub convergence_status: String,
    pub optimizer_termination_reason: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EisParameterConfidenceInterval {
    pub parameter: String,
    pub unit: String,
    pub lower: Option<f64>,
    pub upper: Option<f64>,
    pub confidence_level: f64,
    pub method: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EisResidualPoint {
    pub frequency_hz: f64,
    pub real: f64,
    pub imaginary: f64,
    pub magnitude: f64,
    pub phase_deg: f64,
    pub normalized_real: Option<f64>,
    pub normalized_imaginary: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EisResidualData {
    pub points: Vec<EisResidualPoint>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct EisFitDiagnostics {
    pub measured_frequency_min_hz: Option<f64>,
    pub measured_frequency_max_hz: Option<f64>,
    pub parameter_at_bound: Vec<String>,
    pub non_identifiable: bool,
    pub lin_kk_mu: Option<f64>,
    pub lin_kk_terms: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EisFitWarningKind {
    MissingCircuitIdentity,
    SingularCovariance,
    MissingCovariance,
    ParameterAtBound,
    NonIdentifiable,
    PoorResiduals,
    NonFiniteValue,
    InformationCriteriaNotComparable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EisFitWarning {
    pub kind: EisFitWarningKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EisFitArtifact {
    pub schema_version: u32,
    pub fit_id: String,
    pub experiment_id: Option<String>,
    pub sensor_id: Option<String>,
    pub circuit_expression: String,
    pub circuit_canonical_form: String,
    pub measured: EisMeasuredData,
    pub fitted: EisFittedData,
    pub parameters: Vec<EisFittedParameter>,
    pub statistics: EisFitStatistics,
    pub confidence_intervals: Vec<EisParameterConfidenceInterval>,
    pub residuals: EisResidualData,
    pub diagnostics: EisFitDiagnostics,
    pub provenance: AnalysisProvenance,
    pub warnings: Vec<EisFitWarning>,
}

impl EisFitArtifact {
    pub fn from_fit(
        input: &EISData,
        circuit_expression: &str,
        fit: &CircuitFitResult,
        provenance: AnalysisProvenance,
    ) -> Self {
        Self::from_detailed_fit(input, circuit_expression, fit, None, None, None, provenance)
    }

    pub fn from_detailed_fit(
        input: &EISData,
        circuit_expression: &str,
        fit: &CircuitFitResult,
        covariance: Option<Vec<Vec<f64>>>,
        condition_number: Option<f64>,
        jacobian_rank: Option<usize>,
        provenance: AnalysisProvenance,
    ) -> Self {
        let measured = EisMeasuredData {
            frequency_hz: input.freq.clone(),
            z_real_ohm: input.z_re.clone(),
            z_imag_ohm: input.z_im.clone(),
            magnitude_ohm: input
                .z_re
                .iter()
                .zip(&input.z_im)
                .map(|(re, im)| re.hypot(*im))
                .collect(),
            phase_deg: input.phase.clone(),
        };
        let fitted = EisFittedData {
            z_real_ohm: fit.fitted_z_re.clone(),
            z_imag_ohm: fit.fitted_z_im.clone(),
            magnitude_ohm: fit.fitted_magnitude.clone(),
            phase_deg: fit.fitted_phase.clone(),
        };
        let mut residual_points = Vec::new();
        let mut rss_real = 0.0;
        let mut rss_imag = 0.0;
        let mut weighted_rss = 0.0;
        let mut mae = 0.0;
        let n = input
            .freq
            .len()
            .min(fit.fitted_z_re.len())
            .min(fit.fitted_z_im.len());
        for i in 0..n {
            let real = fit.fitted_z_re[i] - input.z_re[i];
            let imaginary = fit.fitted_z_im[i] - input.z_im[i];
            let weight = input.z_re[i].hypot(input.z_im[i]).max(1.0);
            rss_real += real * real;
            rss_imag += imaginary * imaginary;
            weighted_rss += (real / weight).powi(2) + (imaginary / weight).powi(2);
            mae += real.abs() + imaginary.abs();
            residual_points.push(EisResidualPoint {
                frequency_hz: input.freq[i],
                real,
                imaginary,
                magnitude: (fit.fitted_z_re[i].hypot(fit.fitted_z_im[i])
                    - input.z_re[i].hypot(input.z_im[i])),
                phase_deg: fit.fitted_phase.get(i).copied().unwrap_or(0.0)
                    - input.phase.get(i).copied().unwrap_or(0.0),
                normalized_real: Some(real / weight),
                normalized_imaginary: Some(imaginary / weight),
            });
        }
        let combined_rss = rss_real + rss_imag;
        let observations = (2 * n) as f64;
        let parameter_count = fit.fitted_parameters.len() as f64;
        let rmse = (combined_rss / observations.max(1.0)).sqrt();
        let weighted_rmse = (weighted_rss / observations.max(1.0)).sqrt();
        let scale = (weighted_rss / observations.max(1.0)).max(1e-30);
        let aic = observations * scale.ln() + 2.0 * parameter_count;
        let aicc = if observations > parameter_count + 1.0 {
            Some(
                aic + 2.0 * parameter_count * (parameter_count + 1.0)
                    / (observations - parameter_count - 1.0),
            )
        } else {
            None
        };
        let bic = observations * scale.ln() + parameter_count * observations.max(1.0).ln();
        let covariance_is_singular = covariance
            .as_ref()
            .is_some_and(|matrix| matrix.iter().flatten().any(|value| !value.is_finite()));
        let standard_errors = covariance.as_ref().map(|matrix| {
            (0..fit.fitted_parameters.len())
                .map(|i| {
                    matrix
                        .get(i)
                        .and_then(|row| row.get(i))
                        .copied()
                        .map(|v| v.max(0.0).sqrt())
                })
                .collect::<Vec<_>>()
        });
        let bounds = crate::impedance::parse_circuit_string(circuit_expression)
            .ok()
            .map(|circuit| circuit.get_bounds())
            .unwrap_or_default();
        let parameters = fit
            .parameter_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let bound = bounds.get(i).copied();
                let value = fit.fitted_parameters.get(i).copied().unwrap_or(f64::NAN);
                let at_bound = bound.is_some_and(|(lower, upper)| {
                    (value - lower).abs() <= lower.abs().max(1.0) * 1e-8
                        || (value - upper).abs() <= upper.abs().max(1.0) * 1e-8
                });
                EisFittedParameter {
                    name: name.clone(),
                    element_id: element_id_for_name(name),
                    element_type: element_type_for_name(name),
                    semantic_role: None,
                    unit: fit.parameter_units.get(i).cloned().unwrap_or_default(),
                    value,
                    standard_error: standard_errors
                        .as_ref()
                        .and_then(|values| values.get(i).copied())
                        .flatten(),
                    lower_bound: bound.map(|value| value.0),
                    upper_bound: bound.map(|value| value.1),
                    at_bound,
                }
            })
            .collect::<Vec<_>>();
        let mut warnings = Vec::new();
        if covariance.is_none() {
            warnings.push(EisFitWarning {
                kind: EisFitWarningKind::MissingCovariance,
                message: "parameter covariance was not available for this fit".to_string(),
            });
        }
        if covariance_is_singular {
            warnings.push(EisFitWarning {
                kind: EisFitWarningKind::SingularCovariance,
                message: "parameter covariance contained non-finite entries".to_string(),
            });
        }
        if let Some(condition) = condition_number
            && (!condition.is_finite() || condition > 1e12)
        {
            warnings.push(EisFitWarning {
                kind: EisFitWarningKind::NonIdentifiable,
                message: "Jacobian diagnostics indicate a poorly identified fit".to_string(),
            });
        }
        if !circuit_expression.contains('R') {
            warnings.push(EisFitWarning {
                kind: EisFitWarningKind::MissingCircuitIdentity,
                message: "circuit expression does not contain an explicit resistive identity"
                    .to_string(),
            });
        }
        for parameter in &parameters {
            if parameter.at_bound {
                warnings.push(EisFitWarning {
                    kind: EisFitWarningKind::ParameterAtBound,
                    message: format!(
                        "parameter {} is at or within numerical tolerance of a physical bound",
                        parameter.name
                    ),
                });
            }
        }
        let valid_frequency_min = input
            .freq
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .reduce(f64::min);
        let valid_frequency_max = input
            .freq
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > 0.0)
            .reduce(f64::max);
        let statistics = EisFitStatistics {
            valid_frequency_points: n,
            fitted_parameters: fit.fitted_parameters.len(),
            rss_real: finite_option(rss_real),
            rss_imag: finite_option(rss_imag),
            combined_rss: finite_option(combined_rss),
            weighted_rss: finite_option(weighted_rss),
            rmse: finite_option(rmse),
            weighted_rmse: finite_option(weighted_rmse),
            mae: finite_option(mae / observations.max(1.0)),
            reduced_chi_squared: None,
            aic: finite_option(aic),
            aicc,
            bic: finite_option(bic),
            parameter_covariance: covariance,
            condition_number,
            jacobian_rank,
            convergence_status: "converged".to_string(),
            optimizer_termination_reason: None,
        };
        let confidence_intervals = parameters
            .iter()
            .map(|parameter| {
                let se = parameter.standard_error;
                EisParameterConfidenceInterval {
                    parameter: parameter.name.clone(),
                    unit: parameter.unit.clone(),
                    lower: se.and_then(|value| {
                        finite_option(parameter.value - 1.96 * value).map(|candidate| {
                            candidate.max(parameter.lower_bound.unwrap_or(f64::MIN))
                        })
                    }),
                    upper: se.and_then(|value| {
                        finite_option(parameter.value + 1.96 * value).map(|candidate| {
                            candidate.min(parameter.upper_bound.unwrap_or(f64::MAX))
                        })
                    }),
                    confidence_level: 0.95,
                    method: "jacobian_local_normal_approximation".to_string(),
                }
            })
            .collect();
        let parameter_at_bound = parameters
            .iter()
            .filter(|parameter| parameter.at_bound)
            .map(|parameter| parameter.name.clone())
            .collect();
        let _ = PI;
        Self {
            schema_version: 1,
            fit_id: format!("{}:{}", input.label, circuit_expression),
            experiment_id: input.metadata.get("experiment_id").cloned(),
            sensor_id: input.metadata.get("sensor_id").cloned(),
            circuit_expression: circuit_expression.to_string(),
            circuit_canonical_form: circuit_expression.to_string(),
            measured,
            fitted,
            parameters,
            statistics,
            confidence_intervals,
            residuals: EisResidualData {
                points: residual_points,
            },
            diagnostics: EisFitDiagnostics {
                measured_frequency_min_hz: valid_frequency_min,
                measured_frequency_max_hz: valid_frequency_max,
                parameter_at_bound,
                non_identifiable: condition_number.is_some_and(|v| !v.is_finite() || v > 1e12),
                lin_kk_mu: None,
                lin_kk_terms: None,
            },
            provenance,
            warnings,
        }
    }

    pub fn validate_finite(&self) -> bool {
        let text = serde_json::to_string(self).unwrap_or_default();
        !text.contains("NaN") && !text.contains("Infinity")
    }
}

fn finite_option(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

fn element_id_for_name(name: &str) -> String {
    let Some((prefix, suffix)) = name.rsplit_once('_') else {
        return name.to_string();
    };
    format!("{}{}", element_prefix(prefix), suffix)
}

fn element_type_for_name(name: &str) -> String {
    let Some((prefix, _)) = name.rsplit_once('_') else {
        return name
            .chars()
            .take_while(|ch| ch.is_ascii_alphabetic())
            .collect();
    };
    element_prefix(prefix).to_string()
}

fn element_prefix(parameter_prefix: &str) -> &'static str {
    match parameter_prefix {
        "Q" | "alpha" => "CPE",
        "sigma" => "W",
        "R_G" | "t_G" => "G",
        "tau_k" | "gamma" => "Zarc",
        "Z0" => "Wo",
        "R" => "R",
        "C" => "C",
        "L" => "L",
        _ => "element",
    }
}

#[allow(dead_code)]
fn _ast_type_name(node: &CircuitNode) -> &'static str {
    match node {
        CircuitNode::Element(element, _, _) => element.code(),
        CircuitNode::Series(_) => "series",
        CircuitNode::Parallel(_) => "parallel",
    }
}

#[allow(dead_code)]
fn _element_type(_element: ElementType) -> &'static str {
    "element"
}
