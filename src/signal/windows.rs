use crate::{
    domain::{ExperimentEvent, ExperimentEventKind},
    results::{SignalWindowSource, SignalWindowSummary},
    signal_config::{SignalWindowSource as ConfigWindowSource, WindowingConfig},
};
pub fn select(
    time: &[f64],
    events: &[ExperimentEvent],
    config: &WindowingConfig,
) -> (Vec<usize>, SignalWindowSummary) {
    let source = match config.source {
        ConfigWindowSource::EntireMeasurement => SignalWindowSource::EntireMeasurement,
        ConfigWindowSource::ExplicitInterval => SignalWindowSource::ExplicitInterval,
        ConfigWindowSource::EventRelative => SignalWindowSource::EventRelative,
        ConfigWindowSource::StableExperimentRegion => SignalWindowSource::StableExperimentRegion,
        ConfigWindowSource::ResidualArtifact => SignalWindowSource::ResidualArtifact,
    };
    let start = config.start_s.or_else(|| time.first().copied());
    let end = config.end_s.or_else(|| time.last().copied());
    let mut excluded = Vec::new();
    let eligible = |kind: ExperimentEventKind| {
        let name = match kind {
            ExperimentEventKind::ConcentrationStep => "concentration_step",
            ExperimentEventKind::FlowChange => "flow_change",
            ExperimentEventKind::TemperatureChange => "temperature_change",
            ExperimentEventKind::IonicStrengthChange => "ionic_strength_change",
            ExperimentEventKind::InterferentAddition => "interferent_addition",
            ExperimentEventKind::FlushStart => "flush_start",
            ExperimentEventKind::ReadingStart => "reading_start",
            ExperimentEventKind::FlushEnd => "flush_end",
            ExperimentEventKind::ManualAnnotation => "manual_annotation",
        };
        config.eligible_event_kinds.iter().any(|x| x == name)
    };
    for e in events {
        if eligible(e.kind) {
            excluded.push((
                e.timestamp - config.exclude_before_event_s,
                e.timestamp + config.exclude_after_event_s,
            ));
        }
    }
    let indices = time
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            start.is_none_or(|s| **t >= s)
                && end.is_none_or(|e| **t <= e)
                && !excluded.iter().any(|(a, b)| **t >= *a && **t <= *b)
        })
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    let missing = indices.len();
    (
        indices.clone(),
        SignalWindowSummary {
            source,
            start_s: start,
            end_s: end,
            source_observation_count: time.len(),
            source_timestamps: time.to_vec(),
            selected_observation_count: indices.len(),
            excluded_observations: time.len() - indices.len(),
            missing_observations: missing,
            excluded_intervals: excluded,
            resampling_method: None,
            detrending_method: None,
        },
    )
}
