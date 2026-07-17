use rust_electroanalysis_cli::data_file::EISData;
use rust_electroanalysis_cli::domain::AnalysisProvenance;
use rust_electroanalysis_cli::impedance::parse_circuit_string;
use rust_electroanalysis_cli::mechanism::{
    calculate_trend, compare_timescales, extract_eis_timescales,
};
use rust_electroanalysis_cli::results::{
    CharacteristicTimescale, CircuitFitResult, EisFitArtifact, EvidenceLevel,
    MechanismRecordSummary, MechanismWarning, ResolvedMechanismConfig, TimescaleDerivation,
    TimescaleSource, TimescaleValidity,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn provenance() -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "test".to_string(),
        input_path: PathBuf::from("synthetic.csv"),
        input_sha256: "0".repeat(64),
        configuration_path: None,
        configuration_sha256: None,
        generation_timestamp: 1,
        git_commit: None,
    }
}

fn artifact(circuit: &str, params: Vec<f64>) -> EisFitArtifact {
    let node = parse_circuit_string(circuit).expect("circuit");
    let names = node.get_param_names();
    let units = node.get_param_units();
    let input = EISData {
        date: String::new(),
        test_type: "EIS".to_string(),
        instrument_model: String::new(),
        freq: vec![1.0, 10.0, 100.0, 1000.0],
        phase: vec![-1.0; 4],
        z_re: vec![1.0; 4],
        z_im: vec![-1.0; 4],
        label: "synthetic".to_string(),
        metadata: BTreeMap::new(),
        circuit_model: circuit.to_string(),
    };
    let fit = CircuitFitResult {
        fitted_parameters: params.clone(),
        parameter_names: names,
        parameter_units: units,
        fitted_z_re: vec![1.0; 4],
        fitted_z_im: vec![-1.0; 4],
        fitted_magnitude: vec![2.0_f64.sqrt(); 4],
        fitted_phase: vec![-45.0; 4],
    };
    EisFitArtifact::from_fit(&input, circuit, &fit, provenance())
}

#[test]
fn extracts_ideal_parallel_rc_without_cross_branch_pairing() {
    let values = extract_eis_timescales(
        &artifact("R0-p(C1,R1)", vec![5.0, 2.0e-5, 100.0]),
        0.95,
        0.1,
    );
    let rc = values
        .iter()
        .find(|t| t.label.contains("R-C"))
        .expect("RC timescale");
    assert!((rc.value_s - 0.002).abs() < 1e-12);
    assert!(rc.derivation.equation.contains("R*C"));
    assert!(
        rc.derivation
            .circuit_path
            .as_deref()
            .unwrap_or_default()
            .contains("parallel")
    );
}

#[test]
fn extracts_cpe_timescale_using_repository_convention() {
    let values = extract_eis_timescales(
        &artifact("R0-p(CPE1,R1)", vec![5.0, 1.0e-5, 0.8, 100.0]),
        0.95,
        0.1,
    );
    let cpe = values
        .iter()
        .find(|t| t.label.contains("CPE"))
        .expect("CPE timescale");
    let expected = (100.0 * 1.0e-5_f64).powf(1.0 / 0.8);
    assert!((cpe.value_s - expected).abs() < 1e-12);
    assert_eq!(
        cpe.derivation.convention.as_deref(),
        Some("Z_CPE = 1/(Q*(jω)^alpha)")
    );
}

#[test]
fn nested_parallel_branches_produce_two_distinct_timescales() {
    let values = extract_eis_timescales(
        &artifact("p(R0,C0)-p(R1,C1)", vec![2.0, 3.0, 4.0, 5.0]),
        0.95,
        0.1,
    );
    let rc = values
        .iter()
        .filter(|t| t.label.contains("R-C"))
        .collect::<Vec<_>>();
    assert_eq!(rc.len(), 2);
    assert_ne!(rc[0].derivation.circuit_path, rc[1].derivation.circuit_path);
}

#[test]
fn unsupported_topology_does_not_create_arbitrary_timescale() {
    let values = extract_eis_timescales(&artifact("R0-W1", vec![5.0, 2.0]), 0.95, 0.1);
    assert!(values.is_empty());
}

#[test]
fn durable_artifact_is_serializable_and_nonfinite_safe() {
    let artifact = artifact("R0-p(C1,R1)", vec![5.0, 2.0e-5, 100.0]);
    let json = serde_json::to_string(&artifact).expect("artifact JSON");
    assert!(artifact.validate_finite());
    assert!(!json.contains("NaN"));
    assert_eq!(artifact.parameters[0].element_id, "R0");
    assert_eq!(artifact.statistics.valid_frequency_points, 4);
}

#[test]
fn covariance_term_changes_rc_uncertainty() {
    let gradient = [2.0, 100.0];
    let independent = rust_electroanalysis_cli::mechanism::uncertainty::delta_variance(
        &gradient,
        Some(&[vec![0.25, 0.0], vec![0.0, 0.000004]]),
    )
    .expect("variance");
    let correlated = rust_electroanalysis_cli::mechanism::uncertainty::delta_variance(
        &gradient,
        Some(&[vec![0.25, 0.0005], vec![0.0005, 0.000004]]),
    )
    .expect("variance");
    assert!(correlated > independent);
}

fn timescale(id: &str, value: f64) -> CharacteristicTimescale {
    CharacteristicTimescale {
        timescale_id: id.to_string(),
        source: TimescaleSource::EisCircuit,
        label: id.to_string(),
        value_s: value,
        standard_error_s: Some(value * 0.05),
        confidence_interval_s: Some((value * 0.9, value * 1.1)),
        derivation: TimescaleDerivation {
            equation: "test".to_string(),
            circuit_path: None,
            convention: None,
        },
        source_parameters: vec![],
        semantic_role: None,
        validity: TimescaleValidity::Valid,
        warnings: vec![],
    }
}

#[test]
fn comparison_is_transparent_and_never_claims_proof() {
    let config = ResolvedMechanismConfig::default();
    let comparison = compare_timescales(
        "r1",
        &timescale("eis", 1.0),
        &timescale("transient", 1.2),
        &config,
    );
    assert!(matches!(comparison.evidence_level, EvidenceLevel::Strong));
    assert!(!comparison.alternative_explanations.is_empty());
    assert!(
        comparison
            .assumptions
            .iter()
            .any(|a| a.contains("not mechanism proof"))
    );
}

#[test]
fn contradictory_timescale_is_classified_without_degradation_claim() {
    let config = ResolvedMechanismConfig::default();
    let comparison = compare_timescales(
        "r1",
        &timescale("eis", 1.0),
        &timescale("transient", 100.0),
        &config,
    );
    assert_eq!(comparison.evidence_level, EvidenceLevel::Contradictory);
    assert!(!comparison.contradictory_evidence.is_empty());
}

#[test]
fn trend_requires_multiple_records_and_remains_descriptive() {
    let records = (0..3)
        .map(|i| MechanismRecordSummary {
            record_id: format!("r{i}"),
            experiment_id: Some(format!("e{i}")),
            sensor_id: Some("s".to_string()),
            condition: None,
            sensor_age_days: Some(i as f64),
            metadata: BTreeMap::new(),
            calibration_context_available: false,
            warnings: vec![MechanismWarning {
                kind: "calibration_context_unavailable".to_string(),
                message: "absent".to_string(),
            }],
        })
        .collect::<Vec<_>>();
    let trend = calculate_trend(
        "tau",
        &records,
        &[
            ("r0".to_string(), 1.0),
            ("r1".to_string(), 2.0),
            ("r2".to_string(), 3.0),
        ],
        "sensor_age_days",
        3,
    );
    assert_eq!(trend.records, 3);
    assert!(trend.slope.unwrap() > 0.0);
    assert!(trend.warnings.is_empty());
}
