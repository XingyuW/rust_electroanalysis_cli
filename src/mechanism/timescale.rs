//! Topology-aware characteristic-timescale derivation.

use super::uncertainty::{confidence_interval, delta_variance};
use crate::impedance::{CircuitNode, ElementType};
use crate::results::{
    CharacteristicTimescale, EisFitArtifact, MechanismWarning, TimescaleDerivation,
    TimescaleSource, TimescaleValidity,
};
use std::f64::consts::PI;

pub fn extract_eis_timescales(
    artifact: &EisFitArtifact,
    confidence_level: f64,
    boundary_margin: f64,
) -> Vec<CharacteristicTimescale> {
    let Ok(circuit) = crate::impedance::parse_circuit_string(&artifact.circuit_expression) else {
        return vec![unavailable(
            "circuit",
            "circuit expression could not be parsed",
        )];
    };
    let names = circuit.get_param_names();
    let values = artifact
        .parameters
        .iter()
        .map(|p| p.value)
        .collect::<Vec<_>>();
    let covariance = artifact.statistics.parameter_covariance.as_deref();
    let mut output = Vec::new();
    walk(
        &circuit,
        "root",
        &names,
        &values,
        covariance,
        artifact,
        confidence_level,
        boundary_margin,
        &mut output,
    );
    output
}

#[allow(clippy::too_many_arguments)]
fn walk(
    node: &CircuitNode,
    path: &str,
    names: &[String],
    values: &[f64],
    covariance: Option<&[Vec<f64>]>,
    artifact: &EisFitArtifact,
    confidence_level: f64,
    boundary_margin: f64,
    output: &mut Vec<CharacteristicTimescale>,
) {
    match node {
        CircuitNode::Element(element, index, _label) => {
            let direct = match element {
                ElementType::Wo
                | ElementType::Ws
                | ElementType::G
                | ElementType::Gs
                | ElementType::K
                | ElementType::Zarc => Some(1),
                _ => None,
            };
            if let Some(offset) = direct {
                let parameter_index = *index + offset;
                if let Some(&value) = values
                    .get(parameter_index)
                    .filter(|v| v.is_finite() && **v > 0.0)
                {
                    let se = covariance
                        .and_then(|matrix| {
                            matrix
                                .get(parameter_index)
                                .and_then(|row| row.get(parameter_index))
                        })
                        .copied()
                        .filter(|v| v.is_finite() && *v >= 0.0)
                        .map(f64::sqrt);
                    let warnings = frequency_warnings(value, artifact, boundary_margin);
                    let validity = if warnings.is_empty() {
                        TimescaleValidity::Valid
                    } else {
                        TimescaleValidity::ValidWithWarnings
                    };
                    output.push(make_timescale(
                        format!("{path}.element"),
                        format!("{} direct fitted timescale", element.code()),
                        value,
                        se,
                        confidence_level,
                        vec![
                            names
                                .get(parameter_index)
                                .cloned()
                                .unwrap_or_else(|| format!("parameter_{parameter_index}")),
                        ],
                        format!(
                            "tau = {} parameter",
                            element.param_names().get(offset).copied().unwrap_or("tau")
                        ),
                        Some(path.to_string()),
                        None,
                        validity,
                        warnings,
                    ));
                }
            }
        }
        CircuitNode::Series(nodes) => {
            for (i, child) in nodes.iter().enumerate() {
                walk(
                    child,
                    &format!("{path}.series[{i}]"),
                    names,
                    values,
                    covariance,
                    artifact,
                    confidence_level,
                    boundary_margin,
                    output,
                );
            }
        }
        CircuitNode::Parallel(nodes) => {
            let leaves = nodes
                .iter()
                .enumerate()
                .filter_map(|(i, child)| match child {
                    CircuitNode::Element(element, index, label) => {
                        Some((i, *element, *index, label.clone()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            for i in 0..leaves.len() {
                for j in (i + 1)..leaves.len() {
                    let ((ri, re, rix, rlabel), (ci, ce, cix, clabel)) =
                        (leaves[i].clone(), leaves[j].clone());
                    let pair = match (re, ce) {
                        (ElementType::R, ElementType::C) => Some(false),
                        (ElementType::R, ElementType::Cpe) => Some(true),
                        (ElementType::C, ElementType::R) => Some(false),
                        (ElementType::Cpe, ElementType::R) => Some(true),
                        _ => None,
                    };
                    let Some(is_cpe) = pair else {
                        continue;
                    };
                    let (r_index, c_index) = if re == ElementType::R {
                        (rix, cix)
                    } else {
                        (cix, rix)
                    };
                    let r = values.get(r_index).copied();
                    let c = values.get(c_index).copied();
                    let (Some(r), Some(c)) = (r, c) else {
                        continue;
                    };
                    if !r.is_finite() || !c.is_finite() || r <= 0.0 || c <= 0.0 {
                        continue;
                    }
                    let (value, gradient, equation, convention, params) = if is_cpe {
                        let alpha_index = c_index + 1;
                        let alpha = values.get(alpha_index).copied().unwrap_or(f64::NAN);
                        if !alpha.is_finite() || alpha <= 0.0 || alpha > 1.0 {
                            continue;
                        }
                        let value = (r * c).powf(1.0 / alpha);
                        let mut gradient = vec![0.0; values.len()];
                        gradient[r_index] = value / (alpha * r);
                        gradient[c_index] = value / (alpha * c);
                        gradient[alpha_index] = -value * (r * c).ln() / alpha.powi(2);
                        (
                            value,
                            gradient,
                            "tau_c = (R*Q)^(1/alpha)".to_string(),
                            Some("Z_CPE = 1/(Q*(jω)^alpha)".to_string()),
                            vec![
                                names.get(r_index).cloned().unwrap_or_default(),
                                names.get(c_index).cloned().unwrap_or_default(),
                                names.get(alpha_index).cloned().unwrap_or_default(),
                            ],
                        )
                    } else {
                        let value = r * c;
                        let mut gradient = vec![0.0; values.len()];
                        gradient[r_index] = c;
                        gradient[c_index] = r;
                        (
                            value,
                            gradient,
                            "tau = R*C".to_string(),
                            Some("ideal parallel R-C branch".to_string()),
                            vec![
                                names.get(r_index).cloned().unwrap_or_default(),
                                names.get(c_index).cloned().unwrap_or_default(),
                            ],
                        )
                    };
                    if !value.is_finite() || value <= 0.0 {
                        continue;
                    }
                    let se = delta_variance(&gradient, covariance).map(f64::sqrt);
                    let mut warnings = frequency_warnings(value, artifact, boundary_margin);
                    if covariance.is_none() {
                        warnings.push(MechanismWarning {
                            kind: "missing_covariance".to_string(),
                            message: "uncertainty propagation could not use parameter covariance"
                                .to_string(),
                        });
                    }
                    let label =
                        format!("parallel R-{} relaxation", if is_cpe { "CPE" } else { "C" });
                    output.push(make_timescale(
                        format!("{path}.parallel[{ri}].branch[{ci}]"),
                        label,
                        value,
                        se,
                        confidence_level,
                        params,
                        equation,
                        Some(format!("{path}.parallel[{ri}] + {path}.parallel[{ci}]")),
                        convention,
                        if warnings.is_empty() {
                            TimescaleValidity::Valid
                        } else {
                            TimescaleValidity::ValidWithWarnings
                        },
                        warnings,
                    ));
                    let _ = (rlabel, clabel);
                }
            }
            for (i, child) in nodes.iter().enumerate() {
                walk(
                    child,
                    &format!("{path}.parallel[{i}]"),
                    names,
                    values,
                    covariance,
                    artifact,
                    confidence_level,
                    boundary_margin,
                    output,
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn make_timescale(
    id: String,
    label: String,
    value: f64,
    se: Option<f64>,
    level: f64,
    source_parameters: Vec<String>,
    equation: String,
    circuit_path: Option<String>,
    convention: Option<String>,
    validity: TimescaleValidity,
    warnings: Vec<MechanismWarning>,
) -> CharacteristicTimescale {
    CharacteristicTimescale {
        timescale_id: id,
        source: TimescaleSource::EisCircuit,
        label,
        value_s: value,
        standard_error_s: se,
        confidence_interval_s: confidence_interval(value, se, level),
        derivation: TimescaleDerivation {
            equation,
            circuit_path,
            convention,
        },
        source_parameters,
        semantic_role: None,
        validity,
        warnings,
    }
}

fn frequency_warnings(
    value: f64,
    artifact: &EisFitArtifact,
    boundary_margin: f64,
) -> Vec<MechanismWarning> {
    let fc = 1.0 / (2.0 * PI * value);
    let min = artifact.diagnostics.measured_frequency_min_hz;
    let max = artifact.diagnostics.measured_frequency_max_hz;
    let mut warnings = Vec::new();
    if let (Some(min), Some(max)) = (min, max) {
        if fc < min || fc > max {
            warnings.push(MechanismWarning {
                kind: "timescale_outside_measured_frequency_window".to_string(),
                message: format!(
                    "characteristic frequency {fc:.6e} Hz is outside [{min:.6e}, {max:.6e}] Hz"
                ),
            });
        } else if (fc - min) / (max - min).max(f64::MIN_POSITIVE) < boundary_margin
            || (max - fc) / (max - min).max(f64::MIN_POSITIVE) < boundary_margin
        {
            warnings.push(MechanismWarning {
                kind: "frequency_boundary".to_string(),
                message: "characteristic frequency is close to the measured frequency boundary"
                    .to_string(),
            });
        }
    }
    warnings
}

fn unavailable(id: &str, message: &str) -> CharacteristicTimescale {
    CharacteristicTimescale {
        timescale_id: id.to_string(),
        source: TimescaleSource::EisCircuit,
        label: "unavailable EIS timescale".to_string(),
        value_s: 0.0,
        standard_error_s: None,
        confidence_interval_s: None,
        derivation: TimescaleDerivation {
            equation: "not available".to_string(),
            circuit_path: None,
            convention: None,
        },
        source_parameters: Vec::new(),
        semantic_role: None,
        validity: TimescaleValidity::Unavailable,
        warnings: vec![MechanismWarning {
            kind: "no_timescale_candidates".to_string(),
            message: message.to_string(),
        }],
    }
}

pub fn extract_transient_timescales(
    report: &crate::results::TransientAnalysisReport,
    allow_warning_fits: bool,
    confidence_level: f64,
) -> Vec<CharacteristicTimescale> {
    let mut output = Vec::new();
    for event in &report.events {
        let Some(selected) = event.selected_model else {
            continue;
        };
        let Some(fit) = event.candidate_fits.iter().find(|fit| {
            fit.model == selected
                && fit.is_successful()
                && (allow_warning_fits || fit.warnings.is_empty())
        }) else {
            continue;
        };
        let features = &fit.derived_features;
        let candidates = match fit.model {
            crate::potentiometry::transient::models::TransientModelKind::Double
            | crate::potentiometry::transient::models::TransientModelKind::DoubleDrift => vec![
                ("tau_fast", features.tau_fast_s),
                ("tau_slow", features.tau_slow_s),
            ],
            _ => vec![("single fitted transient timescale", features.tau_fast_s)],
        };
        for (label, value) in candidates {
            let Some(value) = value.filter(|v| v.is_finite() && *v > 0.0) else {
                continue;
            };
            let interval = fit
                .confidence_intervals
                .iter()
                .find(|ci| ci.name == label || ci.name.contains(label))
                .and_then(|ci| ci.lower.zip(ci.upper));
            let se = interval.map(|(lo, hi)| (hi - lo) / (2.0 * 1.96));
            let mut warnings = Vec::new();
            if event.failure.is_some() {
                warnings.push(MechanismWarning {
                    kind: "transient_fit_unavailable".to_string(),
                    message: "event fit was not available".to_string(),
                });
            }
            if let Some(duration) = event.segment.finite_duration_s
                && value > duration
            {
                warnings.push(MechanismWarning {
                    kind: "transient_timescale_outside_observation_window".to_string(),
                    message: "transient timescale exceeds the finite observation duration"
                        .to_string(),
                });
            }
            output.push(CharacteristicTimescale {
                timescale_id: format!("event_{}_{}", event.event_index, label.replace(' ', "_")),
                source: TimescaleSource::TransientFit,
                label: label.to_string(),
                value_s: value,
                standard_error_s: se,
                confidence_interval_s: interval
                    .or_else(|| confidence_interval(value, se, confidence_level)),
                derivation: TimescaleDerivation {
                    equation: format!("selected {:?} model parameter", fit.model),
                    circuit_path: None,
                    convention: None,
                },
                source_parameters: fit
                    .parameters
                    .iter()
                    .filter(|p| p.name.to_ascii_lowercase().contains("tau"))
                    .map(|p| p.name.clone())
                    .collect(),
                semantic_role: None,
                validity: if warnings.is_empty() {
                    TimescaleValidity::Valid
                } else {
                    TimescaleValidity::ValidWithWarnings
                },
                warnings,
            });
        }
    }
    output
}
