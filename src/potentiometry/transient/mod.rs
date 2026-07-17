//! Event-based transient-response analysis for potentiometric channels.

pub mod diagnostics;
pub mod fitting;
pub mod models;
pub mod segmentation;
pub mod selection;

use crate::domain::{ElectrochemicalExperiment, ExperimentEventKind};
use crate::results::transient::{
    TransientAnalysisReport, TransientEventResult, TransientFitResult,
};
use crate::transient_config::ResolvedTransientConfig;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub use models::{BaselineMethod, ResponseMode, TransientModelKind};

/// Runtime options supplied by the runner after CLI/config resolution.
#[derive(Debug, Clone)]
pub struct TransientAnalysisOptions {
    pub event_kind: ExperimentEventKind,
    pub event_index: Option<usize>,
    pub config: ResolvedTransientConfig,
}

/// Analyze all selected events without modifying the source experiment.
pub fn analyze_experiment(
    experiment: &ElectrochemicalExperiment,
    channel_name: &str,
    options: &TransientAnalysisOptions,
) -> Result<TransientAnalysisReport, crate::potentiometry::PotentiometryError> {
    options.config.validate().map_err(|error| {
        crate::potentiometry::PotentiometryError::InvalidConfiguration(error.to_string())
    })?;
    let channel = experiment
        .measurement_data
        .channel(channel_name)
        .ok_or_else(
            || crate::potentiometry::PotentiometryError::MissingChannel {
                channel: channel_name.to_string(),
            },
        )?;

    let eligible = experiment
        .events
        .iter()
        .enumerate()
        .filter(|(_, event)| event.kind == options.event_kind)
        .collect::<Vec<_>>();
    if eligible.is_empty() {
        return Err(crate::potentiometry::PotentiometryError::NoEligibleEvents {
            event_kind: format_event_kind(options.event_kind),
        });
    }

    if let Some(index) = options.event_index
        && index >= eligible.len()
    {
        return Err(
            crate::potentiometry::PotentiometryError::InvalidEventWindow(format!(
                "event index {index} is outside the {} eligible events",
                eligible.len()
            )),
        );
    }

    let mut events = Vec::new();
    for (eligible_index, (source_index, event)) in eligible.into_iter().enumerate() {
        if options
            .event_index
            .is_some_and(|index| index != eligible_index)
        {
            continue;
        }

        let concentration_before =
            segmentation::derive_concentration_before(&experiment.events, source_index, event);
        let concentration_after = segmentation::concentration_context(event);
        let result = match segmentation::prepare_segment(
            &experiment.measurement_data,
            channel,
            event,
            source_index,
            &options.config,
        ) {
            Ok(segment) => analyze_one_event(
                eligible_index,
                event,
                concentration_before,
                concentration_after,
                segment,
                &options.config,
            ),
            Err(error) => TransientEventResult::failed(
                eligible_index,
                event.clone(),
                concentration_before,
                concentration_after,
                error.to_string(),
            ),
        };
        events.push(result);
    }

    Ok(TransientAnalysisReport {
        schema_version: 1,
        experiment_id: experiment.experiment_id.clone(),
        channel: channel.name.clone(),
        channel_unit: channel.unit.clone(),
        parse_diagnostics: experiment.measurement_data.diagnostics(),
        configuration: options.config.clone(),
        provenance: experiment.provenance.clone(),
        events,
    })
}

fn analyze_one_event(
    event_index: usize,
    event: &crate::domain::ExperimentEvent,
    concentration_before: Option<crate::results::transient::ConcentrationContext>,
    concentration_after: Option<crate::results::transient::ConcentrationContext>,
    segment: segmentation::PreparedSegment,
    config: &ResolvedTransientConfig,
) -> TransientEventResult {
    let mut warnings = segment.warnings.clone();
    let mut candidate_fits = Vec::new();
    let mut rng = StdRng::seed_from_u64(config.uncertainty.seed);

    for model in config.models.enabled.iter().copied() {
        match fitting::fit_transient_model(&segment, model, config, &mut rng) {
            Ok(fit) => candidate_fits.push(fit),
            Err(error) => candidate_fits.push(TransientFitResult::failed(model, error.to_string())),
        }
    }

    let selection = selection::select_model(&mut candidate_fits, config.selection.criterion);
    let selected_model = selection.selected_model;
    if selected_model.is_none() {
        warnings.push(crate::results::transient::TransientWarning::new(
            crate::results::transient::TransientWarningKind::AllModelsFailed,
            "all configured candidate models failed or were invalid",
        ));
    }
    let failure =
        selected_model
            .is_none()
            .then(|| crate::results::transient::TransientFitFailure {
                message: crate::potentiometry::PotentiometryError::AllCandidateModelsFailed {
                    event_index,
                }
                .to_string(),
            });

    if let Some(index) = selection.selected_index
        && config.uncertainty.bootstrap_iterations > 0
        && let Err(error) = fitting::bootstrap_fit(&segment, &mut candidate_fits[index], config)
    {
        candidate_fits[index]
            .warnings
            .push(crate::results::transient::TransientWarning::new(
                crate::results::transient::TransientWarningKind::BootstrapUnavailable,
                error.to_string(),
            ));
    }

    TransientEventResult {
        event_index,
        event: event.clone(),
        concentration_before,
        concentration_after,
        segment: segment.summary,
        baseline: segment.baseline,
        candidate_fits,
        selected_model,
        warnings,
        failure,
    }
}

fn format_event_kind(kind: ExperimentEventKind) -> String {
    match kind {
        ExperimentEventKind::ConcentrationStep => "concentration-step",
        ExperimentEventKind::FlowChange => "flow-change",
        ExperimentEventKind::TemperatureChange => "temperature-change",
        ExperimentEventKind::IonicStrengthChange => "ionic-strength-change",
        ExperimentEventKind::InterferentAddition => "interferent-addition",
        ExperimentEventKind::FlushStart => "flush-start",
        ExperimentEventKind::ReadingStart => "reading-start",
        ExperimentEventKind::FlushEnd => "flush-end",
        ExperimentEventKind::ManualAnnotation => "manual-annotation",
    }
    .to_string()
}
