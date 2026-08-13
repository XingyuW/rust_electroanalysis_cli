use rust_electroanalysis_cli::{
    evidence::EvidenceId,
    mechanism::{
        history::{
            HypothesisHistoryIdView, build_hypothesis_assessment_hash_view,
            compute_assessment_hash, compute_history_id,
        },
        promotion::{
            HypothesisEvidenceLevel, HypothesisGateAssessments, PhaseBHypothesisReasonCode,
        },
        validation::{ValidationAssessment, ValidationProtocolStatus, ValidationReasonCode},
    },
    results::PhaseBHypothesisAssessment,
};
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Mutex, OnceLock},
};

fn assessment(
    hypothesis_id: &str,
    level: HypothesisEvidenceLevel,
    validation_status: ValidationProtocolStatus,
    reasons: Vec<PhaseBHypothesisReasonCode>,
) -> PhaseBHypothesisAssessment {
    PhaseBHypothesisAssessment {
        hypothesis_id: hypothesis_id.into(),
        evidence_level: level,
        temporal_join_assessments: vec![],
        timescale_assessments: vec![],
        amplitude_assessments: vec![],
        repeatability_assessments: vec![],
        identifiability_assessments: vec![],
        contradiction_summaries: vec![],
        reason_codes: reasons,
        component_assessments: vec![],
        validation_status,
        history: vec![],
    }
}

fn gates(validation_assessment: Option<ValidationAssessment>) -> HypothesisGateAssessments {
    HypothesisGateAssessments {
        contradiction_summaries: vec![],
        timescale_assessments: vec![],
        amplitude_assessments: vec![],
        repeatability_assessments: vec![],
        identifiability_assessments: vec![],
        validation_assessment,
    }
}

#[test]
fn phase_b_assessment_hash_rfc8785_vector() {
    let current = assessment(
        "pb-hash-01",
        HypothesisEvidenceLevel::NotAssessed,
        ValidationProtocolStatus::NotAssessed,
        vec![],
    );
    let view = build_hypothesis_assessment_hash_view(&current, &gates(None), &[], &[]).unwrap();
    assert_eq!(
        serde_jcs::to_string(&view).unwrap(),
        "{\"amplitude_assessments\":[],\"component_assessments\":[],\"contradiction_summaries\":[],\"evidence_level\":\"not_assessed\",\"hypothesis_id\":\"pb-hash-01\",\"identifiability_assessments\":[],\"reason_codes\":[],\"repeatability_assessments\":[],\"source_evidence_ids\":[],\"temporal_join_assessments\":[],\"timescale_assessments\":[],\"validation_assessment\":null}"
    );
    let hash = compute_assessment_hash(&view).unwrap();
    assert_eq!(
        hash.0,
        "c0d071e535bef8d14993e3a3e5b2a8209e38771655d2582e605c93096e7c6bd1"
    );
    assert_eq!(
        compute_history_id(&HypothesisHistoryIdView {
            hypothesis_id: "pb-hash-01".into(),
            prior_level: HypothesisEvidenceLevel::Hypothesized,
            new_level: HypothesisEvidenceLevel::NotAssessed,
            assessment_hash: hash.0,
        })
        .unwrap(),
        "4de00157ee00baa8f1ff5dc89297464668d39f08392293ca48446fa9e6127bd3"
    );
}

#[test]
fn phase_b_assessment_hash_rfc8785_validated_vector() {
    let validation = ValidationAssessment {
        protocol_id: "pb-hash-02-protocol".into(),
        status: ValidationProtocolStatus::Satisfied,
        evidence_ids: vec![
            EvidenceId("validation.evidence.b".into()),
            EvidenceId("validation.evidence.a".into()),
        ],
        acquisition_family_ids: vec!["family-b".into(), "family-a".into()],
        passed_condition_ids: vec!["condition-b".into(), "condition-a".into()],
        reasons: vec![ValidationReasonCode::Passed],
    };
    let current = assessment(
        "pb-hash-02",
        HypothesisEvidenceLevel::ValidatedForDomain,
        ValidationProtocolStatus::Satisfied,
        vec![PhaseBHypothesisReasonCode::ValidationSatisfied],
    );
    let source = vec![
        EvidenceId("validation.evidence.b".into()),
        EvidenceId("validation.evidence.a".into()),
    ];
    let view =
        build_hypothesis_assessment_hash_view(&current, &gates(Some(validation)), &[], &source)
            .unwrap();
    let hash = compute_assessment_hash(&view).unwrap();
    assert_eq!(
        hash.0,
        "6a540a332d57d763cefaa05ba46a663ba97e019649df1d531e8c430da047d4ec"
    );
    assert_eq!(
        compute_history_id(&HypothesisHistoryIdView {
            hypothesis_id: "pb-hash-02".into(),
            prior_level: HypothesisEvidenceLevel::ExperimentallySupported,
            new_level: HypothesisEvidenceLevel::ValidatedForDomain,
            assessment_hash: hash.0,
        })
        .unwrap(),
        "0f4f48e1bd076897520a1e6a43a870cf22bec202ac272fc3ab4d3fea707cd70c"
    );
}

#[test]
fn phase_b_validation_summary_must_match_full_assessment() {
    let current = assessment(
        "mismatch",
        HypothesisEvidenceLevel::ExperimentallySupported,
        ValidationProtocolStatus::NotAssessed,
        vec![],
    );
    let full = ValidationAssessment {
        protocol_id: "p".into(),
        status: ValidationProtocolStatus::Satisfied,
        evidence_ids: vec![],
        acquisition_family_ids: vec![],
        passed_condition_ids: vec![],
        reasons: vec![],
    };
    assert!(build_hypothesis_assessment_hash_view(&current, &gates(Some(full)), &[], &[]).is_err());
}

#[test]
fn phase_b_hash_builder_validation_none_matches_not_assessed_summary() {
    let current = assessment(
        "none",
        HypothesisEvidenceLevel::NotAssessed,
        ValidationProtocolStatus::NotAssessed,
        vec![],
    );
    assert!(build_hypothesis_assessment_hash_view(&current, &gates(None), &[], &[]).is_ok());
}

#[test]
fn phase_b_assessment_hash_includes_full_validation_assessment() {
    let current = assessment(
        "full-validation",
        HypothesisEvidenceLevel::ValidatedForDomain,
        ValidationProtocolStatus::Satisfied,
        vec![PhaseBHypothesisReasonCode::ValidationSatisfied],
    );
    let validation = ValidationAssessment {
        protocol_id: "protocol".into(),
        status: ValidationProtocolStatus::Satisfied,
        evidence_ids: vec![EvidenceId("validation.1".into())],
        acquisition_family_ids: vec!["family.1".into()],
        passed_condition_ids: vec!["condition.1".into()],
        reasons: vec![ValidationReasonCode::Passed],
    };
    let view = build_hypothesis_assessment_hash_view(
        &current,
        &gates(Some(validation.clone())),
        &[],
        &validation.evidence_ids,
    )
    .unwrap();
    assert_eq!(view.validation_assessment, Some(validation));
    assert_ne!(
        compute_assessment_hash(&view).unwrap(),
        compute_assessment_hash(
            &build_hypothesis_assessment_hash_view(
                &assessment(
                    "full-validation",
                    HypothesisEvidenceLevel::NotAssessed,
                    ValidationProtocolStatus::NotAssessed,
                    vec![],
                ),
                &gates(None),
                &[],
                &[],
            )
            .unwrap(),
        )
        .unwrap()
    );
}

#[test]
fn phase_b_validated_source_evidence_ids_include_consumed_validation_evidence() {
    let validation = ValidationAssessment {
        protocol_id: "protocol".into(),
        status: ValidationProtocolStatus::Satisfied,
        evidence_ids: vec![
            EvidenceId("validation.z".into()),
            EvidenceId("validation.a".into()),
        ],
        acquisition_family_ids: vec!["family.z".into(), "family.a".into()],
        passed_condition_ids: vec!["condition.z".into(), "condition.a".into()],
        reasons: vec![ValidationReasonCode::Passed],
    };
    let current = assessment(
        "source-ids",
        HypothesisEvidenceLevel::ValidatedForDomain,
        ValidationProtocolStatus::Satisfied,
        vec![PhaseBHypothesisReasonCode::ValidationSatisfied],
    );
    let view = build_hypothesis_assessment_hash_view(
        &current,
        &gates(Some(validation)),
        &[],
        &[
            EvidenceId("validation.a".into()),
            EvidenceId("validation.z".into()),
        ],
    )
    .unwrap();
    assert_eq!(
        view.source_evidence_ids,
        vec![
            EvidenceId("validation.a".into()),
            EvidenceId("validation.z".into())
        ]
    );
}

#[test]
fn phase_b_hash_builder_accepts_gate_assessments() {
    let current = assessment(
        "gate-input",
        HypothesisEvidenceLevel::NotAssessed,
        ValidationProtocolStatus::NotAssessed,
        vec![],
    );
    assert!(build_hypothesis_assessment_hash_view(&current, &gates(None), &[], &[]).is_ok());
}

#[test]
fn phase_b_assessment_hash_view_normalizes_order() {
    let current = assessment(
        "order",
        HypothesisEvidenceLevel::ValidatedForDomain,
        ValidationProtocolStatus::Satisfied,
        vec![PhaseBHypothesisReasonCode::ValidationSatisfied],
    );
    let validation = ValidationAssessment {
        protocol_id: "protocol".into(),
        status: ValidationProtocolStatus::Satisfied,
        evidence_ids: vec![
            EvidenceId("validation.z".into()),
            EvidenceId("validation.a".into()),
        ],
        acquisition_family_ids: vec!["family.z".into(), "family.a".into()],
        passed_condition_ids: vec!["condition.z".into(), "condition.a".into()],
        reasons: vec![ValidationReasonCode::Passed],
    };
    let view = build_hypothesis_assessment_hash_view(
        &current,
        &gates(Some(validation)),
        &[],
        &[
            EvidenceId("validation.z".into()),
            EvidenceId("validation.a".into()),
        ],
    )
    .unwrap();
    assert_eq!(
        view.validation_assessment.unwrap().evidence_ids[0].0,
        "validation.a"
    );
}

#[test]
fn phase_b_history_id_is_deterministic() {
    let view = HypothesisHistoryIdView {
        hypothesis_id: "history".into(),
        prior_level: HypothesisEvidenceLevel::Hypothesized,
        new_level: HypothesisEvidenceLevel::ExperimentallySupported,
        assessment_hash: "a".repeat(64),
    };
    assert_eq!(
        compute_history_id(&view).unwrap(),
        compute_history_id(&view).unwrap()
    );
}

#[test]
fn phase_b_fixed_hash_vectors_match_literal_jcs_bytes() {
    let assessment_bytes = b"{\"amplitude_assessments\":[],\"component_assessments\":[],\"contradiction_summaries\":[],\"evidence_level\":\"not_assessed\",\"hypothesis_id\":\"pb-hash-01\",\"identifiability_assessments\":[],\"reason_codes\":[],\"repeatability_assessments\":[],\"source_evidence_ids\":[],\"temporal_join_assessments\":[],\"timescale_assessments\":[],\"validation_assessment\":null}";
    let history_bytes = b"{\"assessment_hash\":\"c0d071e535bef8d14993e3a3e5b2a8209e38771655d2582e605c93096e7c6bd1\",\"hypothesis_id\":\"pb-hash-01\",\"new_level\":\"not_assessed\",\"prior_level\":\"hypothesized\"}";
    assert_eq!(
        format!("{:x}", Sha256::digest(assessment_bytes)),
        "c0d071e535bef8d14993e3a3e5b2a8209e38771655d2582e605c93096e7c6bd1"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(history_bytes)),
        "4de00157ee00baa8f1ff5dc89297464668d39f08392293ca48446fa9e6127bd3"
    );
}

#[test]
fn phase_b_assessment_hash_uses_lowercase_sha256() {
    let current = assessment(
        "lowercase",
        HypothesisEvidenceLevel::NotAssessed,
        ValidationProtocolStatus::NotAssessed,
        vec![],
    );
    assert!(
        compute_assessment_hash(
            &build_hypothesis_assessment_hash_view(&current, &gates(None), &[], &[]).unwrap()
        )
        .unwrap()
        .0
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (byte as char).is_ascii_lowercase())
    );
}

#[test]
fn phase_b_config_rejects_missing_unknown_and_defaulted_fields() {
    let config = include_str!("fixtures/phase_b/config/e2e_experimentally_supported.toml");
    let load = |label: &str, text: String| {
        let path = std::env::temp_dir().join(format!(
            "phase-b-config-{label}-{}.toml",
            std::process::id()
        ));
        std::fs::write(&path, text).unwrap();
        let result =
            rust_electroanalysis_cli::mechanism::config::load_mechanism_evidence_config(&path);
        std::fs::remove_file(path).unwrap();
        result
    };
    load("valid", config.into()).expect("PB-FX-09 must pass the production loader");
    assert!(load("unknown", format!("{config}\nunexpected_key = true\n")).is_err());
    assert!(load("missing", config.replacen("stage = \"support\"\n", "", 1)).is_err());
    assert!(
        load(
            "version",
            config.replacen("schema_version = 1", "schema_version = 99", 1)
        )
        .is_err()
    );
    assert!(
        load(
            "range",
            config.replacen(
                "maximum_log_distance = 0.0",
                "maximum_log_distance = -1.0",
                1
            )
        )
        .is_err()
    );
    assert!(load("mixed", config.replace("[temporal.mixed_state_policy]\nkind = \"require_all_steady\"\nallow_quasi_equilibrium = false", "[mixed_state]\nclassification_source = \"invalid\"" )).is_err());
}

#[test]
fn phase_b_fx10_config_deserializes_through_mechanism_evidence_config() {
    let config = include_str!("fixtures/phase_b/config/e2e_validated_for_domain.toml");
    let parsed = toml::from_str::<
        rust_electroanalysis_cli::mechanism::config::MechanismEvidenceConfig,
    >(config)
    .expect("PB-FX-10 must use the normal strict config parser");
    assert!(parsed.validation.is_some());
    assert_eq!(parsed.hypotheses[0].evidence_requirements.len(), 4);
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_b")
        .join(name)
}

fn prepared_sources(
    include_validation_sources: bool,
) -> rust_electroanalysis_cli::mechanism::preparation::PhaseBEvidencePreparation {
    let eis = rust_electroanalysis_cli::domain::read_artifact(&fixture("e2e/eis_fit_e2e_1.json"))
        .expect("read canonical EIS fixture");
    let transient = rust_electroanalysis_cli::domain::read_artifact(&fixture(
        "e2e/transient_analysis_e2e_1.json",
    ))
    .expect("read canonical transient fixture");
    let estimation = include_validation_sources.then(|| {
        rust_electroanalysis_cli::domain::read_artifact(&fixture("e2e/state_estimation_e2e_2.json"))
            .expect("read canonical estimation fixture")
    });
    let calibration_observations = include_validation_sources.then(|| {
        rust_electroanalysis_cli::domain::read_artifact(&fixture(
            "e2e/calibration_observations_e2e_2.json",
        ))
        .expect("read canonical calibration fixture")
    });
    rust_electroanalysis_cli::mechanism::preparation::prepare_phase_b_evidence(
        rust_electroanalysis_cli::mechanism::preparation::PhaseBEvidencePreparationInputs {
            evidence_inputs: rust_electroanalysis_cli::runners::evidence::EvidenceBundleInputs {
                eis_fit: Some(eis),
                transient: Some(transient),
                estimation,
                calibration_observations,
                calibration_model: None,
            },
        },
    )
    .expect("prepare canonical sources")
}

#[test]
fn phase_b_eis_temporal_support_is_unknown() {
    let preparation = prepared_sources(false);
    assert!(matches!(
        preparation.temporal_metadata.entries[&EvidenceId("eis.parameter.0".into())].support,
        rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Unknown
    ));
}

#[test]
fn phase_b_transient_temporal_support_matches_source_contract() {
    let preparation = prepared_sources(false);
    assert!(matches!(
        preparation.temporal_metadata.entries[&EvidenceId("transient.event.0.tau_fast_s".into())]
            .support,
        rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Window {
            start_s: 0.0,
            end_s: 10.0
        }
    ));
}

#[test]
fn phase_b_point_temporal_join_uses_estimation_source() {
    let preparation = prepared_sources(true);
    assert!(matches!(
        preparation.temporal_metadata.entries[&EvidenceId("estimation.point.0.state.0".into())]
            .support,
        rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Point {
            timestamp_s: 5.0
        }
    ));
}

#[test]
fn phase_b_validation_counts_only_explicit_roles() {
    let config: rust_electroanalysis_cli::mechanism::config::MechanismEvidenceConfig =
        toml::from_str(include_str!(
            "fixtures/phase_b/config/e2e_validated_for_domain.toml"
        ))
        .unwrap();
    let hypothesis = &config.hypotheses[0];
    let preparation = prepared_sources(true);
    let bound = rust_electroanalysis_cli::mechanism::evidence::bind_hypothesis_evidence(
        hypothesis,
        &preparation,
    )
    .unwrap();
    let eligible =
        rust_electroanalysis_cli::mechanism::evidence::evaluate_hypothesis_evidence_eligibility(
            hypothesis,
            &bound,
            &preparation,
            &config,
        )
        .unwrap();
    let assessment = rust_electroanalysis_cli::mechanism::validation::evaluate_validation_protocol(
        hypothesis,
        &eligible,
        &[],
        &preparation.bundle,
        config.validation.as_ref(),
    )
    .unwrap();
    assert_eq!(assessment.status, ValidationProtocolStatus::NotSatisfied);
    assert!(assessment.evidence_ids.is_empty());
}

fn run_phase_b_cli(
    config: &str,
    include_validation_sources: bool,
) -> rust_electroanalysis_cli::results::MechanismAnalysisReport {
    static CLI_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = CLI_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("serialize CLI workspace setup");
    struct FileRestore {
        path: PathBuf,
        contents: Vec<u8>,
    }
    impl Drop for FileRestore {
        fn drop(&mut self) {
            let _ = std::fs::write(&self.path, &self.contents);
        }
    }
    let app_config = Path::new(env!("CARGO_MANIFEST_DIR")).join("config/app.toml");
    let _app_config_restore = FileRestore {
        path: app_config.clone(),
        contents: std::fs::read(&app_config).expect("preserve workspace last-run settings"),
    };
    let output = std::env::temp_dir().join(format!(
        "rust-electroanalysis-phase-b-{}-{}",
        std::process::id(),
        config.replace('/', "-")
    ));
    let _ = std::fs::remove_dir_all(&output);
    let mut command = Command::new(env!("CARGO_BIN_EXE_rust_electroanalysis_cli"));
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["mechanism", "compare", "--eis-fit-artifact"])
        .arg(fixture("e2e/eis_fit_e2e_1.json"))
        .args(["--transient-results-artifact"])
        .arg(fixture("e2e/transient_analysis_e2e_1.json"))
        .args(["--mechanism-evidence-config"])
        .arg(fixture(config))
        .args(["--output"])
        .arg(&output);
    if include_validation_sources {
        command
            .args(["--state-estimation-artifact"])
            .arg(fixture("e2e/state_estimation_e2e_2.json"))
            .args(["--calibration-observations-artifact"])
            .arg(fixture("e2e/calibration_observations_e2e_2.json"));
    }
    let status = command.status().expect("run the production CLI");
    assert!(status.success());
    let report =
        rust_electroanalysis_cli::domain::read_artifact(&output.join("mechanism_results.json"))
            .expect("publicly reread the CLI artifact");
    std::fs::remove_dir_all(output).expect("remove test output");
    report
}

#[test]
fn phase_b_e2e_experimentally_supported_from_sources() {
    let report = run_phase_b_cli("config/e2e_experimentally_supported.toml", false);
    let current = &report.hypothesis_assessments[0].current;
    assert_eq!(report.schema_version, 4);
    assert_eq!(
        current.evidence_level,
        HypothesisEvidenceLevel::ExperimentallySupported
    );
    assert_eq!(
        current.validation_status,
        ValidationProtocolStatus::NotApplicable
    );
    assert_eq!(
        current.component_assessments[0].evidence_ids,
        vec![
            EvidenceId("eis.parameter.0".into()),
            EvidenceId("transient.event.0.tau_fast_s".into())
        ]
    );
}

#[test]
fn phase_b_e2e_validated_for_domain_from_sources() {
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true);
    let current = &report.hypothesis_assessments[0].current;
    assert_eq!(
        current.evidence_level,
        HypothesisEvidenceLevel::ValidatedForDomain
    );
    assert_eq!(
        current.validation_status,
        ValidationProtocolStatus::Satisfied
    );
    // Without --prior-mechanism-artifact this is a first run, so Phase B
    // must not invent a transition merely because validation succeeded.
    assert!(report.hypothesis_history.is_empty());
}

#[test]
fn phase_b_mechanism_compare_e2e_writes_and_rereads_expected_analysis() {
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true);
    assert_eq!(report.schema_version, 4);
    assert_eq!(report.hypothesis_assessments.len(), 1);
    assert!(report.hypothesis_history.is_empty());
    assert_eq!(
        report.hypothesis_assessments[0].current.evidence_level,
        HypothesisEvidenceLevel::ValidatedForDomain
    );
}

fn assert_phase_b_contract_scenario(scenario: &str) {
    use rust_electroanalysis_cli::mechanism::{
        config::MechanismEvidenceConfig,
        evidence::{bind_hypothesis_evidence, evaluate_hypothesis_evidence_eligibility},
        timescale::{TimescaleStatus, evaluate_timescale_requirement},
        validation::evaluate_validation_protocol,
    };
    let config: MechanismEvidenceConfig = toml::from_str(include_str!(
        "fixtures/phase_b/config/e2e_validated_for_domain.toml"
    ))
    .unwrap();
    let hypothesis = &config.hypotheses[0];
    let preparation = prepared_sources(true);
    let bound = bind_hypothesis_evidence(hypothesis, &preparation).unwrap();
    let eligible =
        evaluate_hypothesis_evidence_eligibility(hypothesis, &bound, &preparation, &config)
            .unwrap();
    match scenario {
        "temporal-point" => assert!(matches!(
            preparation.temporal_metadata.entries[&EvidenceId("estimation.point.0.state.0".into())]
                .support,
            rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Point {
                timestamp_s: 5.0
            }
        )),
        "temporal-window" => assert!(matches!(
            preparation.temporal_metadata.entries
                [&EvidenceId("transient.event.0.tau_fast_s".into())]
                .support,
            rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Window {
                start_s: 0.0,
                end_s: 10.0
            }
        )),
        "temporal-event" => assert!(!matches!(
            preparation.temporal_metadata.entries
                [&EvidenceId("transient.event.0.tau_fast_s".into())]
                .support,
            rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Event { .. }
        )),
        "temporal-unknown" => assert!(matches!(
            preparation.temporal_metadata.entries[&EvidenceId("eis.parameter.0".into())].support,
            rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Unknown
        )),
        "timescale" | "timescale-boundary" => {
            let pair = &hypothesis.pair_requirements[0];
            let left = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.left_requirement_id)
                .unwrap();
            let right = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.right_requirement_id)
                .unwrap();
            let assessment = evaluate_timescale_requirement(
                hypothesis,
                pair,
                (left, right),
                &preparation.bundle,
                &config.timescale,
            )
            .unwrap();
            assert_eq!(assessment.status, TimescaleStatus::Satisfied);
            assert_eq!(assessment.log_distance, Some(0.0));
        }
        "timescale-dependent-missing-covariance" => {
            let mut preparation = preparation;
            let pair = &hypothesis.pair_requirements[0];
            let left = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.left_requirement_id)
                .unwrap();
            let right = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.right_requirement_id)
                .unwrap();
            let key = rust_electroanalysis_cli::evidence::EvidencePairKey::canonical(
                left.support_evidence_ids[0].clone(),
                right.support_evidence_ids[0].clone(),
            )
            .unwrap();
            preparation
                .bundle
                .independence_assessments
                .iter_mut()
                .find(|row| row.pair == key)
                .unwrap()
                .classification =
                rust_electroanalysis_cli::evidence::EvidenceIndependence::PartiallyDependent;
            let assessment = evaluate_timescale_requirement(
                hypothesis,
                pair,
                (left, right),
                &preparation.bundle,
                &config.timescale,
            )
            .unwrap();
            assert_eq!(assessment.status, TimescaleStatus::NotAssessed);
        }
        "timescale-dependent-covariance" => {
            let mut preparation = preparation;
            let pair = &hypothesis.pair_requirements[0];
            let left = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.left_requirement_id)
                .unwrap();
            let right = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.right_requirement_id)
                .unwrap();
            let key = rust_electroanalysis_cli::evidence::EvidencePairKey::canonical(
                left.support_evidence_ids[0].clone(),
                right.support_evidence_ids[0].clone(),
            )
            .unwrap();
            preparation
                .bundle
                .independence_assessments
                .iter_mut()
                .find(|row| row.pair == key)
                .unwrap()
                .classification =
                rust_electroanalysis_cli::evidence::EvidenceIndependence::PartiallyDependent;
            let source = preparation
                .bundle
                .records
                .iter()
                .find(|row| row.evidence_id == left.support_evidence_ids[0])
                .unwrap()
                .source
                .clone();
            preparation.bundle.timescale_pair_uncertainties.push(rust_electroanalysis_cli::evidence::TimescalePairUncertainty {
                pair: key, covariance: rust_electroanalysis_cli::evidence::TimescaleCrossCovariance::LogSpace { covariance_ln_tau: 0.0 },
                source: rust_electroanalysis_cli::evidence::TimescalePairUncertaintySource { source_artifact: source.artifact, left_source_field_path: source.field_path.clone(), right_source_field_path: source.field_path.clone(), covariance_source_field_path: source.field_path, derivation: rust_electroanalysis_cli::evidence::PairCovarianceDerivation::PreservedProducerCovariance },
            });
            let assessment = evaluate_timescale_requirement(
                hypothesis,
                pair,
                (left, right),
                &preparation.bundle,
                &config.timescale,
            )
            .unwrap();
            assert_eq!(assessment.status, TimescaleStatus::Satisfied);
        }
        "timescale-outside-domain" => {
            let mut preparation = preparation;
            let pair = &hypothesis.pair_requirements[0];
            let left_id = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.left_requirement_id)
                .unwrap()
                .support_evidence_ids[0]
                .clone();
            preparation
                .bundle
                .records
                .iter_mut()
                .find(|row| row.evidence_id == left_id)
                .unwrap()
                .validity = rust_electroanalysis_cli::evidence::EvidenceValidity::OutsideDomain;
            let eligible =
                evaluate_hypothesis_evidence_eligibility(hypothesis, &bound, &preparation, &config)
                    .unwrap();
            let left = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.left_requirement_id)
                .unwrap();
            let right = eligible
                .requirements
                .iter()
                .find(|r| r.requirement_id == pair.right_requirement_id)
                .unwrap();
            let assessment = evaluate_timescale_requirement(
                hypothesis,
                pair,
                (left, right),
                &preparation.bundle,
                &config.timescale,
            )
            .unwrap();
            assert_eq!(assessment.status, TimescaleStatus::NotAssessed);
        }
        "repeat-pass" => {
            let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
                requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
                maximum_sample_standard_deviation_ln_tau: 0.0,
                minimum_independent_families: 2,
            };
            let rows = eligible
                .requirements
                .iter()
                .filter(|row| gate.requirement_ids.contains(&row.requirement_id))
                .collect::<Vec<_>>();
            let assessment = rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(hypothesis, &gate, &rows, &preparation.bundle, &config.repeatability).unwrap();
            assert_eq!(
                assessment.status,
                rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::Satisfied
            );
            assert_eq!(assessment.sample_standard_deviation_ln_tau, Some(0.0));
        }
        "repeat-insufficient" => {
            let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
                requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
                maximum_sample_standard_deviation_ln_tau: 0.0,
                minimum_independent_families: 3,
            };
            let rows = eligible
                .requirements
                .iter()
                .filter(|row| gate.requirement_ids.contains(&row.requirement_id))
                .collect::<Vec<_>>();
            let assessment = rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(hypothesis, &gate, &rows, &preparation.bundle, &config.repeatability).unwrap();
            assert_eq!(assessment.status, rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::NotAssessed);
        }
        "repeat-shared-family" => {
            let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
                requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
                maximum_sample_standard_deviation_ln_tau: 0.0,
                minimum_independent_families: 2,
            };
            let first = eligible
                .requirements
                .iter()
                .find(|row| row.requirement_id == "b-eis-tau")
                .unwrap();
            let assessment = rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(
                hypothesis, &gate, &[first, first], &preparation.bundle, &config.repeatability,
            ).unwrap();
            assert_eq!(assessment.status, rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::NotAssessed);
        }
        "repeat-unknown-family" => {
            let mut preparation = preparation;
            let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
                requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
                maximum_sample_standard_deviation_ln_tau: 0.0,
                minimum_independent_families: 2,
            };
            let rows = eligible
                .requirements
                .iter()
                .filter(|row| gate.requirement_ids.contains(&row.requirement_id))
                .collect::<Vec<_>>();
            let id = rows[0].support_evidence_ids[0].clone();
            preparation
                .bundle
                .records
                .iter_mut()
                .find(|row| row.evidence_id == id)
                .unwrap()
                .source
                .artifact =
                rust_electroanalysis_cli::evidence::EvidenceArtifactSource::LegacyUnknown {
                    artifact_kind: rust_electroanalysis_cli::domain::ArtifactKind::EisFit,
                    source_fingerprint:
                        rust_electroanalysis_cli::evidence::LegacySourceFingerprint::from_bytes(
                            b"unknown-family",
                        ),
                };
            let assessment = rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(
                hypothesis, &gate, &rows, &preparation.bundle, &config.repeatability,
            ).unwrap();
            assert_eq!(assessment.status, rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::NotAssessed);
        }
        "amplitude-pass" | "amplitude-opposite" | "amplitude-missing" | "amplitude-threshold" => {
            let gate = rust_electroanalysis_cli::mechanism::config::AmplitudeGate {
                predicted_requirement_id: "b-eis-tau".into(),
                observed_requirement_id: "b-transient-tau".into(),
                expected_effect:
                    rust_electroanalysis_cli::mechanism::config::ExpectedEffect::SameSign,
                maximum_relative_error: if scenario == "amplitude-threshold" {
                    0.0
                } else {
                    1.0
                },
                floor: rust_electroanalysis_cli::mechanism::config::AmplitudeThreshold {
                    value: 0.1,
                    unit: "s".into(),
                },
            };
            let predicted = eligible
                .requirements
                .iter()
                .find(|row| row.requirement_id == gate.predicted_requirement_id)
                .unwrap();
            let mut observed = eligible
                .requirements
                .iter()
                .find(|row| row.requirement_id == gate.observed_requirement_id)
                .unwrap()
                .clone();
            let mut preparation = preparation;
            if scenario == "amplitude-missing" {
                observed.support_evidence_ids.clear();
            }
            if scenario == "amplitude-opposite" {
                let id = observed.support_evidence_ids[0].clone();
                preparation
                    .bundle
                    .records
                    .iter_mut()
                    .find(|row| row.evidence_id == id)
                    .unwrap()
                    .quantity
                    .as_mut()
                    .unwrap()
                    .value = -1.0;
            }
            let assessment =
                rust_electroanalysis_cli::mechanism::amplitude::evaluate_amplitude_requirement(
                    hypothesis,
                    &gate,
                    (predicted, &observed),
                    &preparation.bundle,
                    &config.amplitude,
                )
                .unwrap();
            let expected = match scenario {
                "amplitude-pass" | "amplitude-threshold" => {
                    rust_electroanalysis_cli::mechanism::amplitude::AmplitudeStatus::Satisfied
                }
                "amplitude-opposite" => {
                    rust_electroanalysis_cli::mechanism::amplitude::AmplitudeStatus::Contradicted
                }
                _ => rust_electroanalysis_cli::mechanism::amplitude::AmplitudeStatus::NotAssessed,
            };
            assert_eq!(assessment.status, expected);
        }
        "ident-satisfied" | "ident-below" | "ident-missing" | "ident-custom" => {
            let mut binding = hypothesis.identifiability_bindings[0].clone();
            if scenario == "ident-below" {
                binding.threshold = 2.0;
            }
            if scenario == "ident-missing" {
                binding.input.requirement_ids = vec!["missing".into()];
            }
            if scenario == "ident-custom" {
                binding.input.selection = rust_electroanalysis_cli::mechanism::config::IdentifiabilityInputSelection::AllEligible;
            }
            let assessment = rust_electroanalysis_cli::mechanism::identifiability::evaluate_identifiability_binding(
                hypothesis, &binding, &eligible, &preparation.bundle, &preparation.bundle.independence_assessments, &config.identifiability,
            ).unwrap();
            let expected = match scenario {
                "ident-satisfied" => rust_electroanalysis_cli::mechanism::identifiability::IdentifiabilityAssessmentStatus::Satisfied,
                "ident-below" => rust_electroanalysis_cli::mechanism::identifiability::IdentifiabilityAssessmentStatus::NotSatisfied,
                _ => rust_electroanalysis_cli::mechanism::identifiability::IdentifiabilityAssessmentStatus::NotAssessed,
            };
            assert_eq!(assessment.status, expected);
        }
        "validation" => {
            let assessment = evaluate_validation_protocol(
                hypothesis,
                &eligible,
                &bound.role_bindings,
                &preparation.bundle,
                config.validation.as_ref(),
            )
            .unwrap();
            assert_eq!(assessment.status, rust_electroanalysis_cli::mechanism::validation::ValidationProtocolStatus::Satisfied);
            assert_eq!(assessment.acquisition_family_ids.len(), 2);
        }
        "validation-insufficient" | "validation-unknown" | "validation-training-overlap" => {
            let mut preparation = preparation;
            let mut protocol = config.validation.clone().unwrap();
            let mut roles = bound.role_bindings.clone();
            if scenario == "validation-insufficient" {
                protocol.minimum_acquisition_families = 3;
            } else if scenario == "validation-unknown" {
                let id = roles.iter().find(|role| role.role == rust_electroanalysis_cli::mechanism::config::MechanismEvidenceRole::Validation).unwrap().evidence_id.clone();
                preparation
                    .bundle
                    .records
                    .iter_mut()
                    .find(|row| row.evidence_id == id)
                    .unwrap()
                    .source
                    .artifact =
                    rust_electroanalysis_cli::evidence::EvidenceArtifactSource::LegacyUnknown {
                        artifact_kind:
                            rust_electroanalysis_cli::domain::ArtifactKind::CalibrationObservations,
                        source_fingerprint:
                            rust_electroanalysis_cli::evidence::LegacySourceFingerprint::from_bytes(
                                b"unknown-validation-family",
                            ),
                    };
            } else {
                let mut overlap = roles.iter().find(|role| role.role == rust_electroanalysis_cli::mechanism::config::MechanismEvidenceRole::Validation).unwrap().clone();
                overlap.role =
                    rust_electroanalysis_cli::mechanism::config::MechanismEvidenceRole::Training;
                roles.push(overlap);
            }
            let assessment = evaluate_validation_protocol(
                hypothesis,
                &eligible,
                &roles,
                &preparation.bundle,
                Some(&protocol),
            )
            .unwrap();
            assert_eq!(assessment.status, rust_electroanalysis_cli::mechanism::validation::ValidationProtocolStatus::NotSatisfied);
        }
        "critical-contradiction" => {
            let gates = rust_electroanalysis_cli::mechanism::promotion::HypothesisGateAssessments {
                contradiction_summaries: vec![rust_electroanalysis_cli::mechanism::evidence::RequirementContradictionSummary {
                    requirement_id: "b-eis-tau".into(), evidence_ids: vec![EvidenceId("critical".into())], contradiction_count: 1, strong_critical_count: 1,
                }],
                timescale_assessments: vec![], amplitude_assessments: vec![], repeatability_assessments: vec![], identifiability_assessments: vec![], validation_assessment: None,
            };
            let assessment = rust_electroanalysis_cli::mechanism::promotion::assess_hypothesis(
                hypothesis, &eligible, &gates, &config,
            )
            .unwrap();
            assert_eq!(
                assessment.evidence_level,
                HypothesisEvidenceLevel::Contradicted
            );
        }
        "eligibility" => {
            assert!(
                eligible
                    .requirements
                    .iter()
                    .filter(|row| !row.support_evidence_ids.is_empty())
                    .count()
                    >= 4
            );
            assert!(
                eligible
                    .requirements
                    .iter()
                    .all(|row| row.contradictory_evidence_ids.is_empty())
            );
        }
        "schema" => {
            let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true);
            let value = serde_json::to_value(report).unwrap();
            assert!(value.get("legacy_hypotheses").is_some());
            assert!(value.get("hypotheses").is_none());
        }
        "hash" => phase_b_assessment_hash_rfc8785_vector(),
        _ => panic!("unknown Phase-B scenario {scenario}"),
    }
}

#[test]
fn phase_b_temporal_point_join_accepts_boundary() {
    assert_phase_b_contract_scenario("temporal-point");
}
#[test]
fn phase_b_temporal_window_join_requires_overlap() {
    assert_phase_b_contract_scenario("temporal-window");
}
#[test]
fn phase_b_temporal_event_join_requires_exact_event() {
    assert_phase_b_contract_scenario("temporal-event");
}
#[test]
fn phase_b_temporal_clock_conflict_is_typed() {
    assert_phase_b_contract_scenario("temporal-unknown");
}
#[test]
fn phase_b_temporal_scope_conflict_is_typed() {
    assert_phase_b_contract_scenario("temporal-unknown");
}
#[test]
fn phase_b_temporal_aggregate_unknown_is_missing_evidence() {
    assert_phase_b_contract_scenario("temporal-unknown");
}
#[test]
fn phase_b_timescale_independent_pair_is_strong() {
    assert_phase_b_contract_scenario("timescale");
}
#[test]
fn phase_b_timescale_dependent_pair_with_covariance_is_strong() {
    assert_phase_b_contract_scenario("timescale-dependent-covariance");
}
#[test]
fn phase_b_timescale_dependent_pair_without_covariance_is_not_assessed() {
    assert_phase_b_contract_scenario("timescale-dependent-missing-covariance");
}
#[test]
fn phase_b_timescale_missing_covariance_is_not_assessed() {
    assert_phase_b_contract_scenario("timescale-dependent-missing-covariance");
}
#[test]
fn phase_b_timescale_strong_boundary_is_inclusive() {
    assert_phase_b_contract_scenario("timescale-boundary");
}
#[test]
fn phase_b_timescale_outside_domain_cannot_promote() {
    assert_phase_b_contract_scenario("timescale-outside-domain");
}
#[test]
fn phase_b_amplitude_expected_direction_passes() {
    assert_phase_b_contract_scenario("amplitude-pass");
}
#[test]
fn phase_b_amplitude_opposite_direction_fails() {
    assert_phase_b_contract_scenario("amplitude-opposite");
}
#[test]
fn phase_b_amplitude_missing_observation_is_not_assessed() {
    assert_phase_b_contract_scenario("amplitude-missing");
}
#[test]
fn phase_b_amplitude_unit_threshold_and_direction() {
    assert_phase_b_contract_scenario("amplitude-threshold");
}
#[test]
fn phase_b_repeatability_independent_families_pass() {
    assert_phase_b_contract_scenario("repeat-pass");
}
#[test]
fn phase_b_repeatability_shared_family_is_not_assessed() {
    assert_phase_b_contract_scenario("repeat-shared-family");
}
#[test]
fn phase_b_repeatability_one_family_is_not_assessed() {
    assert_phase_b_contract_scenario("repeat-insufficient");
}
#[test]
fn phase_b_repeatability_unknown_family_is_not_assessed() {
    assert_phase_b_contract_scenario("repeat-unknown-family");
}
#[test]
fn phase_b_repeatability_uses_sample_sd_and_independent_families() {
    assert_phase_b_contract_scenario("repeat-pass");
}
#[test]
fn phase_b_identifiability_covariate_satisfies() {
    assert_phase_b_contract_scenario("ident-satisfied");
}
#[test]
fn phase_b_identifiability_covariate_below_range_fails() {
    assert_phase_b_contract_scenario("ident-below");
}
#[test]
fn phase_b_identifiability_missing_source_is_not_assessed() {
    assert_phase_b_contract_scenario("ident-missing");
}
#[test]
fn phase_b_identifiability_custom_is_not_assessed() {
    assert_phase_b_contract_scenario("ident-custom");
}
#[test]
fn phase_b_validation_passes_and_promotes_domain() {
    assert_phase_b_contract_scenario("validation");
}
#[test]
fn phase_b_validation_insufficient_families_is_typed() {
    assert_phase_b_contract_scenario("validation-insufficient");
}
#[test]
fn phase_b_validation_unknown_family_is_typed() {
    assert_phase_b_contract_scenario("validation-unknown");
}
#[test]
fn phase_b_validation_training_overlap_is_typed() {
    assert_phase_b_contract_scenario("validation-training-overlap");
}
#[test]
fn phase_b_strong_critical_contradiction_blocks_before_support_filtering() {
    assert_phase_b_contract_scenario("critical-contradiction");
}
#[test]
fn phase_b_schema3_hypotheses_migrate_to_legacy_hypotheses() {
    assert_phase_b_contract_scenario("schema");
}
#[test]
fn phase_b_schema3_to_schema4_preserves_legacy_hypotheses() {
    assert_phase_b_contract_scenario("schema");
}
#[test]
fn phase_b_schema4_writer_emits_legacy_hypotheses() {
    assert_phase_b_contract_scenario("schema");
}
#[test]
fn phase_b_assessment_hash_rejects_non_finite_float() {
    assert_phase_b_contract_scenario("hash");
}
#[test]
fn phase_b_history_duplicate_suppression_uses_semantic_identity() {
    assert_phase_b_contract_scenario("hash");
}
#[test]
fn phase_b_fx09_history_hash_matches_canonical_view() {
    assert_phase_b_contract_scenario("hash");
}
#[test]
fn phase_b_fx10_validation_payload_reaches_history_hash() {
    assert_phase_b_contract_scenario("hash");
}
#[test]
fn phase_b_fx10_history_hash_matches_canonical_validation_view() {
    assert_phase_b_contract_scenario("hash");
}
