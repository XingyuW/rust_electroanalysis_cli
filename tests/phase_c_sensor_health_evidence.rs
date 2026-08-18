use rust_electroanalysis_cli::{
    cli::{CliError, CommandSpec, parse_cli_args},
    domain::read_artifact,
    health_config::PhaseCHealthEvidenceConfig,
    results::SensorHealthAssessment,
    runners::health,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_OUTPUT_ID: AtomicU64 = AtomicU64::new(0);

fn temporary_output_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let id = NEXT_OUTPUT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "phase_c_health_e2e_{}_{}_{}",
        std::process::id(),
        nonce,
        id
    ))
}

#[test]
fn phase_c_health_cli_rejects_phase_c_sources_without_config() {
    let args = vec![
        "electroanalysis".into(),
        "health".into(),
        "assess".into(),
        "--signal-results".into(),
        "signal.json".into(),
        "--estimation-artifact".into(),
        "estimation.json".into(),
    ];
    assert!(matches!(
        parse_cli_args(&args),
        Err(CliError::InvalidCombination(_))
    ));
}

#[test]
fn phase_c_health_cli_parses_exact_optional_artifact_flags() {
    let args = vec![
        "electroanalysis".into(),
        "health".into(),
        "assess".into(),
        "--signal-results".into(),
        "signal.json".into(),
        "--phase-c-config".into(),
        "phase_c.toml".into(),
        "--estimation-artifact".into(),
        "estimation.json".into(),
        "--model-artifact".into(),
        "model.json".into(),
        "--mechanism-artifact".into(),
        "mechanism.json".into(),
        "--lineage-catalog".into(),
        "catalog.json".into(),
    ];
    let parsed = parse_cli_args(&args).expect("valid Phase-C CLI invocation");
    assert!(matches!(
        parsed.command,
        Some(CommandSpec::HealthAssess {
            phase_c_config: Some(_),
            estimation_artifact: Some(_),
            model_artifact: Some(_),
            mechanism_artifact: Some(_),
            lineage_catalog: Some(_),
            ..
        })
    ));
}

#[test]
fn phase_c_health_cli_does_not_accept_state_estimation_alias() {
    let args = vec![
        "electroanalysis".into(),
        "health".into(),
        "assess".into(),
        "--signal-results".into(),
        "signal.json".into(),
        "--phase-c-config".into(),
        "phase_c.toml".into(),
        "--state-estimation-artifact".into(),
        "estimation.json".into(),
    ];
    assert!(parse_cli_args(&args).is_err());
}

#[test]
fn phase_c_config_fixture_is_strict_and_fingerprinted_from_raw_bytes() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_c/config/valid_phase_c.toml");
    let loaded = PhaseCHealthEvidenceConfig::load(&path).expect("valid strict Phase-C config");
    assert_eq!(loaded.config.schema_version, 1);
    assert_eq!(loaded.config_sha256.len(), 64);
    assert!(
        loaded
            .config_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    );
}

#[test]
fn phase_c_configured_runner_writes_schema4_assessment() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let signal =
        root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json");
    let config = root.join("tests/fixtures/phase_c/config/valid_phase_c.toml");
    let output = temporary_output_dir();
    health::assess(
        &root,
        &signal,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("configured Phase-C runner succeeds");
    let assessment: SensorHealthAssessment =
        read_artifact(&output.join("health_assessment.json")).expect("schema-4 assessment");
    assert_eq!(assessment.schema_version, 4);
    let phase_c = assessment.phase_c.as_ref().expect("Phase-C report");
    assert_eq!(phase_c.dimension_assessments.len(), 9);
    let thresholded_signal = phase_c
        .evidence_bundle
        .records
        .iter()
        .find(|record| record.evidence_id.0 == "signal.descriptive.rms")
        .expect("signal threshold evidence");
    assert!(
        thresholded_signal
            .threshold_provenance
            .iter()
            .all(
                |threshold| threshold.configuration_hash.as_deref() == Some(&phase_c.config_sha256)
            )
    );
    std::fs::remove_dir_all(output).expect("remove test output");
}
