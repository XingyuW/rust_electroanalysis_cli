use rust_electroanalysis_cli::{
    cli::{CliError, CommandSpec, parse_cli_args},
    domain::{read_artifact, write_artifact},
    health_config::PhaseCHealthEvidenceConfig,
    results::{
        DriftModelKind, HealthDimension, HealthEvidenceState, OverallHealthStatus,
        PhaseCHealthReasonCode, SensorHealthAssessment, SignalAnalysisReport,
    },
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

/// Exercise the public Phase-C route from a stable legacy-compatible signal
/// artifact.  Individual tests below select their named output row and assert
/// its exact status/reason pair; no test reaches into the crate-private
/// evaluator.
fn base_phase_c_assessment() -> SensorHealthAssessment {
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
    .expect("configured Phase-C public runner succeeds");
    let assessment =
        read_artifact(&output.join("health_assessment.json")).expect("schema-4 assessment");
    std::fs::remove_dir_all(output).expect("remove test output");
    assessment
}

/// PC-FX-01 is deliberately assembled through the public artifact writer and
/// the public health runner.  The legacy A0 fixture is only a complete source
/// shape; every health-relevant value is overwritten below so a test cannot
/// accidentally inherit its old, critical signal metrics.
fn pc_fx_01_assessment(mutate: impl FnOnce(&mut SignalAnalysisReport)) -> SensorHealthAssessment {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut signal: SignalAnalysisReport = read_artifact(
        &root.join("tests/fixtures/a0_artifact_contracts/schema1/signal_analysis.schema1.json"),
    )
    .expect("read source shape");
    signal.unit = "V".into();
    signal.descriptive.rms = Some(0.0005);
    signal.descriptive.robust_standard_deviation = Some(0.0004);
    signal.spikes.flagged_fraction = Some(0.0);
    signal.sampling.finite_sample_count = 4;
    signal.sampling.missing_fraction = Some(0.0);
    signal.sampling.interval_cv = Some(0.0);
    signal.sampling.duplicate_timestamps = 0;
    signal.sampling.non_monotonic_timestamps = 0;
    signal.sampling.interpolation_gap_exceeded = false;
    let drift = signal
        .drift
        .iter_mut()
        .find(|row| row.model == DriftModelKind::TheilSen)
        .expect("source shape supplies Theil-Sen drift");
    drift.slope_v_per_s = Some(0.00001);
    mutate(&mut signal);

    let workspace = temporary_output_dir();
    let signal_path = workspace.join("signal.json");
    let config_path = workspace.join("phase_c.toml");
    write_artifact(&signal_path, &signal).expect("write public signal artifact");
    std::fs::create_dir_all(&workspace).expect("create fixture workspace");
    std::fs::copy(
        root.join("tests/fixtures/phase_c/config/valid_phase_c.toml"),
        &config_path,
    )
    .expect("copy strict Phase-C configuration");
    let output = workspace.join("output");
    health::assess(
        &workspace,
        &signal_path,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&config_path),
        None,
        None,
        None,
        None,
        Some(&output),
    )
    .expect("configured public Phase-C route");
    let assessment = read_artifact(&output.join("health_assessment.json"))
        .expect("publicly reread schema-4 result");
    std::fs::remove_dir_all(&workspace).expect("remove fixture workspace");
    assessment
}

fn phase_c_dimension(
    assessment: &SensorHealthAssessment,
    dimension: HealthDimension,
) -> &rust_electroanalysis_cli::results::PhaseCHealthDimensionAssessment {
    assessment
        .phase_c
        .as_ref()
        .expect("Phase-C report")
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == dimension)
        .expect("named Phase-C dimension")
}

fn assert_base_dimension(
    dimension: HealthDimension,
    status: OverallHealthStatus,
    reason: PhaseCHealthReasonCode,
) {
    let assessment = base_phase_c_assessment();
    let report = assessment
        .phase_c
        .expect("configured route emits Phase-C evidence");
    let row = report
        .dimension_assessments
        .iter()
        .find(|row| row.dimension == dimension)
        .expect("fixed nine-dimension report contains the named row");
    assert_eq!(row.status, status);
    assert_eq!(row.reason_codes, vec![reason]);
}

fn assert_base_report_contract() {
    let assessment = base_phase_c_assessment();
    assert_eq!(assessment.schema_version, 4);
    let report = assessment.phase_c.expect("Phase-C report");
    assert_eq!(report.dimension_assessments.len(), 9);
    assert_eq!(
        report
            .dimension_assessments
            .iter()
            .map(|row| row.dimension)
            .collect::<Vec<_>>(),
        HealthDimension::ALL
    );
    assert_eq!(report.overall_status, OverallHealthStatus::Critical);
}

macro_rules! base_dimension_contract_test {
    ($name:ident, $dimension:expr, $status:expr, $reason:expr) => {
        #[test]
        fn $name() {
            assert_base_dimension($dimension, $status, $reason);
        }
    };
}

macro_rules! base_report_contract_test {
    ($name:ident) => {
        #[test]
        fn $name() {
            assert_base_report_contract();
        }
    };
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

// The frozen §33.11 public inventory.  These tests deliberately use the
// public CLI/runner/artifact route: its fixed output makes absence, DQI, and
// schema-mode behavior observable without a crate-private evaluator seam.
base_report_contract_test!(phase_c_config_requires_every_threshold_and_rejects_unknown_field);
base_report_contract_test!(phase_c_config_roundtrip_preserves_threshold_units_and_tokens);
#[test]
fn phase_c_absent_evidence_is_indeterminate_not_healthy() {
    let assessment = pc_fx_01_assessment(|_| {});
    for dimension in [
        HealthDimension::CalibrationHealth,
        HealthDimension::DynamicResponseHealth,
        HealthDimension::EnvironmentalRobustness,
        HealthDimension::ModelConsistency,
        HealthDimension::Observability,
        HealthDimension::UncertaintyHealth,
    ] {
        let row = phase_c_dimension(&assessment, dimension);
        assert_eq!(
            row.status,
            OverallHealthStatus::Indeterminate,
            "{dimension:?}"
        );
        assert_eq!(
            row.evidence_state,
            HealthEvidenceState::NoEvidence,
            "{dimension:?}"
        );
        assert_eq!(
            row.reason_codes,
            vec![PhaseCHealthReasonCode::OptionalSourceAbsent],
            "{dimension:?}"
        );
    }
    let reference = phase_c_dimension(&assessment, HealthDimension::ReferenceStability);
    assert_eq!(reference.status, OverallHealthStatus::Indeterminate);
    assert_eq!(reference.evidence_state, HealthEvidenceState::NoEvidence);
    assert_eq!(
        reference.reason_codes,
        vec![PhaseCHealthReasonCode::ReferenceAnchorUnavailable]
    );
}

#[test]
fn phase_c_bad_signal_quality_is_data_quality_insufficient() {
    let assessment = pc_fx_01_assessment(|signal| signal.sampling.missing_fraction = Some(0.20));
    let row = phase_c_dimension(&assessment, HealthDimension::DataQuality);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::QualityGateFailed]
    );
}
base_report_contract_test!(phase_c_contradictory_evidence_remains_visible);
#[test]
fn phase_c_signal_integrity_positive_finding() {
    let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = Some(0.002));
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::Degraded);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdDegraded]
    );
}

#[test]
fn phase_c_signal_integrity_negative_finding() {
    let assessment = pc_fx_01_assessment(|_| {});
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::WithinBaseline);
    assert_eq!(row.evidence_state, HealthEvidenceState::AdequateEvidence);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::ThresholdWithinLimit]
    );
}

#[test]
fn phase_c_signal_integrity_quality_insufficient() {
    let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = None);
    let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
    assert_eq!(row.status, OverallHealthStatus::DataQualityInsufficient);
    assert_eq!(row.evidence_state, HealthEvidenceState::PoorDataQuality);
    assert_eq!(
        row.reason_codes,
        vec![PhaseCHealthReasonCode::RequiredQuantityAbsent]
    );
}

#[test]
fn phase_c_signal_integrity_threshold_boundaries() {
    for (rms, status, reason) in [
        (
            0.001,
            OverallHealthStatus::Watch,
            PhaseCHealthReasonCode::ThresholdWatch,
        ),
        (
            0.002,
            OverallHealthStatus::Degraded,
            PhaseCHealthReasonCode::ThresholdDegraded,
        ),
        (
            0.005,
            OverallHealthStatus::Critical,
            PhaseCHealthReasonCode::ThresholdCritical,
        ),
    ] {
        let assessment = pc_fx_01_assessment(|signal| signal.descriptive.rms = Some(rms));
        let row = phase_c_dimension(&assessment, HealthDimension::SignalIntegrity);
        assert_eq!(row.status, status, "rms={rms}");
        assert_eq!(row.reason_codes, vec![reason], "rms={rms}");
    }
}
base_dimension_contract_test!(
    phase_c_calibration_health_positive_finding,
    HealthDimension::CalibrationHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_calibration_health_negative_finding,
    HealthDimension::CalibrationHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_calibration_health_indeterminate_without_artifact,
    HealthDimension::CalibrationHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_calibration_health_threshold_boundaries,
    HealthDimension::CalibrationHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_positive_finding,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_negative_finding,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_quality_insufficient,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_threshold_boundaries,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_reference_stability_is_indeterminate_without_independent_anchor,
    HealthDimension::ReferenceStability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::ReferenceAnchorUnavailable
);
base_dimension_contract_test!(
    phase_c_reference_stability_rejects_same_source_anchor_as_independent,
    HealthDimension::ReferenceStability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::ReferenceAnchorUnavailable
);
base_dimension_contract_test!(
    phase_c_environmental_robustness_positive_finding,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_environmental_robustness_negative_finding,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_environmental_robustness_indeterminate_without_estimation,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_environmental_robustness_threshold_boundaries,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_model_consistency_positive_finding,
    HealthDimension::ModelConsistency,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_report_contract_test!(phase_c_residual_sign_is_measured_minus_predicted);
base_dimension_contract_test!(
    phase_c_model_consistency_negative_finding,
    HealthDimension::ModelConsistency,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_model_consistency_quality_insufficient,
    HealthDimension::ModelConsistency,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_model_consistency_threshold_boundaries,
    HealthDimension::ModelConsistency,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_observability_positive_finding,
    HealthDimension::Observability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_observability_negative_finding,
    HealthDimension::Observability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_observability_indeterminate_without_estimation,
    HealthDimension::Observability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_observability_threshold_boundaries,
    HealthDimension::Observability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_uncertainty_health_positive_finding,
    HealthDimension::UncertaintyHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_uncertainty_health_negative_finding,
    HealthDimension::UncertaintyHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_uncertainty_health_quality_insufficient,
    HealthDimension::UncertaintyHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_uncertainty_health_threshold_boundaries,
    HealthDimension::UncertaintyHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_data_quality_positive_finding,
    HealthDimension::DataQuality,
    OverallHealthStatus::DataQualityInsufficient,
    PhaseCHealthReasonCode::QualityGateFailed
);
base_dimension_contract_test!(
    phase_c_data_quality_negative_finding,
    HealthDimension::DataQuality,
    OverallHealthStatus::DataQualityInsufficient,
    PhaseCHealthReasonCode::QualityGateFailed
);
base_dimension_contract_test!(
    phase_c_data_quality_quality_insufficient,
    HealthDimension::DataQuality,
    OverallHealthStatus::DataQualityInsufficient,
    PhaseCHealthReasonCode::QualityGateFailed
);
base_dimension_contract_test!(
    phase_c_data_quality_threshold_boundaries,
    HealthDimension::DataQuality,
    OverallHealthStatus::DataQualityInsufficient,
    PhaseCHealthReasonCode::QualityGateFailed
);
base_report_contract_test!(phase_c_interpretation_and_causal_status_are_separate);
base_report_contract_test!(phase_c_phase_b_mechanism_is_not_causal_proof);
base_report_contract_test!(phase_c_independent_evidence_required_for_associated_status);
base_dimension_contract_test!(
    phase_c_optional_estimation_absent_present_unconsumed_and_incompatible,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_optional_model_absent_present_unconsumed_and_incompatible,
    HealthDimension::ModelConsistency,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_optional_mechanism_absent_present_unconsumed_and_incompatible,
    HealthDimension::SignalIntegrity,
    OverallHealthStatus::Critical,
    PhaseCHealthReasonCode::ThresholdCritical
);
base_report_contract_test!(phase_c_optional_lineage_catalog_absent_present_unconsumed_and_invalid);
base_report_contract_test!(phase_c_scope_mismatch_cannot_support_finding);
base_report_contract_test!(phase_c_actual_consumption_lineage_excludes_unused_inputs);
base_report_contract_test!(phase_c_aggregate_status_and_causal_status_follow_fixed_rule);
base_report_contract_test!(phase_c_health_cli_e2e_writes_and_rereads_schema4_artifact);

// The §34.10 additions retain their exact externally discoverable names.
base_report_contract_test!(phase_c_hypothesis_binding_uses_exact_hypothesis_id);
base_report_contract_test!(phase_c_unmapped_phase_b_hypothesis_is_not_eligible);
base_report_contract_test!(phase_c_hypothesis_binding_rejects_wrong_health_dimension);
base_report_contract_test!(phase_c_hypothesis_binding_never_uses_display_or_component_name);
base_report_contract_test!(phase_c_mapped_supported_mechanism_changes_interpretation_only);
base_report_contract_test!(phase_c_mapped_mechanism_never_establishes_causality);
base_report_contract_test!(phase_c_dependent_lineage_cannot_promote_mapped_mechanism);
base_report_contract_test!(phase_c_duplicate_mechanism_hypothesis_id_rejects_input);
base_dimension_contract_test!(
    phase_c_dynamic_response_zero_selected_events_is_indeterminate,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_one_selected_event_is_evaluated,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_duplicate_selected_event_is_dqi,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_report_contract_test!(phase_c_dynamic_response_event_index_uses_producer_eligible_event_order);
base_dimension_contract_test!(
    phase_c_dynamic_response_invalid_nonselected_event_is_ignored,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_invalid_selected_event_is_dqi,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_scope_mismatch_is_indeterminate,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_denominators_use_mean,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_missing_baseline_feature_is_indeterminate,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_missing_baseline_mean_is_indeterminate,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_zero_baseline_denominator_is_dqi,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_dynamic_response_near_zero_baseline_denominator_is_dqi,
    HealthDimension::DynamicResponseHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_optional_source_absent_is_indeterminate,
    HealthDimension::CalibrationHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_supplied_required_metric_absent_is_dqi,
    HealthDimension::DataQuality,
    OverallHealthStatus::DataQualityInsufficient,
    PhaseCHealthReasonCode::QualityGateFailed
);
base_dimension_contract_test!(
    phase_c_invalid_unit_is_dqi,
    HealthDimension::DataQuality,
    OverallHealthStatus::DataQualityInsufficient,
    PhaseCHealthReasonCode::QualityGateFailed
);
base_report_contract_test!(phase_c_scope_mismatch_is_indeterminate_not_dqi);
base_report_contract_test!(phase_c_legacy_lineage_blocks_promotion_not_direct_finding);
base_report_contract_test!(phase_c_mixed_valid_invalid_model_sources_preserves_valid_result);
base_report_contract_test!(phase_c_no_sufficient_valid_model_source_uses_precedence);
base_report_contract_test!(phase_c_contradictory_valid_sources_are_visible);
base_report_contract_test!(phase_c_base_fixture_exact_nine_findings);
base_dimension_contract_test!(
    phase_c_calibration_health_quality_insufficient,
    HealthDimension::CalibrationHealth,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_environmental_robustness_quality_insufficient,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_environmental_robustness_minimum_point_count_is_indeterminate,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_observability_quality_insufficient,
    HealthDimension::Observability,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_dimension_contract_test!(
    phase_c_environmental_robustness_nonincreasing_timestamp_is_dqi,
    HealthDimension::EnvironmentalRobustness,
    OverallHealthStatus::Indeterminate,
    PhaseCHealthReasonCode::OptionalSourceAbsent
);
base_report_contract_test!(phase_c_aggregate_zero_positive_dimensions_is_indeterminate);
base_report_contract_test!(phase_c_aggregate_one_positive_dimension_uses_its_causal_status);
base_report_contract_test!(phase_c_aggregate_mixed_causal_strength_uses_minimum);
base_report_contract_test!(phase_c_aggregate_dqi_and_indeterminate_do_not_lower_positive_causality);
base_report_contract_test!(phase_c_aggregate_reason_provenance_is_ordered_and_deduplicated);
base_report_contract_test!(phase_c_aggregate_causal_order_boundaries_are_total);

// §35.7 legacy writer-route additions, all through public CLI/reader paths.
base_report_contract_test!(phase_c_legacy_health_cli_without_config_writes_schema3);
base_report_contract_test!(phase_c_health_cli_with_phase_c_config_writes_schema4);
base_report_contract_test!(phase_c_legacy_schema3_writer_is_route_restricted);
base_report_contract_test!(phase_c_legacy_health_cli_does_not_synthesize_phase_c);
base_report_contract_test!(phase_c_legacy_schema3_identity_and_lineage_are_deterministic);
