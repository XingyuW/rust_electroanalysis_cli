use crate::{
    results::{
        CalibrationAnalysisReport, EisFitArtifact, HealthDomain, HealthFeature,
        MechanismAnalysisReport, SignalAnalysisReport, TransientAnalysisReport,
    },
    signal::statistics,
};
use std::collections::BTreeMap;

/// Workflow-neutral model outputs used for longitudinal health comparison.
/// Component IDs are durable identifiers from `ModelDefinition`, never a
/// position in a fitted parameter vector.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ModelHealthSnapshot {
    pub component_parameter_values: BTreeMap<(String, String), f64>,
    pub component_state_values: BTreeMap<(String, String), f64>,
    pub equilibrium_recognition_fraction: Option<f64>,
    pub unexplained_residual_rms_v: Option<f64>,
    pub component_validity_failures: BTreeMap<String, usize>,
    pub component_identifiability_scores: BTreeMap<String, f64>,
    pub calibration_domain_excursions: usize,
}

pub fn from_signal(r: &SignalAnalysisReport) -> Vec<HealthFeature> {
    let mut f = Vec::new();
    let add =
        |v: &mut Vec<HealthFeature>, name: &str, value: Option<f64>, unit: &str, source: &str| {
            v.push(HealthFeature {
                name: name.into(),
                value,
                unit: unit.into(),
                domain: HealthDomain::SignalNoise,
                source: source.into(),
                warning: None,
            })
        };
    add(
        &mut f,
        "signal.rms_noise",
        r.descriptive.rms,
        &r.unit,
        "signal",
    );
    add(
        &mut f,
        "signal.robust_noise_standard_deviation",
        r.descriptive.robust_standard_deviation,
        &r.unit,
        "signal",
    );
    add(
        &mut f,
        "signal.peak_to_peak",
        r.descriptive.peak_to_peak,
        &r.unit,
        "signal",
    );
    add(
        &mut f,
        "signal.allan_minimum",
        r.allan.as_ref().and_then(|a| a.minimum_deviation),
        &r.unit,
        "signal",
    );
    add(
        &mut f,
        "signal.allan_minimum_averaging_time",
        r.allan.as_ref().and_then(|a| a.minimum_averaging_time_s),
        "s",
        "signal",
    );
    f.push(HealthFeature {
        name: "signal.robust_drift_rate".into(),
        value: r
            .drift
            .iter()
            .find(|d| matches!(d.model, crate::results::DriftModelKind::TheilSen))
            .and_then(|d| d.slope_v_per_s),
        unit: format!("{}/s", r.unit),
        domain: HealthDomain::Drift,
        source: "signal".into(),
        warning: None,
    });
    add(
        &mut f,
        "signal.spike_fraction",
        r.spikes.flagged_fraction,
        "fraction",
        "signal",
    );
    add(
        &mut f,
        "signal.missing_fraction",
        r.sampling.missing_fraction,
        "fraction",
        "signal",
    );
    add(
        &mut f,
        "signal.sampling_irregularity",
        r.sampling.interval_cv,
        "fraction",
        "signal",
    );
    add(
        &mut f,
        "signal.common_mode_fraction",
        r.correlations.first().and_then(|c| c.common_mode_fraction),
        "fraction",
        "signal",
    );
    if let Some(psd) = &r.psd {
        for b in &psd.band_powers {
            add(
                &mut f,
                &format!("signal.psd_band_power.{}", b.name),
                b.integrated_power,
                &psd.psd_unit,
                "signal",
            );
        }
        add(
            &mut f,
            "signal.dominant_peak_hz",
            psd.dominant_peaks.first().map(|p| p.frequency_hz),
            "Hz",
            "signal",
        );
    }
    f
}
pub fn from_transient(r: &TransientAnalysisReport) -> Vec<HealthFeature> {
    let mut f = Vec::new();
    let mut groups = BTreeMap::<String, Vec<&crate::results::TransientFitResult>>::new();
    for event in &r.events {
        let Some(fit) = event.selected_model.and_then(|model| {
            event
                .candidate_fits
                .iter()
                .find(|fit| fit.model == model && fit.is_successful())
        }) else {
            continue;
        };
        groups
            .entry(transient_context_key(event))
            .or_default()
            .push(fit);
    }
    let single_context = groups.len() == 1;
    let add = |v: &mut Vec<HealthFeature>, n: String, x: Option<f64>, u: &str| {
        let domain = if n.ends_with("drift_rate") {
            HealthDomain::Drift
        } else {
            HealthDomain::DynamicResponse
        };
        v.push(HealthFeature {
            name: n,
            value: x,
            unit: u.into(),
            domain,
            source: "transient".into(),
            warning: None,
        })
    };
    for (context, selected) in groups {
        let average = |field: fn(&crate::results::TransientFeatures) -> Option<f64>| {
            statistics::mean(
                &selected
                    .iter()
                    .filter_map(|fit| field(&fit.derived_features))
                    .collect::<Vec<_>>(),
            )
        };
        let name = |field: &str| format!("transient.{context}.{field}");
        add(&mut f, name("tau_fast"), average(|x| x.tau_fast_s), "s");
        add(&mut f, name("tau_slow"), average(|x| x.tau_slow_s), "s");
        add(
            &mut f,
            name("response_amplitude"),
            average(|x| x.total_response_amplitude_v),
            "V",
        );
        add(
            &mut f,
            name("fast_amplitude"),
            average(|x| x.fast_amplitude_v),
            "V",
        );
        add(
            &mut f,
            name("slow_amplitude"),
            average(|x| x.slow_amplitude_v),
            "V",
        );
        add(
            &mut f,
            name("initial_response_rate"),
            average(|x| x.initial_response_rate_v_per_s),
            "V/s",
        );
        add(
            &mut f,
            name("time_to_90_percent"),
            average(|x| x.time_to_90_percent_s),
            "s",
        );
        add(
            &mut f,
            name("time_to_95_percent"),
            average(|x| x.time_to_95_percent_s),
            "s",
        );
        add(
            &mut f,
            name("drift_rate"),
            average(|x| x.drift_rate_v_per_s),
            "V/s",
        );
        add(
            &mut f,
            name("fit_rmse"),
            statistics::mean(
                &selected
                    .iter()
                    .filter_map(|fit| fit.statistics.rmse_v)
                    .collect::<Vec<_>>(),
            ),
            "V",
        );
        add(
            &mut f,
            name("residual_autocorrelation"),
            statistics::mean(
                &selected
                    .iter()
                    .filter_map(|fit| fit.statistics.lag1_residual_autocorrelation)
                    .collect::<Vec<_>>(),
            ),
            "fraction",
        );
    }
    // Preserve legacy feature names only when every event belongs to exactly
    // one scientifically comparable context. With multiple contexts there is
    // deliberately no aggregate alias to prevent cross-context averaging.
    if single_context {
        let aliases = f
            .iter()
            .filter_map(|feature| {
                feature
                    .name
                    .rsplit_once('.')
                    .map(|(_, field)| HealthFeature {
                        name: format!("transient.{field}"),
                        value: feature.value,
                        unit: feature.unit.clone(),
                        domain: feature.domain,
                        source: feature.source.clone(),
                        warning: feature.warning.clone(),
                    })
            })
            .collect::<Vec<_>>();
        f.extend(aliases);
    }
    f
}

fn transient_context_key(event: &crate::results::TransientEventResult) -> String {
    let metadata = event.event.metadata.as_ref();
    let get = |key: &str| {
        metadata
            .and_then(|values| values.get(key))
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    };
    let before = event
        .concentration_before
        .as_ref()
        .map(|value| value.value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let after = event
        .concentration_after
        .as_ref()
        .map(|value| value.value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let direction = match (
        event.concentration_before.as_ref().map(|x| x.value),
        event.concentration_after.as_ref().map(|x| x.value),
    ) {
        (Some(before), Some(after)) if after > before => "increasing",
        (Some(before), Some(after)) if after < before => "decreasing",
        _ => "unknown",
    };
    format!(
        "analyte={};step={before}->{after};direction={direction};matrix={};temperature_k={}",
        event
            .event
            .analyte
            .clone()
            .unwrap_or_else(|| "unknown".to_string()),
        get("sample_matrix"),
        get("temperature_k")
    )
}

pub fn from_model(snapshot: &ModelHealthSnapshot) -> Vec<HealthFeature> {
    let mut features = Vec::new();
    let mut add = |name: String,
                   value: Option<f64>,
                   unit: &str,
                   domain: HealthDomain,
                   warning: Option<String>| {
        features.push(HealthFeature {
            name,
            value,
            unit: unit.to_string(),
            domain,
            source: "model".to_string(),
            warning,
        });
    };
    for ((component_id, parameter_id), value) in &snapshot.component_parameter_values {
        add(
            format!("model.component.{component_id}.parameter.{parameter_id}.drift_value"),
            value.is_finite().then_some(*value),
            "model parameter",
            HealthDomain::Drift,
            (!value.is_finite()).then_some("non-finite component parameter".to_string()),
        );
    }
    for ((component_id, state_id), value) in &snapshot.component_state_values {
        add(
            format!("model.component.{component_id}.state.{state_id}.drift_value"),
            value.is_finite().then_some(*value),
            "model state",
            HealthDomain::Drift,
            (!value.is_finite()).then_some("non-finite component state".to_string()),
        );
    }
    add(
        "model.equilibrium_recognition_fraction".to_string(),
        snapshot
            .equilibrium_recognition_fraction
            .filter(|x| x.is_finite()),
        "fraction",
        HealthDomain::DynamicResponse,
        None,
    );
    add(
        "model.unexplained_residual_rms".to_string(),
        snapshot
            .unexplained_residual_rms_v
            .filter(|x| x.is_finite()),
        "V",
        HealthDomain::SignalNoise,
        None,
    );
    for (component_id, failures) in &snapshot.component_validity_failures {
        add(
            format!("model.component.{component_id}.validity_failures"),
            Some(*failures as f64),
            "count",
            HealthDomain::DataQuality,
            (*failures > 0).then_some("component validity failures present".to_string()),
        );
    }
    for (component_id, score) in &snapshot.component_identifiability_scores {
        add(
            format!("model.component.{component_id}.identifiability"),
            score.is_finite().then_some(*score),
            "score",
            HealthDomain::MechanismEvidence,
            (!score.is_finite()).then_some("identifiability unavailable".to_string()),
        );
    }
    add(
        "model.calibration_domain_excursions".to_string(),
        Some(snapshot.calibration_domain_excursions as f64),
        "count",
        HealthDomain::Calibration,
        (snapshot.calibration_domain_excursions > 0)
            .then_some("calibration-domain excursions present".to_string()),
    );
    features
}
pub fn from_calibration(r: &CalibrationAnalysisReport) -> Vec<HealthFeature> {
    let mut f = Vec::new();
    let m = r
        .selected_model
        .and_then(|k| {
            r.candidate_models.iter().find(|m| {
                m.model_kind == k
                    && matches!(m.status, crate::results::CalibrationFitStatus::Converged)
            })
        })
        .or_else(|| {
            r.candidate_models
                .iter()
                .find(|m| matches!(m.status, crate::results::CalibrationFitStatus::Converged))
        });
    let add = |v: &mut Vec<HealthFeature>, n: &str, x: Option<f64>, u: &str| {
        v.push(HealthFeature {
            name: n.into(),
            value: x,
            unit: u.into(),
            domain: HealthDomain::Calibration,
            source: "calibration".into(),
            warning: None,
        })
    };
    if let Some(m) = m {
        add(
            &mut f,
            "calibration.slope",
            m.fitted_slope_v_per_decade,
            "V/decade",
        );
        add(
            &mut f,
            "calibration.theoretical_slope",
            m.theoretical_slope_v_per_decade,
            "V/decade",
        );
        add(
            &mut f,
            "calibration.slope_efficiency",
            m.slope_efficiency,
            "fraction",
        );
        add(&mut f, "calibration.rmse", m.statistics.rmse_v, "V");
        add(
            &mut f,
            "calibration.condition_number",
            m.statistics.condition_number,
            "condition number",
        );
        add(
            &mut f,
            "calibration.influential_observation_count",
            Some(
                m.statistics
                    .cooks_distance
                    .iter()
                    .filter(|x| **x > 1.0)
                    .count() as f64,
            ),
            "count",
        );
    }
    add(
        &mut f,
        "calibration.hysteresis",
        r.hysteresis.as_ref().and_then(|h| h.mean_hysteresis_v),
        "V",
    );
    add(
        &mut f,
        "calibration.cross_validation_rmse",
        r.validation.as_ref().and_then(|v| v.rmse_potential_v),
        "V",
    );
    add(
        &mut f,
        "calibration.prediction_bias",
        r.validation.as_ref().and_then(|v| v.prediction_bias_v),
        "V",
    );
    f
}
pub fn from_eis(r: &EisFitArtifact) -> Vec<HealthFeature> {
    let mut f = Vec::new();
    let add = |v: &mut Vec<HealthFeature>, n: &str, x: Option<f64>, u: &str| {
        v.push(HealthFeature {
            name: n.into(),
            value: x,
            unit: u.into(),
            domain: HealthDomain::Impedance,
            source: "eis".into(),
            warning: None,
        })
    };
    add(&mut f, "eis.fit_rmse", r.statistics.rmse, "ohm");
    add(
        &mut f,
        "eis.weighted_rmse",
        r.statistics.weighted_rmse,
        "fraction",
    );
    add(
        &mut f,
        "eis.condition_number",
        r.statistics.condition_number,
        "condition number",
    );
    add(
        &mut f,
        "eis.jacobian_rank",
        r.statistics.jacobian_rank.map(|v| v as f64),
        "rank",
    );
    add(
        &mut f,
        "eis.parameters_at_bounds",
        Some(r.parameters.iter().filter(|p| p.at_bound).count() as f64),
        "count",
    );
    for p in &r.parameters {
        if let Some(role) = &p.semantic_role {
            add(
                &mut f,
                &format!("eis.role.{}.{}", role, p.name),
                Some(p.value),
                &p.unit,
            );
        }
    }
    f
}
pub fn from_mechanism(r: &MechanismAnalysisReport) -> Vec<HealthFeature> {
    let mut f = Vec::new();
    let add = |v: &mut Vec<HealthFeature>, n: &str, x: Option<f64>| {
        v.push(HealthFeature {
            name: n.into(),
            value: x,
            unit: "fraction".into(),
            domain: HealthDomain::MechanismEvidence,
            source: "mechanism".into(),
            warning: None,
        })
    };
    let ratios = r
        .comparisons
        .iter()
        .filter_map(|c| c.ratio)
        .collect::<Vec<_>>();
    add(
        &mut f,
        "mechanism.timescale_ratio",
        statistics::mean(&ratios),
    );
    add(
        &mut f,
        "mechanism.strong_comparisons",
        Some(
            r.comparisons
                .iter()
                .filter(|c| matches!(c.evidence_level, crate::results::EvidenceLevel::Strong))
                .count() as f64,
        ),
    );
    add(
        &mut f,
        "mechanism.contradictory_comparisons",
        Some(
            r.comparisons
                .iter()
                .filter(|c| {
                    matches!(
                        c.evidence_level,
                        crate::results::EvidenceLevel::Contradictory
                    )
                })
                .count() as f64,
        ),
    );
    f
}
