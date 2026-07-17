use crate::{
    estimation::{
        calibration_adapter::CalibrationObservationModel, error::EstimationError,
        measurement::MeasurementObservation, model::StateModel, state::CalibrationDomainStatus,
    },
    estimation_config::{
        CovarianceSourceKind, MeasurementNoiseSourceKind, ProcessNoiseConfig,
        ResolvedEstimationConfig,
    },
    results::{CalibrationAnalysisReport, SignalAnalysisReport},
};
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CovarianceSource {
    Configured,
    SignalArtifact,
    CalibrationArtifact,
    TransientArtifact,
    EstimatedFromTrainingData,
}
impl From<CovarianceSourceKind> for CovarianceSource {
    fn from(v: CovarianceSourceKind) -> Self {
        match v {
            CovarianceSourceKind::Configured => Self::Configured,
            CovarianceSourceKind::SignalArtifact => Self::SignalArtifact,
            CovarianceSourceKind::CalibrationArtifact => Self::CalibrationArtifact,
            CovarianceSourceKind::TransientArtifact => Self::TransientArtifact,
            CovarianceSourceKind::EstimatedFromTrainingData => Self::EstimatedFromTrainingData,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CovarianceCandidate {
    pub source: CovarianceSource,
    pub variance: Option<f64>,
    pub unit: String,
    pub accepted: bool,
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CovarianceResolution {
    pub candidates: Vec<CovarianceCandidate>,
    pub selected_source: CovarianceSource,
    pub rejected_sources: Vec<String>,
    pub combination_assumptions: Vec<String>,
    pub final_variance: f64,
    pub unit: String,
}

/// The variance actually supplied to one scalar voltage update.  Keeping the
/// uninflated value and provenance separate is important: calibration-domain
/// inflation is a model-risk allowance, not an additional instrument-noise
/// observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementVarianceResolution {
    pub uninflated_variance_v2: f64,
    pub effective_variance_v2: f64,
    pub source: String,
    pub inflation_factor: f64,
    pub inflation_reason: Option<String>,
}

impl CovarianceResolution {
    pub fn validate(&self) -> Result<(), EstimationError> {
        if !self.final_variance.is_finite() || self.final_variance < 0.0 {
            Err(EstimationError::Covariance(
                "resolved variance is invalid".into(),
            ))
        } else {
            Ok(())
        }
    }
}

pub fn resolve_measurement_covariance(
    config: &ResolvedEstimationConfig,
    observation_variance: Option<f64>,
    signal: Option<&SignalAnalysisReport>,
    calibration: Option<&CalibrationAnalysisReport>,
) -> Result<CovarianceResolution, EstimationError> {
    let mut candidates = Vec::new();
    let mut add = |source, variance, reason: &str| {
        candidates.push(CovarianceCandidate {
            source,
            variance,
            unit: "V^2".into(),
            accepted: false,
            reason: reason.into(),
        })
    };
    add(
        CovarianceSource::Configured,
        Some(config.measurement_noise.configured_variance_v2),
        "explicit configuration",
    );
    add(
        CovarianceSource::SignalArtifact,
        signal.and_then(|s| s.descriptive.robust_standard_deviation.map(|v| v * v)),
        "robust voltage standard deviation squared; PSD units were not used",
    );
    add(
        CovarianceSource::CalibrationArtifact,
        calibration.and_then(|c| {
            c.selected_model
                .and_then(|k| c.candidate_models.iter().find(|m| m.model_kind == k))
                .and_then(|m| m.statistics.rmse_v)
                .map(|v| v * v)
        }),
        "calibration residual RMSE squared",
    );
    add(
        CovarianceSource::Configured,
        observation_variance,
        "per-observation uncertainty",
    );
    let requested = match config.measurement_noise.source {
        crate::estimation_config::MeasurementNoiseSourceKind::Configured => {
            CovarianceSource::Configured
        }
        crate::estimation_config::MeasurementNoiseSourceKind::PerObservation => {
            CovarianceSource::Configured
        }
        crate::estimation_config::MeasurementNoiseSourceKind::SignalRobustVariance => {
            CovarianceSource::SignalArtifact
        }
        crate::estimation_config::MeasurementNoiseSourceKind::StableWindowVariance => {
            CovarianceSource::SignalArtifact
        }
        crate::estimation_config::MeasurementNoiseSourceKind::CalibrationResidualVariance => {
            CovarianceSource::CalibrationArtifact
        }
        crate::estimation_config::MeasurementNoiseSourceKind::CalibrationPredictionUncertainty => {
            CovarianceSource::CalibrationArtifact
        }
    };
    let selected_variance = if matches!(
        config.measurement_noise.source,
        crate::estimation_config::MeasurementNoiseSourceKind::PerObservation
    ) {
        observation_variance
    } else {
        candidates
            .iter()
            .find(|c| c.source == requested)
            .and_then(|c| c.variance)
    }
    .or_else(|| {
        candidates
            .iter()
            .find(|c| c.source == CovarianceSource::Configured)
            .and_then(|c| c.variance)
    });
    let mut value = selected_variance.ok_or_else(|| {
        EstimationError::Covariance(
            "no compatible voltage measurement variance is available".into(),
        )
    })?;
    value = value.clamp(
        config.measurement_noise.minimum_variance_v2,
        config.measurement_noise.maximum_variance_v2,
    );
    for c in &mut candidates {
        c.accepted = c.source == requested && c.variance.is_some();
    }
    let source = if candidates.iter().any(|c| c.accepted) {
        requested
    } else {
        CovarianceSource::Configured
    };
    let rejected_sources = candidates
        .iter()
        .filter(|c| !c.accepted)
        .map(|c| format!("{:?}: {}", c.source, c.reason))
        .collect();
    let result=CovarianceResolution { candidates, selected_source:source, rejected_sources, combination_assumptions:vec!["one primary voltage-variance source selected; candidate variances were not double-counted".into()], final_variance:value, unit:"V^2".into() };
    result.validate()?;
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_observation_variance(
    config: &ResolvedEstimationConfig,
    observation: &MeasurementObservation,
    signal: Option<&SignalAnalysisReport>,
    calibration_report: Option<&CalibrationAnalysisReport>,
    calibration: &dyn CalibrationObservationModel,
    log10_activity: f64,
    environment: &crate::estimation::environment::AlignedEnvironment,
    domain: CalibrationDomainStatus,
) -> Result<MeasurementVarianceResolution, EstimationError> {
    let source = config.measurement_noise.source;
    let (variance, source_name, reason) = match source {
        MeasurementNoiseSourceKind::Configured => (
            Some(config.measurement_noise.configured_variance_v2),
            "configured".to_string(),
            "explicit configured voltage variance".to_string(),
        ),
        MeasurementNoiseSourceKind::PerObservation => (
            observation.observation_variance_v2,
            "per_observation".to_string(),
            "variance supplied with the selected measurement row".to_string(),
        ),
        MeasurementNoiseSourceKind::SignalRobustVariance => (
            signal.and_then(|s| s.descriptive.robust_standard_deviation.map(|v| v * v)),
            "signal_robust_variance".to_string(),
            "robust standard deviation from the signal artifact squared".to_string(),
        ),
        MeasurementNoiseSourceKind::StableWindowVariance => (
            signal.and_then(|s| s.descriptive.sample_variance),
            "stable_window_variance".to_string(),
            "sample variance from the selected signal window".to_string(),
        ),
        MeasurementNoiseSourceKind::CalibrationResidualVariance => (
            calibration_report
                .and_then(|c| c.selected_model)
                .and_then(|kind| {
                    calibration_report?.candidate_models.iter().find(|m| m.model_kind == kind)
                })
                .and_then(|m| m.statistics.rmse_v)
                .map(|v| v * v),
            "calibration_residual_variance".to_string(),
            "calibration residual RMSE squared; this is residual scatter, not prediction uncertainty".to_string(),
        ),
        MeasurementNoiseSourceKind::CalibrationPredictionUncertainty => (
            calibration.prediction_uncertainty_v2(log10_activity, environment),
            "calibration_prediction_uncertainty".to_string(),
            "delta-method prediction variance from calibration parameter covariance or documented diagonal approximation".to_string(),
        ),
    };
    let mut source_name = source_name;
    let mut reason = reason;
    let variance = match variance {
        Some(value) => value,
        None if !matches!(source, MeasurementNoiseSourceKind::PerObservation)
            && !matches!(
                source,
                MeasurementNoiseSourceKind::CalibrationPredictionUncertainty
            ) =>
        {
            source_name.push_str("_fallback_configured");
            reason
                .push_str("; requested artifact was unavailable, so configured variance was used");
            config.measurement_noise.configured_variance_v2
        }
        None => {
            return Err(EstimationError::Covariance(format!(
                "measurement variance source '{source_name}' is unavailable"
            )));
        }
    };
    if !variance.is_finite() || variance <= 0.0 {
        return Err(EstimationError::Covariance(format!(
            "measurement variance from '{source_name}' must be finite and positive"
        )));
    }
    let uninflated = variance.clamp(
        config.measurement_noise.minimum_variance_v2,
        config.measurement_noise.maximum_variance_v2,
    );
    if !uninflated.is_finite() || uninflated <= 0.0 {
        return Err(EstimationError::Covariance(
            "effective uninflated measurement variance is invalid".into(),
        ));
    }
    let (factor, inflation_reason) = match domain {
        CalibrationDomainStatus::Outside
            if config.measurement_noise.inflate_outside_domain
                || config.extrapolation.inflate_measurement_variance =>
        {
            (
                config.extrapolation.variance_inflation_factor,
                Some("outside calibration domain".to_string()),
            )
        }
        _ if config.extrapolation.inflate_measurement_variance
            && calibration.near_boundary(
                log10_activity,
                environment,
                config.extrapolation.near_boundary_fraction,
            ) =>
        {
            (
                config.extrapolation.near_boundary_variance_inflation_factor,
                Some("near calibration-domain boundary".to_string()),
            )
        }
        _ => (1.0, None),
    };
    let effective = (uninflated * factor).min(config.measurement_noise.maximum_variance_v2);
    if !effective.is_finite() || effective <= 0.0 {
        return Err(EstimationError::Covariance(
            "inflated measurement variance is invalid".into(),
        ));
    }
    Ok(MeasurementVarianceResolution {
        uninflated_variance_v2: uninflated,
        effective_variance_v2: effective,
        source: source_name,
        inflation_factor: effective / uninflated,
        inflation_reason,
    })
}

pub fn resolve_process_covariance(
    config: &ResolvedEstimationConfig,
    model: &StateModel,
    dt_s: f64,
) -> Result<(DMatrix<f64>, CovarianceResolution), EstimationError> {
    if !dt_s.is_finite() || dt_s <= 0.0 {
        return Err(EstimationError::Covariance(
            "transition interval must be positive".into(),
        ));
    }
    let q = model.process_covariance(dt_s, &config.process_noise);
    if q.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(EstimationError::Covariance(
            "process covariance contains an invalid entry".into(),
        ));
    }
    let total = q.diagonal().iter().sum::<f64>();
    let resolution = CovarianceResolution {
        candidates: vec![CovarianceCandidate {
            source: config.process_noise.source.into(),
            variance: Some(total),
            unit: "state covariance".into(),
            accepted: true,
            reason: "configured process spectral density integrated over actual dt".into(),
        }],
        selected_source: config.process_noise.source.into(),
        rejected_sources: Vec::new(),
        combination_assumptions: vec![
            "random-walk terms scale with actual timestamp interval".into(),
            "polarization covariance uses exact first-order integrated variance".into(),
        ],
        final_variance: total,
        unit: "state covariance".into(),
    };
    Ok((q, resolution))
}

pub fn symmetrize(matrix: &mut DMatrix<f64>) {
    *matrix = (&*matrix + matrix.transpose()) * 0.5;
}
pub fn is_psd(matrix: &DMatrix<f64>, tolerance: f64) -> bool {
    if matrix.nrows() != matrix.ncols() || matrix.iter().any(|v| !v.is_finite()) {
        return false;
    }
    matrix
        .clone()
        .symmetric_eigen()
        .eigenvalues
        .iter()
        .all(|v| *v >= -tolerance)
}
pub fn finite_vector(v: &DVector<f64>) -> bool {
    v.iter().all(|x| x.is_finite())
}
pub fn process_noise_config(config: &ResolvedEstimationConfig) -> &ProcessNoiseConfig {
    &config.process_noise
}
