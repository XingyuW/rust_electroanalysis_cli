use rust_electroanalysis_cli::{
    ArtifactKind, VersionedArtifact,
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
    fn schema_version(&self) -> u32 {
        self.schema_version
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
