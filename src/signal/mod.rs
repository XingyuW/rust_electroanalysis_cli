//! Generic, non-destructive signal processing algorithms.

pub mod allan;
pub mod comparison;
pub mod correlation;
pub mod drift;
pub mod error;
pub mod psd;
pub mod residuals;
pub mod sampling;
pub mod spikes;
pub mod statistics;
pub mod windows;

pub use crate::signal_config::{
    DuplicateTimestampPolicy, NonMonotonicTimestampPolicy, SamplingPolicy, SignalWindowSource,
};

use crate::{
    domain::{AnalysisProvenance, ExperimentEvent, ExperimentEventKind, MultiChannelMeasurement},
    results::{
        DriftModelKind, ResidualAnalysisResult, SignalAnalysisReport, SignalWarning, SpikeAnalysis,
    },
    signal_config::ResolvedSignalConfig,
};
use error::SignalError;

pub fn analyze_measurement(
    measurement: &MultiChannelMeasurement,
    channel_name: &str,
    events: Option<&[ExperimentEvent]>,
    config: &ResolvedSignalConfig,
    provenance: Option<AnalysisProvenance>,
) -> Result<SignalAnalysisReport, SignalError> {
    let channel = measurement.channel(channel_name).ok_or_else(|| {
        SignalError::invalid(format!("selected channel '{channel_name}' does not exist"))
    })?;
    let events = events.unwrap_or(&[]);
    let (indices, mut window) = windows::select(&measurement.time, events, &config.windowing);
    if indices.is_empty() {
        return Err(SignalError::invalid(
            "analysis window contains no observations",
        ));
    }
    let time = indices
        .iter()
        .map(|i| measurement.time[*i])
        .collect::<Vec<_>>();
    let values = indices
        .iter()
        .map(|i| channel.values[*i])
        .collect::<Vec<_>>();
    let (sampling, analysis_time, analysis_values) =
        sampling::analyze_sampling(&time, &values, &config.sampling)?;
    window.missing_observations = analysis_values.iter().filter(|v| v.is_none()).count();
    window.resampling_method = if sampling.interpolation_count > 0 {
        Some("linear".into())
    } else {
        None
    };
    window.detrending_method = Some(format!("{:?}", config.psd.detrend));
    let descriptive = statistics::descriptive(
        &analysis_values,
        &config.statistics.quantiles,
        config.statistics.confidence_level,
    );
    let mut warnings = Vec::new();
    if !sampling.is_regular {
        warnings.push(SignalWarning::IrregularSampling);
    }
    if sampling.duplicate_timestamps > 0 {
        warnings.push(SignalWarning::DuplicateTimestamps);
    }
    if sampling.non_monotonic_timestamps > 0 {
        warnings.push(SignalWarning::NonMonotonicTimestamps);
    }
    if sampling.missing_fraction.is_some_and(|v| v > 0.2) {
        warnings.push(SignalWarning::ExcessiveMissingData);
    }
    if sampling.interpolation_count > 0 {
        warnings.push(SignalWarning::ResamplingPerformed);
    }
    if sampling.interpolation_gap_exceeded {
        warnings.push(SignalWarning::InterpolationGapExceeded);
    }
    if analysis_values.len() < 16 {
        warnings.push(SignalWarning::RecordTooShort);
    }
    let finite_values = analysis_values
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    let complete = finite_values.len() == analysis_values.len();
    let psd = if config.psd.enabled && sampling.is_regular && complete {
        match psd::welch(&analysis_time, &finite_values, &config.psd) {
            Ok(mut p) => {
                p.psd_unit = format!("{}^2/Hz", channel.unit);
                Some(p)
            }
            Err(_) => {
                warnings.push(SignalWarning::RecordTooShort);
                None
            }
        }
    } else {
        None
    };
    let allan = if config.allan.enabled && sampling.is_regular && complete {
        allan::overlapping(&analysis_time, &finite_values, &config.allan).ok()
    } else {
        None
    };
    let finite_pairs = analysis_time
        .iter()
        .copied()
        .zip(&analysis_values)
        .filter_map(|(t, v)| v.map(|y| (t, y)))
        .collect::<Vec<_>>();
    let (dt, dv): (Vec<_>, Vec<_>) = finite_pairs.into_iter().unzip();
    let mut drift = Vec::new();
    for name in &config.drift.models {
        let model = match name.to_ascii_lowercase().as_str() {
            "theil_sen" => DriftModelKind::TheilSen,
            "weighted_linear" => DriftModelKind::WeightedLinear,
            "event_segmented_linear" => DriftModelKind::EventSegmentedLinear,
            _ => DriftModelKind::OrdinaryLinear,
        };
        if dt.len() > 1
            && config.drift.minimum_duration_s
                <= dt.last().unwrap_or(&0.0) - dt.first().unwrap_or(&0.0)
        {
            drift.push(drift::estimate(&dt, &dv, model));
        } else {
            warnings.push(SignalWarning::DriftDurationInsufficient);
        }
    }
    let spikes = if config.spikes.enabled {
        spikes::detect(&analysis_time, &analysis_values, &config.spikes)
    } else {
        SpikeAnalysis {
            method: "disabled".into(),
            flagged: Vec::new(),
            flagged_fraction: Some(0.0),
            maximum_flagged_fraction: config.spikes.maximum_flagged_fraction,
        }
    };
    if spikes
        .flagged_fraction
        .is_some_and(|v| v > config.spikes.maximum_flagged_fraction)
    {
        warnings.push(SignalWarning::ExcessiveSpikeFraction);
    }
    let mut correlations = Vec::new();
    if config.correlation.enabled {
        for other in &measurement.channels {
            if other.name == channel.name {
                continue;
            }
            let c = correlation::pair(
                &channel.name,
                &measurement.time,
                &channel.values,
                &other.name,
                &measurement.time,
                &other.values,
                &config.correlation,
            );
            correlations.push(c);
        }
    }
    let provenance =
        provenance.ok_or_else(|| SignalError::invalid("signal provenance is required"))?;
    Ok(SignalAnalysisReport {
        schema_version: 1,
        analysis_id: format!("signal:{}:{}", provenance.input_sha256, channel.name),
        experiment_id: None,
        sensor_id: channel.sensor_id.clone(),
        channel: channel.name.clone(),
        unit: channel.unit.clone(),
        analysis_timestamps: analysis_time,
        analysis_values: analysis_values.clone(),
        window,
        sampling,
        descriptive,
        psd,
        allan,
        drift,
        spikes,
        correlations,
        residual_analysis: Vec::<ResidualAnalysisResult>::new(),
        configuration: config.clone(),
        provenance,
        warnings,
    })
}

pub fn event_is_exclusion_kind(kind: ExperimentEventKind) -> bool {
    matches!(
        kind,
        ExperimentEventKind::ConcentrationStep
            | ExperimentEventKind::FlowChange
            | ExperimentEventKind::TemperatureChange
            | ExperimentEventKind::InterferentAddition
    )
}
