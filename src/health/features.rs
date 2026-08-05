use crate::{
    results::{
        CalibrationAnalysisReport, EisFitArtifact, HealthDomain, HealthFeature,
        MechanismAnalysisReport, SignalAnalysisReport, TransientAnalysisReport,
    },
    signal::statistics,
};

pub fn from_signal(r: &SignalAnalysisReport) -> Vec<HealthFeature> {
    let mut f = Vec::new();
    let add = |v: &mut Vec<HealthFeature>,
               name: &str,
               value: Option<f64>,
               unit: &str,
               source: &str,
               domain: HealthDomain| {
        v.push(HealthFeature {
            name: name.into(),
            value,
            unit: unit.into(),
            domain,
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
        HealthDomain::SignalNoise,
    );
    add(
        &mut f,
        "signal.robust_noise_standard_deviation",
        r.descriptive.robust_standard_deviation,
        &r.unit,
        "signal",
        HealthDomain::SignalNoise,
    );
    add(
        &mut f,
        "signal.peak_to_peak",
        r.descriptive.peak_to_peak,
        &r.unit,
        "signal",
        HealthDomain::SignalNoise,
    );
    add(
        &mut f,
        "signal.allan_minimum",
        r.allan.as_ref().and_then(|a| a.minimum_deviation),
        &r.unit,
        "signal",
        HealthDomain::SignalNoise,
    );
    add(
        &mut f,
        "signal.allan_minimum_averaging_time",
        r.allan.as_ref().and_then(|a| a.minimum_averaging_time_s),
        "s",
        "signal",
        HealthDomain::SignalNoise,
    );
    add(
        &mut f,
        "signal.robust_drift_rate",
        r.drift
            .iter()
            .find(|d| matches!(d.model, crate::results::DriftModelKind::TheilSen))
            .and_then(|d| d.slope_v_per_s),
        format!("{}/s", r.unit).as_str(),
        "signal",
        HealthDomain::Drift,
    );
    add(
        &mut f,
        "signal.spike_fraction",
        r.spikes.flagged_fraction,
        "fraction",
        "signal",
        HealthDomain::SignalNoise,
    );
    add(
        &mut f,
        "signal.missing_fraction",
        r.sampling.missing_fraction,
        "fraction",
        "signal",
        HealthDomain::DataQuality,
    );
    add(
        &mut f,
        "signal.sampling_irregularity",
        r.sampling.interval_cv,
        "fraction",
        "signal",
        HealthDomain::DataQuality,
    );
    add(
        &mut f,
        "signal.common_mode_fraction",
        r.correlations.first().and_then(|c| c.common_mode_fraction),
        "fraction",
        "signal",
        HealthDomain::SignalNoise,
    );
    if let Some(psd) = &r.psd {
        for b in &psd.band_powers {
            add(
                &mut f,
                &format!("signal.psd_band_power.{}", b.name),
                b.integrated_power,
                &psd.psd_unit,
                "signal",
                HealthDomain::SignalNoise,
            );
        }
        add(
            &mut f,
            "signal.dominant_peak_hz",
            psd.dominant_peaks.first().map(|p| p.frequency_hz),
            "Hz",
            "signal",
            HealthDomain::SignalNoise,
        );
    }
    f
}
pub fn from_transient(r: &TransientAnalysisReport) -> Vec<HealthFeature> {
    let selected = r
        .events
        .iter()
        .enumerate()
        .filter_map(|(event_index, e)| {
            e.selected_model.and_then(|m| {
                e.candidate_fits
                    .iter()
                    .find(|x| x.model == m && x.is_successful())
                    .map(|fit| (event_index, e, fit))
            })
        })
        .collect::<Vec<_>>();
    if selected.len() <= 1 {
        return transient_feature_group("transient", &selected);
    }
    let mut groups = std::collections::BTreeMap::<String, Vec<_>>::new();
    for item in selected {
        groups
            .entry(transient_context_key(item.0, item.1))
            .or_default()
            .push(item);
    }
    groups
        .into_iter()
        .flat_map(|(context, group)| {
            transient_feature_group(&format!("transient[{context}]"), &group)
        })
        .collect()
}

fn transient_feature_group(
    prefix: &str,
    selected: &[(
        usize,
        &crate::results::TransientEventResult,
        &crate::results::TransientFitResult,
    )],
) -> Vec<HealthFeature> {
    let mut f = Vec::new();
    let avg = |name: fn(&crate::results::TransientFeatures) -> Option<f64>| {
        statistics::mean(
            &selected
                .iter()
                .filter_map(|(_, _, fit)| name(&fit.derived_features))
                .collect::<Vec<_>>(),
        )
    };
    let add = |v: &mut Vec<HealthFeature>, n: &str, x: Option<f64>, u: &str| {
        v.push(HealthFeature {
            name: n.into(),
            value: x,
            unit: u.into(),
            domain: HealthDomain::DynamicResponse,
            source: prefix.into(),
            warning: None,
        })
    };
    add(
        &mut f,
        &format!("{prefix}.tau_fast"),
        avg(|x| x.tau_fast_s),
        "s",
    );
    add(
        &mut f,
        &format!("{prefix}.tau_slow"),
        avg(|x| x.tau_slow_s),
        "s",
    );
    add(
        &mut f,
        &format!("{prefix}.response_amplitude"),
        avg(|x| x.total_response_amplitude_v),
        "V",
    );
    add(
        &mut f,
        &format!("{prefix}.fast_amplitude"),
        avg(|x| x.fast_amplitude_v),
        "V",
    );
    add(
        &mut f,
        &format!("{prefix}.slow_amplitude"),
        avg(|x| x.slow_amplitude_v),
        "V",
    );
    add(
        &mut f,
        &format!("{prefix}.initial_response_rate"),
        avg(|x| x.initial_response_rate_v_per_s),
        "V/s",
    );
    add(
        &mut f,
        &format!("{prefix}.time_to_90_percent"),
        avg(|x| x.time_to_90_percent_s),
        "s",
    );
    add(
        &mut f,
        &format!("{prefix}.time_to_95_percent"),
        avg(|x| x.time_to_95_percent_s),
        "s",
    );
    add(
        &mut f,
        &format!("{prefix}.drift_rate"),
        avg(|x| x.drift_rate_v_per_s),
        "V/s",
    );
    let rmse = selected
        .iter()
        .filter_map(|(_, _, fit)| fit.statistics.rmse_v)
        .collect::<Vec<_>>();
    add(
        &mut f,
        &format!("{prefix}.fit_rmse"),
        statistics::mean(&rmse),
        "V",
    );
    let ac = selected
        .iter()
        .filter_map(|(_, _, fit)| fit.statistics.lag1_residual_autocorrelation)
        .collect::<Vec<_>>();
    add(
        &mut f,
        &format!("{prefix}.residual_autocorrelation"),
        statistics::mean(&ac),
        "fraction",
    );
    f
}

fn transient_context_key(
    event_index: usize,
    event: &crate::results::TransientEventResult,
) -> String {
    let before = event.concentration_before.as_ref();
    let after = event.concentration_after.as_ref();
    let analyte = after
        .and_then(|value| value.analyte.as_deref())
        .or_else(|| before.and_then(|value| value.analyte.as_deref()))
        .unwrap_or("unknown");
    let step = after
        .zip(before)
        .map(|(after, before)| after.value - before.value);
    let direction = match step {
        Some(value) if value > 0.0 => "increasing",
        Some(value) if value < 0.0 => "decreasing",
        Some(_) => "unchanged",
        None => "unknown",
    };
    let unit = after
        .and_then(|value| value.unit.as_deref())
        .or_else(|| before.and_then(|value| value.unit.as_deref()))
        .unwrap_or("unknown");
    // Matrix and temperature are not carried by a transient artifact. Include
    // their absence in the key and isolate events so unknown context is never
    // silently treated as comparable scientific evidence.
    format!(
        "analyte={analyte};step={};unit={unit};direction={direction};matrix=unknown;temperature_k=unknown;event={event_index}",
        step.map(|value| format!("{value:.12e}"))
            .unwrap_or_else(|| "unknown".into())
    )
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
