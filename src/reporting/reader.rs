//! Canonical artifact loading and the reused Phase-C compatibility gate.

use crate::{
    domain::{
        self, ArtifactError, ArtifactExperimentScope, ArtifactLineageCatalog, ArtifactLineageState,
        ScopeKey, VersionedArtifact,
    },
    report_config::ReportRenderOptions,
    reporting::{CompatibilityAxis, PublicReportError},
    results::{
        CalibrationAnalysisReport, CalibrationObservationSet, EisFitArtifact,
        MechanismAnalysisReport, ModelAnalysisReport, SensorHealthAssessment, SignalAnalysisReport,
        StateEstimationReport, TransientAnalysisReport,
    },
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CompatibilityStatus {
    Compatible,
    LegacyUnknown,
    NotProvided,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) struct CompatibilityOutcome {
    pub status: CompatibilityStatus,
    pub mismatch_axis: Option<CompatibilityAxis>,
}

/// Typed, canonical-reader-only inputs for a public report render.
#[derive(Debug, Clone)]
pub(crate) struct ReportInputs {
    pub input_paths: ReportInputPaths,
    pub mechanism: MechanismAnalysisReport,
    pub health: SensorHealthAssessment,
    pub lineage_catalog: Option<ArtifactLineageCatalog>,
    pub eis: Option<EisFitArtifact>,
    pub transient: Option<TransientAnalysisReport>,
    pub calibration: Option<CalibrationAnalysisReport>,
    pub calibration_observations: Option<CalibrationObservationSet>,
    pub signal: Option<SignalAnalysisReport>,
    pub estimation: Option<StateEstimationReport>,
    pub model: Option<ModelAnalysisReport>,
    pub required_compatibility: CompatibilityOutcome,
    pub optional_compatibility: Vec<(&'static str, &'static str, CompatibilityOutcome)>,
}

#[derive(Debug, Clone)]
pub(crate) struct ReportInputPaths {
    pub mechanism: PathBuf,
    pub health: PathBuf,
    pub lineage_catalog: Option<PathBuf>,
    pub eis: Option<PathBuf>,
    pub transient: Option<PathBuf>,
    pub calibration: Option<PathBuf>,
    pub calibration_observations: Option<PathBuf>,
    pub signal: Option<PathBuf>,
    pub estimation: Option<PathBuf>,
    pub model: Option<PathBuf>,
}

impl ReportInputs {
    pub fn read(options: &ReportRenderOptions) -> Result<Self, PublicReportError> {
        // Catalogs never enter an artifact scope gate; retain this explicit
        // presentation status at the canonical-reader boundary.
        let _catalog_compatibility = CompatibilityStatus::NotApplicable;
        options.validate_pairing()?;
        let mechanism =
            read_artifact::<MechanismAnalysisReport>("--mechanism", &options.mechanism)?;
        ensure_schema(
            "--mechanism",
            &options.mechanism,
            mechanism.schema_version,
            &[1, 2, 3, 4],
        )?;
        let health = read_artifact::<SensorHealthAssessment>("--health", &options.health)?;
        ensure_schema("--health", &options.health, health.schema_version, &[3, 4])?;

        let required_compatibility = require_compatible(
            "--mechanism",
            &mechanism.lineage,
            "--health",
            &health.lineage,
        )?;

        let lineage_catalog =
            match &options.lineage_catalog {
                Some(path) => Some(domain::read_artifact_lineage_catalog(path).map_err(
                    |source| PublicReportError::LineageCatalog {
                        path: path.clone(),
                        source,
                    },
                )?),
                None => None,
            };
        let eis = read_optional::<EisFitArtifact>("--eis", options.eis.as_deref(), &[3])?;
        let transient = read_optional::<TransientAnalysisReport>(
            "--transient",
            options.transient.as_deref(),
            &[3],
        )?;
        let calibration = read_optional::<CalibrationAnalysisReport>(
            "--calibration",
            options.calibration.as_deref(),
            &[3],
        )?;
        let calibration_observations = read_optional::<CalibrationObservationSet>(
            "--calibration-observations",
            options.calibration_observations.as_deref(),
            &[3],
        )?;
        let signal =
            read_optional::<SignalAnalysisReport>("--signal", options.signal.as_deref(), &[3])?;
        let estimation = read_optional::<StateEstimationReport>(
            "--estimation",
            options.estimation.as_deref(),
            &[4],
        )?;
        let model =
            read_optional::<ModelAnalysisReport>("--model", options.model.as_deref(), &[5])?;

        let mut optional_compatibility = Vec::new();
        check_optional(
            "--eis",
            eis.as_ref().map(|value| &value.lineage),
            &mechanism,
            &health,
            &mut optional_compatibility,
        )?;
        check_optional(
            "--transient",
            transient.as_ref().map(|value| &value.lineage),
            &mechanism,
            &health,
            &mut optional_compatibility,
        )?;
        if let (Some(calibration), Some(observations)) = (&calibration, &calibration_observations) {
            let outcome = optional_compatible(
                "--calibration",
                &calibration.lineage,
                "--calibration-observations",
                &observations.lineage,
            )?;
            optional_compatibility.push(("--calibration", "--calibration-observations", outcome));
        }
        check_optional(
            "--calibration",
            calibration.as_ref().map(|value| &value.lineage),
            &mechanism,
            &health,
            &mut optional_compatibility,
        )?;
        check_optional(
            "--calibration-observations",
            calibration_observations
                .as_ref()
                .map(|value| &value.lineage),
            &mechanism,
            &health,
            &mut optional_compatibility,
        )?;
        check_optional(
            "--signal",
            signal.as_ref().map(|value| &value.lineage),
            &mechanism,
            &health,
            &mut optional_compatibility,
        )?;
        check_optional(
            "--estimation",
            estimation.as_ref().map(|value| &value.lineage),
            &mechanism,
            &health,
            &mut optional_compatibility,
        )?;
        check_optional(
            "--model",
            model.as_ref().map(|value| &value.lineage),
            &mechanism,
            &health,
            &mut optional_compatibility,
        )?;

        Ok(Self {
            input_paths: ReportInputPaths {
                mechanism: options.mechanism.clone(),
                health: options.health.clone(),
                lineage_catalog: options.lineage_catalog.clone(),
                eis: options.eis.clone(),
                transient: options.transient.clone(),
                calibration: options.calibration.clone(),
                calibration_observations: options.calibration_observations.clone(),
                signal: options.signal.clone(),
                estimation: options.estimation.clone(),
                model: options.model.clone(),
            },
            mechanism,
            health,
            lineage_catalog,
            eis,
            transient,
            calibration,
            calibration_observations,
            signal,
            estimation,
            model,
            required_compatibility,
            optional_compatibility,
        })
    }
}

fn read_artifact<T: VersionedArtifact>(
    flag: &'static str,
    path: &Path,
) -> Result<T, PublicReportError> {
    domain::read_artifact(path).map_err(|source| PublicReportError::Artifact {
        flag,
        path: path.to_path_buf(),
        source,
    })
}

fn read_optional<T: VersionedArtifact>(
    flag: &'static str,
    path: Option<&Path>,
    allowed_schemas: &[u32],
) -> Result<Option<T>, PublicReportError> {
    let Some(path) = path else {
        return Ok(None);
    };
    let artifact = read_artifact::<T>(flag, path)?;
    ensure_schema(flag, path, artifact.schema_version(), allowed_schemas)?;
    Ok(Some(artifact))
}

fn ensure_schema(
    flag: &'static str,
    path: &Path,
    actual: u32,
    allowed: &[u32],
) -> Result<(), PublicReportError> {
    if allowed.contains(&actual) {
        return Ok(());
    }
    // The canonical reader owns kind validation; this explicit gate is the
    // Phase-D contract's narrower schema policy.
    Err(PublicReportError::Artifact {
        flag,
        path: path.to_path_buf(),
        source: ArtifactError::UnsupportedSchemaVersion {
            path: path.to_path_buf(),
            expected: match flag {
                "--mechanism" => crate::domain::ArtifactKind::MechanismAnalysis,
                "--health" => crate::domain::ArtifactKind::HealthAssessment,
                "--eis" => crate::domain::ArtifactKind::EisFit,
                "--transient" => crate::domain::ArtifactKind::TransientAnalysis,
                "--calibration" => crate::domain::ArtifactKind::CalibrationAnalysis,
                "--calibration-observations" => {
                    crate::domain::ArtifactKind::CalibrationObservations
                }
                "--signal" => crate::domain::ArtifactKind::SignalAnalysis,
                "--estimation" => crate::domain::ArtifactKind::StateEstimation,
                "--model" => crate::domain::ArtifactKind::ModelAnalysis,
                _ => unreachable!("report reader only uses fixed artifact flags"),
            },
            actual,
        },
    })
}

fn check_optional(
    flag: &'static str,
    lineage: Option<&ArtifactLineageState>,
    mechanism: &MechanismAnalysisReport,
    health: &SensorHealthAssessment,
    outcomes: &mut Vec<(&'static str, &'static str, CompatibilityOutcome)>,
) -> Result<(), PublicReportError> {
    let Some(lineage) = lineage else {
        outcomes.push((
            flag,
            "--mechanism",
            CompatibilityOutcome {
                status: CompatibilityStatus::NotProvided,
                mismatch_axis: None,
            },
        ));
        return Ok(());
    };
    let mechanism_outcome = optional_compatible(flag, lineage, "--mechanism", &mechanism.lineage)?;
    outcomes.push((flag, "--mechanism", mechanism_outcome));
    let health_outcome = optional_compatible(flag, lineage, "--health", &health.lineage)?;
    outcomes.push((flag, "--health", health_outcome));
    Ok(())
}

fn require_compatible(
    left_flag: &'static str,
    left: &ArtifactLineageState,
    right_flag: &'static str,
    right: &ArtifactLineageState,
) -> Result<CompatibilityOutcome, PublicReportError> {
    if let Some((axis, left_value, right_value)) = scope_mismatch(left, right) {
        return Err(PublicReportError::RequiredInputsIncompatible {
            left_flag,
            right_flag,
            axis,
            left: left_value,
            right: right_value,
        });
    }
    Ok(CompatibilityOutcome {
        status: if both_known(left, right) {
            CompatibilityStatus::Compatible
        } else {
            CompatibilityStatus::LegacyUnknown
        },
        mismatch_axis: None,
    })
}

fn optional_compatible(
    flag: &'static str,
    actual: &ArtifactLineageState,
    required_flag: &'static str,
    expected: &ArtifactLineageState,
) -> Result<CompatibilityOutcome, PublicReportError> {
    if let Some((axis, actual_value, expected_value)) = scope_mismatch(actual, expected) {
        return Err(PublicReportError::OptionalInputIncompatible {
            flag,
            required_flag,
            axis,
            actual: actual_value,
            expected: expected_value,
        });
    }
    Ok(CompatibilityOutcome {
        status: if both_known(actual, expected) {
            CompatibilityStatus::Compatible
        } else {
            CompatibilityStatus::LegacyUnknown
        },
        mismatch_axis: None,
    })
}

fn both_known(left: &ArtifactLineageState, right: &ArtifactLineageState) -> bool {
    matches!(
        (left, right),
        (
            ArtifactLineageState::Known { .. },
            ArtifactLineageState::Known { .. }
        )
    )
}

/// The exact three-axis Phase-C scope predicate, with the first unequal axis
/// retained solely for the frozen public error vocabulary.  Acquisition
/// families are deliberately absent.
fn scope_mismatch(
    left: &ArtifactLineageState,
    right: &ArtifactLineageState,
) -> Option<(CompatibilityAxis, String, String)> {
    let (
        ArtifactLineageState::Known { identity: left, .. },
        ArtifactLineageState::Known {
            identity: right, ..
        },
    ) = (left, right)
    else {
        return None;
    };
    if left.experiment_scope != right.experiment_scope {
        return Some((
            CompatibilityAxis::ExperimentScope,
            scope_text(&left.experiment_scope),
            scope_text(&right.experiment_scope),
        ));
    }
    if left.sensor_scope != right.sensor_scope {
        return Some((
            CompatibilityAxis::SensorScope,
            scope_key_text(&left.sensor_scope),
            scope_key_text(&right.sensor_scope),
        ));
    }
    if left.channel_scope != right.channel_scope {
        return Some((
            CompatibilityAxis::ChannelScope,
            scope_key_text(&left.channel_scope),
            scope_key_text(&right.channel_scope),
        ));
    }
    None
}

fn scope_text(scope: &ArtifactExperimentScope) -> String {
    format!("{scope:?}")
}
fn scope_key_text(scope: &ScopeKey) -> String {
    format!("{scope:?}")
}
