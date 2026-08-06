use rust_electroanalysis_cli::{
    model_validation::evaluate_manifest,
    results::{ValidationDatasetCategory, ValidationExperiment, ValidationManifest},
    runners::model,
};
use std::{
    collections::BTreeMap,
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn validation_manifest_exports_reproducible_metrics_without_physical_claim() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("ism_validation_{}_{}", std::process::id(), nonce));
    let analysis = root.join("analysis");
    fs::create_dir_all(&root).expect("workspace");
    model::simulate(&root, None, Some(&analysis), 3, 1.0).expect("simulation");
    let manifest = ValidationManifest {
        schema_version: 1,
        study_id: "validation-study".into(),
        model_comparison_paths: vec![],
        experiments: vec![ValidationExperiment {
            experiment_id: "synthetic-step".into(),
            category: ValidationDatasetCategory::ConcentrationSteps,
            sensor_id: "s1".into(),
            analysis_path: "analysis/model_analysis.json".into(),
            reference_state_values: BTreeMap::new(),
            reference_parameter_values: BTreeMap::new(),
            expected_prediction_coverage: Some(0.95),
            is_real_experiment: false,
            calibration_transfer_group: Some("group-a".into()),
        }],
    };
    let results = evaluate_manifest(&manifest, &root).expect("evaluate");
    assert!(
        results
            .metrics
            .iter()
            .any(|metric| metric.metric == "contribution_reconstruction_error")
    );
    assert!(
        results
            .warnings
            .iter()
            .any(|warning| warning.contains("not marked as a real experiment"))
    );
    assert!(results.identifiability_report["limitations"].is_array());
    assert!(results.metrics.iter().any(|metric| {
        metric.metric == "prediction_coverage"
            && metric.value.is_none()
            && metric.evidence_status == "missing_prediction_intervals"
    }));
    assert!(results.metrics.iter().any(|metric| {
        metric.metric == "calibration_transfer_validation" && metric.value.is_none()
    }));
    fs::remove_dir_all(root).ok();
}
