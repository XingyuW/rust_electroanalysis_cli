use rust_electroanalysis_cli::{
    ArtifactKind, CurrentArtifactKindPolicy, VersionedArtifact,
    domain::{
        ArtifactError, ArtifactLineageState, UnknownLineageReason, read_artifact, write_artifact,
    },
    results::{
        CalibrationAnalysisReport, CalibrationObservationSet, EisFitArtifact, HealthTrendReport,
        MechanismAnalysisReport, ModelAnalysisReport, ModelCompilationArtifact,
        OverallHealthStatus, SensorHealthAssessment, SensorHealthBaseline, SignalAnalysisReport,
        StateEstimationReport, StoredCalibrationModel, TransientAnalysisReport, ValidationResults,
    },
    runners::health,
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

fn configured_phase_c_assessment() -> SensorHealthAssessment {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = path("phase_c_schema4_source");
    health::assess(
        &root,
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&root.join("tests/fixtures/phase_c/config/valid_phase_c.toml")),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("configured Phase-C assessment");
    let artifact = read_artifact(&output.join("health_assessment.json")).expect("reader");
    fs::remove_dir_all(output).expect("remove output");
    artifact
}

fn write_health_value(name: &str, value: &serde_json::Value) -> PathBuf {
    let output = path(name);
    let mut value = value.clone();
    value
        .as_object_mut()
        .expect("health artifact object")
        .entry("artifact_kind")
        .or_insert_with(|| serde_json::Value::String("health_assessment".into()));
    fs::write(
        &output,
        serde_json::to_vec(&value).expect("serialize mutation"),
    )
    .expect("write");
    output
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

#[test]
fn phase_c_legacy_schema3_health_artifact_remains_readable() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_c/writer_boundary/legacy_health_assessment_v3.json");
    let wire: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture).expect("read independent wire fixture"))
            .expect("parse independent wire fixture");
    assert_eq!(wire["artifact_kind"], "health_assessment");
    assert_eq!(wire["schema_version"], 3);
    for absent in ["phase_c", "lineage", "artifact_id", "dependencies"] {
        assert!(
            wire.get(absent).is_none(),
            "the independently checked-in schema-3 fixture must not invent {absent}"
        );
    }
    let assessment: SensorHealthAssessment = read_artifact(&fixture).expect("read legacy fixture");
    assert_eq!(assessment.schema_version, 3);
    assert_eq!(assessment.assessment_id, "phase-c-legacy-schema3-fixture");
    assert_eq!(
        assessment.experiment_id.as_deref(),
        Some("phase-c-legacy-experiment")
    );
    assert_eq!(assessment.sensor_id, None);
    assert_eq!(
        assessment.overall_status,
        OverallHealthStatus::DataQualityInsufficient
    );
    assert_eq!(assessment.features.len(), 1);
    assert_eq!(assessment.features[0].name, "signal.mean");
    assert_eq!(assessment.features[0].value, Some(0.2));
    assert_eq!(assessment.features[0].unit, "V");
    assert_eq!(assessment.phase_c, None);
    assert_eq!(
        assessment.lineage,
        ArtifactLineageState::LegacyUnknown {
            source_schema_version: None,
            reason: UnknownLineageReason::FieldAbsentInLegacyArtifact,
        }
    );
}

#[test]
fn phase_c_canonical_health_writer_never_emits_schema3() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_c/writer_boundary/legacy_health_assessment_v3.json");
    let assessment: SensorHealthAssessment = read_artifact(&fixture).expect("read legacy fixture");
    assert_eq!(assessment.schema_version, 3);
    assert_eq!(assessment.phase_c, None);
    let output = path("phase_c_canonical_writer");
    let result = write_artifact(&output, &assessment);
    assert_eq!(
        result.unwrap_err().to_string(),
        "artifact schema validation failed: schema-4 health assessment requires a non-null phase_c"
    );
    assert!(!output.exists());
}

#[test]
fn phase_c_schema4_rejects_missing_or_null_phase_c() {
    let canonical = configured_phase_c_assessment();
    let canonical_path = path("phase_c_complete_schema4_wire");
    write_artifact(&canonical_path, &canonical).expect("write valid schema-4 wire");
    let complete_wire: serde_json::Value =
        serde_json::from_slice(&fs::read(&canonical_path).expect("read canonical wire"))
            .expect("parse canonical wire");
    fs::remove_file(&canonical_path).expect("remove canonical wire");
    for (case, phase_c) in [("missing", None), ("null", Some(serde_json::Value::Null))] {
        let mut value = complete_wire.clone();
        let object = value.as_object_mut().expect("health artifact object");
        if let Some(phase_c) = phase_c {
            object.insert("phase_c".into(), phase_c);
        } else {
            object.remove("phase_c");
        }
        let output = path(&format!("phase_c_{case}_report"));
        fs::write(
            &output,
            serde_json::to_vec(&value).expect("serialize mutation"),
        )
        .expect("write mutation");
        assert_eq!(
            read_artifact::<SensorHealthAssessment>(&output)
                .expect_err("schema-4 must reject a missing or null report")
                .to_string(),
            "artifact schema validation failed: schema-4 health assessment requires a non-null phase_c"
        );
        assert!(
            output.exists(),
            "reader failures must not mutate the input fixture"
        );
        fs::remove_file(output).ok();
    }
}

#[test]
fn phase_c_schema3_health_assessment_remains_readable() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_c/writer_boundary/legacy_health_assessment_v3.json");
    let assessment: SensorHealthAssessment =
        read_artifact(&fixture).expect("read schema-3 fixture");
    assert_eq!(assessment.schema_version, 3);
    assert_eq!(assessment.assessment_id, "phase-c-legacy-schema3-fixture");
    assert_eq!(
        assessment.experiment_id.as_deref(),
        Some("phase-c-legacy-experiment")
    );
    assert_eq!(
        assessment.overall_status,
        OverallHealthStatus::DataQualityInsufficient
    );
    assert_eq!(assessment.phase_c, None);
    assert_eq!(assessment.features[0].name, "signal.mean");
    assert_eq!(assessment.features[0].value, Some(0.2));
    assert_eq!(
        assessment.lineage,
        ArtifactLineageState::LegacyUnknown {
            source_schema_version: None,
            reason: UnknownLineageReason::FieldAbsentInLegacyArtifact,
        }
    );
}

#[test]
fn phase_c_schema4_requires_complete_nine_dimension_report() {
    let assessment = configured_phase_c_assessment();
    for (case, mutate) in [("missing", false), ("duplicate", true)] {
        let mut value = serde_json::to_value(&assessment).expect("serialize current artifact");
        let rows = value["phase_c"]["dimension_assessments"]
            .as_array_mut()
            .expect("nine-row Phase-C report");
        if mutate {
            rows.push(rows[0].clone());
        } else {
            rows.pop();
        }
        let output = write_health_value(&format!("phase_c_{case}_dimension"), &value);
        assert_eq!(
            read_artifact::<SensorHealthAssessment>(&output)
                .expect_err("a schema-4 report requires exactly one of every dimension")
                .to_string(),
            "artifact schema validation failed: schema-4 health assessment requires exactly one record for each health dimension"
        );
        fs::remove_file(output).ok();
    }
}

#[test]
fn phase_c_schema4_roundtrip_preserves_wire_contract() {
    let assessment = configured_phase_c_assessment();
    let output = path("phase_c_schema4_roundtrip");
    write_artifact(&output, &assessment).expect("canonical writer accepts complete Phase-C report");
    let raw: serde_json::Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
    assert_eq!(raw["schema_version"], 4);
    assert_eq!(raw["artifact_kind"], "health_assessment");
    assert!(raw["phase_c"].is_object());
    assert!(raw.get("phase_c_report").is_none());
    assert_eq!(
        raw["phase_c"]["dimension_assessments"]
            .as_array()
            .map(Vec::len),
        Some(9)
    );
    assert_eq!(
        read_artifact::<SensorHealthAssessment>(&output).expect("public reread"),
        assessment
    );
    fs::remove_file(output).ok();
}

#[test]
fn phase_c_schema4_rejects_wrong_kind_missing_kind_and_future_version() {
    let assessment = configured_phase_c_assessment();
    for (name, mutate) in [
        ("wrong_kind", 0_u8),
        ("missing_kind", 1_u8),
        ("future_version", 2_u8),
    ] {
        let mut value = serde_json::to_value(&assessment).expect("serialize current artifact");
        match mutate {
            0 => value["artifact_kind"] = "signal_analysis".into(),
            1 => {
                value
                    .as_object_mut()
                    .expect("object")
                    .remove("artifact_kind");
            }
            _ => value["schema_version"] = 5.into(),
        }
        let output = if mutate == 1 {
            let output = path(name);
            fs::write(
                &output,
                serde_json::to_vec(&value).expect("serialize mutation"),
            )
            .expect("write missing-kind mutation");
            output
        } else {
            write_health_value(name, &value)
        };
        match mutate {
            0 => assert!(matches!(
                read_artifact::<SensorHealthAssessment>(&output),
                Err(ArtifactError::IncompatibleKind { expected: ArtifactKind::HealthAssessment, actual: Some(actual), .. }) if actual == "signal_analysis"
            )),
            1 => assert!(matches!(
                read_artifact::<SensorHealthAssessment>(&output),
                Err(ArtifactError::IncompatibleKind {
                    expected: ArtifactKind::HealthAssessment,
                    actual: None,
                    ..
                })
            )),
            _ => assert!(matches!(
                read_artifact::<SensorHealthAssessment>(&output),
                Err(ArtifactError::UnsupportedSchemaVersion {
                    expected: ArtifactKind::HealthAssessment,
                    actual: 5,
                    ..
                })
            )),
        }
        fs::remove_file(output).ok();
    }
}

#[test]
fn phase_c_schema4_rejects_retired_phase_c_aliases() {
    let assessment = configured_phase_c_assessment();
    let mut value = serde_json::to_value(&assessment).expect("serialize current artifact");
    value["phase_c"]["phase_c_report"] = value["phase_c"].clone();
    let output = write_health_value("phase_c_retired_alias", &value);
    let result = read_artifact::<SensorHealthAssessment>(&output)
        .expect_err("schema-4 must reject a retired Phase-C alias");
    assert!(
        matches!(result, ArtifactError::Validation { ref message } if message.contains("unknown field `phase_c_report`")),
        "{result:?}"
    );
    fs::remove_file(output).ok();
}
