use crate::{
    domain::ElectrochemicalExperiment,
    estimation::{
        calibration_adapter::CalibrationObservationModel,
        environment::AlignedEnvironment,
        error::EstimationError,
        model::StateModel,
        state::{EstimationWarning, EstimationWarningKind},
    },
    estimation_config::ResolvedEstimationConfig,
};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializationReport {
    pub state_mean: Vec<f64>,
    pub state_covariance: Vec<Vec<f64>>,
    pub sources: Vec<String>,
    pub assumptions: Vec<String>,
    pub warnings: Vec<EstimationWarning>,
}

pub fn initialize_state(
    experiment: &ElectrochemicalExperiment,
    channel: &str,
    model: &StateModel,
    calibration: &dyn CalibrationObservationModel,
    config: &ResolvedEstimationConfig,
    environment: &AlignedEnvironment,
) -> Result<(DVector<f64>, DMatrix<f64>, InitializationReport), EstimationError> {
    let c = experiment
        .measurement()
        .channel(channel)
        .ok_or_else(|| EstimationError::invalid("initialization channel is unavailable"))?;
    let first = c.values.iter().copied().flatten().next().ok_or_else(|| {
        EstimationError::invalid("no valid potential is available for initialization")
    })?;
    let mut mean = DVector::zeros(model.dimension());
    let mut sources = Vec::new();
    let mut assumptions = Vec::new();
    let mut warnings = Vec::new();
    let source = config.initialization.activity_source.to_ascii_lowercase();
    let standard_source = config.auxiliary.allow_known_standard_events
        && environment.known_standard
        && environment.known_activity_log10.is_some();
    let log_activity = if standard_source {
        sources.push("annotated known-activity standard event".into());
        environment.known_activity_log10.unwrap()
    } else if source.contains("configured") {
        config.initialization.initial_activity.log10()
    } else {
        match calibration.inverse_log10_activity(first, environment) {
            Ok(v) => {
                sources.push("calibration inversion of first valid potential".into());
                v
            }
            Err(error) => {
                warnings.push(EstimationWarning::new(
                    EstimationWarningKind::ModelDiscrepancy,
                    format!(
                        "initial calibration inversion failed: {error}; configured activity used"
                    ),
                ));
                config.initialization.initial_activity.log10()
            }
        }
    };
    mean[model.index("log10_activity").unwrap_or(0)] = log_activity;
    if !standard_source {
        sources.push(if source.contains("configured") {
            "configured initial activity".into()
        } else {
            "initial activity prior".into()
        });
    }
    if let Some(i) = model.index("baseline_offset") {
        mean[i] = config.initialization.baseline_v;
        sources.push("configured baseline offset".into());
        assumptions.push("baseline is initialized to the configured offset and is latent".into());
    }
    if let Some(i) = model.index("polarization") {
        mean[i] = config.initialization.polarization_v;
        sources.push("configured polarization state".into());
        assumptions.push("polarization is initialized from the configured prior; transient data may constrain its time constant".into());
    }
    if let Some(i) = model.index("sensitivity_scale") {
        mean[i] = if config.initialization.condition_value.is_finite() {
            config.initialization.condition_value
        } else {
            config.state_model.condition_initial
        };
        sources.push("configured sensitivity-scale prior".into());
        if config.auxiliary.condition_requires_auxiliary {
            assumptions.push(
                "condition state is retained only after observability evidence is checked".into(),
            );
        }
    }
    let mut covariance = DMatrix::zeros(model.dimension(), model.dimension());
    for (i, d) in model.definitions.iter().enumerate() {
        covariance[(i, i)] = match d.name.as_str() {
            "log10_activity" => config.initial_covariance.log10_activity_variance,
            "baseline_offset" => config.initial_covariance.baseline_variance_v2,
            "polarization" => config.initial_covariance.polarization_variance_v2,
            "sensitivity_scale" => config.initial_covariance.condition_variance,
            _ => 1e-6,
        };
    }
    if model.has_baseline() && model.has_polarization() {
        assumptions.push("the first-potential inversion does not identify baseline and polarization independently without subsequent dynamics or auxiliary information".into());
    }
    if mean.iter().any(|x| !x.is_finite()) || covariance.iter().any(|x| !x.is_finite()) {
        return Err(EstimationError::Numerical(
            "initial state is nonfinite".into(),
        ));
    }
    let report = InitializationReport {
        state_mean: mean.iter().copied().collect(),
        state_covariance: (0..covariance.nrows())
            .map(|i| {
                (0..covariance.ncols())
                    .map(|j| covariance[(i, j)])
                    .collect()
            })
            .collect(),
        sources,
        assumptions,
        warnings,
    };
    Ok((mean, covariance, report))
}
