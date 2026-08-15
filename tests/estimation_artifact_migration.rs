use rust_electroanalysis_cli::{
    domain::read_artifact,
    estimation::simulation::SimulationOutput,
    results::{StateEstimationReport, StateFilterComparison, StateValidationResult},
};
use std::{fs, path::PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/estimation_migration")
        .join(name)
}

#[test]
fn legacy_state_estimation_report_fixture_migrates() {
    let report: StateEstimationReport =
        read_artifact(&fixture("legacy_state_estimation_report_v1.json")).unwrap();
    assert_eq!(report.schema_version, 1);
    assert_eq!(report.model_backend, None);
    assert_eq!(report.model_profile, None);
    assert_eq!(report.model_id, None);
    assert_eq!(report.compiled_model_summary, None);
    assert!(report.state_bindings.is_empty());
    assert!(report.resolved_input_bindings.is_none());
    assert_eq!(report.ingestion_diagnostics.total_rows, 0);

    let first = serde_json::to_string(&report).unwrap();
    let second = serde_json::to_string(&report).unwrap();
    assert_eq!(first, second);
}

#[test]
fn legacy_simulation_truth_fixture_migrates() {
    let text = fs::read_to_string(fixture("legacy_simulation_truth_v2.json")).unwrap();
    let output: SimulationOutput = serde_json::from_str(&text).unwrap();
    assert_eq!(output.schema_version, 2);
    assert_eq!(
        output.scenario.model.backend,
        rust_electroanalysis_cli::estimation_config::EstimationModelBackend::Legacy
    );
    assert!(
        output
            .observations
            .iter()
            .all(|point| point.compiled.is_none())
    );
    assert!(output.scenario.model.definition.is_none());

    let first = serde_json::to_string(&output).unwrap();
    let second = serde_json::to_string(&output).unwrap();
    assert_eq!(first, second);
}

#[test]
fn legacy_validation_fixture_migrates() {
    let text = fs::read_to_string(fixture("legacy_state_validation_v1.json")).unwrap();
    let validation: StateValidationResult = serde_json::from_str(&text).unwrap();
    assert_eq!(
        validation.truth_source.as_deref(),
        Some("historical simulation truth")
    );
    assert!(validation.metrics.is_empty());
    assert!(validation.contribution_metrics.is_empty());
    assert_eq!(validation.matched_sample_count, 0);

    let first = serde_json::to_string(&validation).unwrap();
    let second = serde_json::to_string(&validation).unwrap();
    assert_eq!(first, second);
}

#[test]
fn legacy_filter_comparison_fixture_migrates() {
    let text = fs::read_to_string(fixture("legacy_state_filter_comparison_v2.json")).unwrap();
    let comparison: StateFilterComparison = serde_json::from_str(&text).unwrap();
    assert_eq!(comparison.schema_version, 2);
    assert_eq!(comparison.records.len(), 1);
    assert_eq!(comparison.records[0].model_backend, None);
    assert_eq!(comparison.records[0].model_profile, None);
    assert_eq!(comparison.ingestion_diagnostics.total_rows, 0);
    assert!(comparison.records[0].activity_rmse.is_none());

    let first = serde_json::to_string(&comparison).unwrap();
    let second = serde_json::to_string(&comparison).unwrap();
    assert_eq!(first, second);
}
