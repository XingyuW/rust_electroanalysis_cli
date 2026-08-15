use rust_electroanalysis_cli::{
    ArtifactKind, CurrentArtifactKindPolicy, VersionedArtifact,
    domain::{ArtifactError, read_artifact, write_artifact},
    results::{
        CalibrationAnalysisReport, CalibrationObservationSet, EisFitArtifact, HealthTrendReport,
        MechanismAnalysisReport, ModelAnalysisReport, ModelCompilationArtifact,
        SensorHealthAssessment, SensorHealthBaseline, SignalAnalysisReport, StateEstimationReport,
        StoredCalibrationModel, TransientAnalysisReport, ValidationResults,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_PATH_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct FixtureArtifact {
    schema_version: u32,
    value: f64,
}
impl VersionedArtifact for FixtureArtifact {
    const ARTIFACT_KIND: ArtifactKind = ArtifactKind::SignalAnalysis;
    const CURRENT_SCHEMA_VERSION: u32 = 2;
    const LEGACY_SCHEMA_VERSIONS: &'static [u32] = &[1];
    const CURRENT_ARTIFACT_KIND_POLICY: CurrentArtifactKindPolicy =
        CurrentArtifactKindPolicy::Required;
    fn schema_version(&self) -> u32 {
        self.schema_version
    }
    fn validate_before_json(&self) -> Result<(), ArtifactError> {
        rust_electroanalysis_cli::domain::validate_serialized_finite(self)
    }
}

fn path(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let id = NEXT_PATH_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "artifact_{name}_{}_{}_{}.json",
        std::process::id(),
        nonce,
        id
    ))
}

#[test]
fn writer_stamps_kind_and_reader_validates_it() {
    let path = path("roundtrip");
    let artifact = FixtureArtifact {
        schema_version: 2,
        value: 1.0,
    };
    write_artifact(&path, &artifact).expect("write");
    let text = fs::read_to_string(&path).expect("read");
    assert!(text.contains("\"artifact_kind\": \"signal_analysis\""));
    assert_eq!(read_artifact::<FixtureArtifact>(&path).unwrap(), artifact);
    fs::remove_file(path).ok();
}

#[test]
fn incompatible_kind_and_future_schema_are_typed_errors() {
    let path = path("reject");
    fs::write(
        &path,
        r#"{"schema_version":2,"artifact_kind":"eis_fit","value":1.0}"#,
    )
    .unwrap();
    assert!(matches!(
        read_artifact::<FixtureArtifact>(&path),
        Err(ArtifactError::IncompatibleKind { .. })
    ));
    fs::write(
        &path,
        r#"{"schema_version":99,"artifact_kind":"signal_analysis","value":1.0}"#,
    )
    .unwrap();
    assert!(matches!(
        read_artifact::<FixtureArtifact>(&path),
        Err(ArtifactError::UnsupportedSchemaVersion { .. })
    ));
    fs::remove_file(path).ok();
}

#[test]
fn previous_schema_without_kind_migrates_only_through_typed_contract() {
    let path = path("legacy");
    fs::write(&path, r#"{"schema_version":1,"value":1.0}"#).unwrap();
    assert_eq!(
        read_artifact::<FixtureArtifact>(&path)
            .unwrap()
            .schema_version,
        1
    );
    fs::remove_file(path).ok();
}

#[test]
fn every_exported_cross_workflow_json_type_declares_a_contract() {
    fn check<T: VersionedArtifact>() {
        assert!(!T::ARTIFACT_KIND.as_str().is_empty());
    }
    check::<EisFitArtifact>();
    check::<TransientAnalysisReport>();
    check::<CalibrationObservationSet>();
    check::<StoredCalibrationModel>();
    check::<CalibrationAnalysisReport>();
    check::<SignalAnalysisReport>();
    check::<SensorHealthBaseline>();
    check::<SensorHealthAssessment>();
    check::<HealthTrendReport>();
    check::<MechanismAnalysisReport>();
    check::<StateEstimationReport>();
    check::<ModelCompilationArtifact>();
    check::<ModelAnalysisReport>();
    check::<ValidationResults>();
}

fn assert_current_rejections<T: VersionedArtifact>() {
    let path = path("matrix");
    let current = T::CURRENT_SCHEMA_VERSION;
    let unsupported = T::CURRENT_SCHEMA_VERSION.saturating_add(1);
    fs::write(
        &path,
        format!(
            r#"{{"schema_version":{current},"artifact_kind":"{}"}}"#,
            "eis_fit"
        ),
    )
    .unwrap();
    assert!(matches!(
        read_artifact::<T>(&path),
        Err(ArtifactError::IncompatibleKind {
            actual: Some(_),
            ..
        })
    ));
    fs::write(&path, format!(r#"{{"schema_version":{current}}}"#)).unwrap();
    assert!(matches!(
        read_artifact::<T>(&path),
        Err(ArtifactError::IncompatibleKind { actual: None, .. })
    ));
    fs::write(
        &path,
        format!(r#"{{"schema_version":{unsupported},"artifact_kind":"eis_fit"}}"#),
    )
    .unwrap();
    assert!(matches!(
        read_artifact::<T>(&path),
        Err(ArtifactError::UnsupportedSchemaVersion { .. })
    ));
    fs::remove_file(path).ok();
}

#[test]
fn mhi_t02b_current_wrong_kind() {
    assert_current_rejections::<TransientAnalysisReport>();
    assert_current_rejections::<CalibrationObservationSet>();
    assert_current_rejections::<StoredCalibrationModel>();
    assert_current_rejections::<CalibrationAnalysisReport>();
    assert_current_rejections::<SignalAnalysisReport>();
    assert_current_rejections::<MechanismAnalysisReport>();
    assert_current_rejections::<SensorHealthAssessment>();
    assert_current_rejections::<HealthTrendReport>();
}

#[test]
fn mhi_t02c_current_missing_kind() {
    assert_current_rejections::<TransientAnalysisReport>();
    assert_current_rejections::<CalibrationObservationSet>();
    assert_current_rejections::<StoredCalibrationModel>();
    assert_current_rejections::<CalibrationAnalysisReport>();
    assert_current_rejections::<SignalAnalysisReport>();
    assert_current_rejections::<MechanismAnalysisReport>();
    assert_current_rejections::<SensorHealthAssessment>();
    assert_current_rejections::<HealthTrendReport>();
}

#[test]
fn mhi_t02e_unsupported() {
    assert_current_rejections::<TransientAnalysisReport>();
    assert_current_rejections::<CalibrationObservationSet>();
    assert_current_rejections::<StoredCalibrationModel>();
    assert_current_rejections::<CalibrationAnalysisReport>();
    assert_current_rejections::<SignalAnalysisReport>();
    assert_current_rejections::<MechanismAnalysisReport>();
    assert_current_rejections::<SensorHealthAssessment>();
    assert_current_rejections::<HealthTrendReport>();
}

#[test]
fn a0_ac_compat_01_preserves_eis_fit_and_health_baseline_matrices() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/a0_artifact_contracts");
    let preserved =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/artifact_contracts");
    let eis_missing = preserved.join("eis_fit_schema2_missing_kind.json");
    let eis_correct = root.join("eis_fit_schema2_correct_kind.json");
    let eis_wrong = root.join("eis_fit_schema2_wrong_kind.json");
    assert!(read_artifact::<EisFitArtifact>(&eis_missing).is_ok());
    assert!(read_artifact::<EisFitArtifact>(&eis_correct).is_ok());
    assert!(matches!(
        read_artifact::<EisFitArtifact>(&eis_wrong),
        Err(ArtifactError::IncompatibleKind {
            actual: Some(_),
            ..
        })
    ));

    let baseline_missing = preserved.join("health_baseline_schema2_missing_kind.json");
    let baseline_correct = root.join("health_baseline_schema2_correct_kind.json");
    let baseline_wrong = root.join("health_baseline_schema2_wrong_kind.json");
    assert!(read_artifact::<SensorHealthBaseline>(&baseline_missing).is_ok());
    assert!(read_artifact::<SensorHealthBaseline>(&baseline_correct).is_ok());
    assert!(matches!(
        read_artifact::<SensorHealthBaseline>(&baseline_wrong),
        Err(ArtifactError::IncompatibleKind {
            actual: Some(_),
            ..
        })
    ));
}
