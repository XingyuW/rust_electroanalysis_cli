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
    time::{SystemTime, UNIX_EPOCH},
};

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
    std::env::temp_dir().join(format!(
        "artifact_{name}_{}_{nonce}.json",
        std::process::id()
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

fn assert_current_contract<T: VersionedArtifact>(kind: &str) {
    assert_eq!(T::CURRENT_SCHEMA_VERSION, 2);
    assert_eq!(T::LEGACY_SCHEMA_VERSIONS, &[1]);
    assert_eq!(T::ARTIFACT_KIND.as_str(), kind);
    assert_eq!(
        T::CURRENT_ARTIFACT_KIND_POLICY,
        CurrentArtifactKindPolicy::Required
    );
}

fn assert_current_rejections<T: VersionedArtifact>() {
    let path = path("matrix");
    fs::write(
        &path,
        format!(r#"{{"schema_version":2,"artifact_kind":"{}"}}"#, "eis_fit"),
    )
    .unwrap();
    assert!(matches!(
        read_artifact::<T>(&path),
        Err(ArtifactError::IncompatibleKind {
            actual: Some(_),
            ..
        })
    ));
    fs::write(&path, r#"{"schema_version":2}"#).unwrap();
    assert!(matches!(
        read_artifact::<T>(&path),
        Err(ArtifactError::IncompatibleKind { actual: None, .. })
    ));
    fs::write(&path, r#"{"schema_version":99,"artifact_kind":"eis_fit"}"#).unwrap();
    assert!(matches!(
        read_artifact::<T>(&path),
        Err(ArtifactError::UnsupportedSchemaVersion { .. })
    ));
    fs::remove_file(path).ok();
}

#[test]
fn mhi_t02a_current_correct_kind() {
    assert_current_contract::<TransientAnalysisReport>("transient_analysis");
    assert_current_contract::<CalibrationObservationSet>("calibration_observations");
    assert_current_contract::<StoredCalibrationModel>("calibration_model");
    assert_current_contract::<CalibrationAnalysisReport>("calibration_analysis");
    assert_current_contract::<SignalAnalysisReport>("signal_analysis");
    assert_current_contract::<MechanismAnalysisReport>("mechanism_analysis");
    assert_current_contract::<SensorHealthAssessment>("health_assessment");
    assert_current_contract::<HealthTrendReport>("health_trend");
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
    let eis_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/artifact_contracts/eis_fit_schema2_missing_kind.json");
    let baseline_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/artifact_contracts/health_baseline_schema2_missing_kind.json");
    assert!(read_artifact::<EisFitArtifact>(&eis_path).is_ok());
    assert!(read_artifact::<SensorHealthBaseline>(&baseline_path).is_ok());

    for (path, kind) in [(&eis_path, "eis_fit"), (&baseline_path, "health_baseline")] {
        let mut value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        value["artifact_kind"] = serde_json::Value::String(kind.into());
        fs::write(path.with_extension("correct.json"), value.to_string()).unwrap();
        let correct_path = path.with_extension("correct.json");
        if kind == "eis_fit" {
            assert!(read_artifact::<EisFitArtifact>(&correct_path).is_ok());
        } else {
            assert!(read_artifact::<SensorHealthBaseline>(&correct_path).is_ok());
        }
        let mut wrong: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        wrong["artifact_kind"] = serde_json::Value::String("signal_analysis".into());
        fs::write(path.with_extension("wrong.json"), wrong.to_string()).unwrap();
        let wrong_path = path.with_extension("wrong.json");
        if kind == "eis_fit" {
            assert!(matches!(
                read_artifact::<EisFitArtifact>(&wrong_path),
                Err(ArtifactError::IncompatibleKind {
                    actual: Some(_),
                    ..
                })
            ));
        } else {
            assert!(matches!(
                read_artifact::<SensorHealthBaseline>(&wrong_path),
                Err(ArtifactError::IncompatibleKind {
                    actual: Some(_),
                    ..
                })
            ));
        }
        fs::remove_file(correct_path).ok();
        fs::remove_file(wrong_path).ok();
    }
}
