use rust_electroanalysis_cli::{
    ArtifactKind, VersionedArtifact,
    domain::{ArtifactError, read_artifact, write_artifact},
    results::{
        CalibrationAnalysisReport, CalibrationObservationSet, EisFitArtifact, HealthTrendReport,
        MechanismAnalysisReport, SensorHealthAssessment, SensorHealthBaseline,
        SignalAnalysisReport, StateEstimationReport, StoredCalibrationModel,
        TransientAnalysisReport,
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
        "rust_electroanalysis_{name}_{}_{}.json",
        std::process::id(),
        nonce
    ))
}

#[test]
fn artifact_writer_stamps_semantic_header_and_reader_round_trips() {
    let path = path("artifact");
    let input = FixtureArtifact {
        schema_version: 2,
        value: 1.0,
    };
    write_artifact(&path, &input).expect("write artifact");
    let text = fs::read_to_string(&path).expect("read artifact");
    assert!(text.contains("\"artifact_kind\": \"signal_analysis\""));
    assert_eq!(
        read_artifact::<FixtureArtifact>(&path).expect("read artifact"),
        input
    );
    fs::remove_file(path).expect("remove artifact");
}

#[test]
fn incompatible_kind_and_schema_are_typed_errors() {
    let path = path("rejected");
    fs::write(
        &path,
        r#"{"schema_version":2,"artifact_kind":"eis_fit","value":1.0}"#,
    )
    .expect("write fixture");
    assert!(matches!(
        read_artifact::<FixtureArtifact>(&path),
        Err(ArtifactError::IncompatibleKind { .. })
    ));
    fs::write(
        &path,
        r#"{"schema_version":99,"artifact_kind":"signal_analysis","value":1.0}"#,
    )
    .expect("write fixture");
    assert!(matches!(
        read_artifact::<FixtureArtifact>(&path),
        Err(ArtifactError::UnsupportedSchemaVersion { .. })
    ));
    fs::remove_file(path).expect("remove artifact");
}

#[test]
fn prior_schema_without_kind_migrates_only_through_typed_contract() {
    let path = path("legacy");
    fs::write(&path, r#"{"schema_version":1,"value":1.0}"#).expect("write legacy fixture");
    assert_eq!(
        read_artifact::<FixtureArtifact>(&path)
            .expect("migrate legacy artifact")
            .schema_version,
        1
    );
    fs::remove_file(path).expect("remove artifact");
}

#[test]
fn every_cross_workflow_artifact_has_a_declared_semantic_contract() {
    fn assert_contract<T: VersionedArtifact>() {
        assert!(!T::ARTIFACT_KIND.as_str().is_empty());
    }
    assert_contract::<EisFitArtifact>();
    assert_contract::<TransientAnalysisReport>();
    assert_contract::<CalibrationObservationSet>();
    assert_contract::<StoredCalibrationModel>();
    assert_contract::<CalibrationAnalysisReport>();
    assert_contract::<SignalAnalysisReport>();
    assert_contract::<SensorHealthBaseline>();
    assert_contract::<SensorHealthAssessment>();
    assert_contract::<HealthTrendReport>();
    assert_contract::<MechanismAnalysisReport>();
    assert_contract::<StateEstimationReport>();
}
