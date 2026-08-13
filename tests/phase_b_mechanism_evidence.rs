use clap::Parser;
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
    prior_artifact: Option<&Path>,
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
        .args(["mechanism", "compare", "--eis-artifact"])
        .arg(fixture("e2e/eis_fit_e2e_1.json"))
        .args(["--transient-artifact"])
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
    if let Some(prior_artifact) = prior_artifact {
        command
            .args(["--prior-mechanism-artifact"])
            .arg(prior_artifact);
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
    let report = run_phase_b_cli("config/e2e_experimentally_supported.toml", false, None);
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
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true, None);
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
    assert!(
        rust_electroanalysis_cli::cli::Cli::try_parse_from([
            "electroanalysis",
            "mechanism",
            "compare",
            "--eis-artifact",
            "eis.json",
            "--transient-artifact",
            "transient.json",
        ])
        .is_ok()
    );
    for retired in ["--eis-fit-artifact", "--transient-results-artifact"] {
        assert!(
            rust_electroanalysis_cli::cli::Cli::try_parse_from([
                "electroanalysis",
                "mechanism",
                "compare",
                retired,
                "artifact.json",
                "--transient-artifact",
                "transient.json",
            ])
            .is_err()
        );
    }
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true, None);
    assert_eq!(report.schema_version, 4);
    assert_eq!(report.hypothesis_assessments.len(), 1);
    assert!(report.hypothesis_history.is_empty());
    assert_eq!(
        report.hypothesis_assessments[0].current.evidence_level,
        HypothesisEvidenceLevel::ValidatedForDomain
    );
}

struct PhaseBContext {
    config: rust_electroanalysis_cli::mechanism::config::MechanismEvidenceConfig,
    preparation: rust_electroanalysis_cli::mechanism::preparation::PhaseBEvidencePreparation,
    bound: rust_electroanalysis_cli::mechanism::evidence::BoundHypothesisEvidence,
    eligible: rust_electroanalysis_cli::mechanism::evidence::EligibleHypothesisEvidence,
}

fn phase_b_context() -> PhaseBContext {
    use rust_electroanalysis_cli::mechanism::{
        config::MechanismEvidenceConfig,
        evidence::{bind_hypothesis_evidence, evaluate_hypothesis_evidence_eligibility},
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
    PhaseBContext {
        config,
        preparation,
        bound,
        eligible,
    }
}

#[test]
fn phase_b_temporal_point_join_accepts_boundary() {
    let mut preparation = prepared_sources(true);
    let left = EvidenceId("estimation.point.0.state.0".into());
    let right = EvidenceId("transient.event.0.tau_fast_s".into());
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .support = rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Point {
        timestamp_s: 5.1,
    };
    preparation
        .temporal_metadata
        .entries
        .get_mut(&left)
        .unwrap()
        .clock_id = Some(rust_electroanalysis_cli::mechanism::temporal::ClockId(
        "clock".into(),
    ));
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .clock_id = Some(rust_electroanalysis_cli::mechanism::temporal::ClockId(
        "clock".into(),
    ));
    for id in [&left, &right] {
        let classification = &mut preparation
            .temporal_metadata
            .entries
            .get_mut(id)
            .unwrap()
            .classification;
        classification.classified_fraction = Some(1.0);
        classification.equilibrium_fraction = Some(1.0);
        classification.steady_state_fraction = Some(1.0);
    }
    let mut config = phase_b_context().config;
    config.temporal.point_tolerance_s = 0.1;
    let assessment = rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
        &rust_electroanalysis_cli::mechanism::temporal::TemporalJoinRequest {
            requirement_id: "boundary".into(),
            left_evidence_id: left,
            right_evidence_id: right,
            mode: rust_electroanalysis_cli::mechanism::config::TemporalJoinMode::PointPoint,
        },
        &preparation.bundle,
        &preparation.temporal_metadata,
        &config.temporal,
    )
    .unwrap();
    assert_eq!(
        assessment.outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Eligible
    );
}
#[test]
fn phase_b_temporal_window_join_requires_overlap() {
    let mut preparation = prepared_sources(true);
    let left = EvidenceId("transient.event.0.tau_fast_s".into());
    let right = EvidenceId("estimation.point.0.state.0".into());
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .support = rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Window {
        start_s: 0.0,
        end_s: 10.0,
    };
    for id in [&left, &right] {
        let metadata = preparation.temporal_metadata.entries.get_mut(id).unwrap();
        metadata.clock_id = Some(rust_electroanalysis_cli::mechanism::temporal::ClockId(
            "clock".into(),
        ));
        metadata.classification.classified_fraction = Some(1.0);
        metadata.classification.equilibrium_fraction = Some(1.0);
        metadata.classification.steady_state_fraction = Some(1.0);
    }
    let config = phase_b_context().config;
    let request =
        |right_evidence_id| rust_electroanalysis_cli::mechanism::temporal::TemporalJoinRequest {
            requirement_id: "window".into(),
            left_evidence_id: left.clone(),
            right_evidence_id,
            mode: rust_electroanalysis_cli::mechanism::config::TemporalJoinMode::WindowWindow,
        };
    assert_eq!(
        rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
            &request(right.clone()),
            &preparation.bundle,
            &preparation.temporal_metadata,
            &config.temporal
        )
        .unwrap()
        .outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Eligible
    );
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .support = rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Window {
        start_s: 20.0,
        end_s: 30.0,
    };
    assert_eq!(
        rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
            &request(right),
            &preparation.bundle,
            &preparation.temporal_metadata,
            &config.temporal
        )
        .unwrap()
        .outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Ineligible
    );
}
#[test]
fn phase_b_temporal_event_join_requires_exact_event() {
    let mut preparation = prepared_sources(true);
    let left = EvidenceId("transient.event.0.tau_fast_s".into());
    let right = EvidenceId("estimation.point.0.state.0".into());
    for (id, event_id) in [(&left, "event-a"), (&right, "event-a")] {
        preparation
            .temporal_metadata
            .entries
            .get_mut(id)
            .unwrap()
            .support =
            rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Event {
                event_id: event_id.into(),
                start_s: 0.0,
                end_s: 1.0,
            };
        let metadata = preparation.temporal_metadata.entries.get_mut(id).unwrap();
        metadata.clock_id = Some(rust_electroanalysis_cli::mechanism::temporal::ClockId(
            "clock".into(),
        ));
        metadata.classification.classified_fraction = Some(1.0);
        metadata.classification.equilibrium_fraction = Some(1.0);
        metadata.classification.steady_state_fraction = Some(1.0);
    }
    let config = phase_b_context().config;
    let request = rust_electroanalysis_cli::mechanism::temporal::TemporalJoinRequest {
        requirement_id: "event".into(),
        left_evidence_id: left.clone(),
        right_evidence_id: right.clone(),
        mode: rust_electroanalysis_cli::mechanism::config::TemporalJoinMode::EventEvent,
    };
    assert_eq!(
        rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
            &request,
            &preparation.bundle,
            &preparation.temporal_metadata,
            &config.temporal
        )
        .unwrap()
        .outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Eligible
    );
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .support = rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Event {
        event_id: "event-b".into(),
        start_s: 0.0,
        end_s: 1.0,
    };
    assert_eq!(
        rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
            &request,
            &preparation.bundle,
            &preparation.temporal_metadata,
            &config.temporal
        )
        .unwrap()
        .outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Ineligible
    );
}
#[test]
fn phase_b_temporal_clock_conflict_is_typed() {
    let mut preparation = prepared_sources(true);
    let left = EvidenceId("estimation.point.0.state.0".into());
    let right = EvidenceId("transient.event.0.tau_fast_s".into());
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .support = rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Point {
        timestamp_s: 5.0,
    };
    preparation
        .temporal_metadata
        .entries
        .get_mut(&left)
        .unwrap()
        .clock_id = Some(rust_electroanalysis_cli::mechanism::temporal::ClockId(
        "a".into(),
    ));
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .clock_id = Some(rust_electroanalysis_cli::mechanism::temporal::ClockId(
        "b".into(),
    ));
    let config = phase_b_context().config;
    let request = rust_electroanalysis_cli::mechanism::temporal::TemporalJoinRequest {
        requirement_id: "clock".into(),
        left_evidence_id: left,
        right_evidence_id: right,
        mode: rust_electroanalysis_cli::mechanism::config::TemporalJoinMode::PointPoint,
    };
    let result = rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
        &request,
        &preparation.bundle,
        &preparation.temporal_metadata,
        &config.temporal,
    )
    .unwrap();
    assert_eq!(
        result.outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Indeterminate
    );
    assert!(result.reasons.contains(
        &rust_electroanalysis_cli::mechanism::temporal::TemporalJoinReasonCode::ClockMismatch
    ));
}
#[test]
fn phase_b_temporal_scope_conflict_is_typed() {
    let mut preparation = prepared_sources(true);
    let left = EvidenceId("estimation.point.0.state.0".into());
    let right = EvidenceId("transient.event.0.tau_fast_s".into());
    preparation
        .temporal_metadata
        .entries
        .get_mut(&right)
        .unwrap()
        .support = rust_electroanalysis_cli::mechanism::temporal::EvidenceTemporalSupport::Point {
        timestamp_s: 5.0,
    };
    preparation
        .bundle
        .records
        .iter_mut()
        .find(|r| r.evidence_id == right)
        .unwrap()
        .experiment_scope = rust_electroanalysis_cli::evidence::EvidenceExperimentScope::Unknown;
    let config = phase_b_context().config;
    let request = rust_electroanalysis_cli::mechanism::temporal::TemporalJoinRequest {
        requirement_id: "scope".into(),
        left_evidence_id: left,
        right_evidence_id: right,
        mode: rust_electroanalysis_cli::mechanism::config::TemporalJoinMode::PointPoint,
    };
    let result = rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
        &request,
        &preparation.bundle,
        &preparation.temporal_metadata,
        &config.temporal,
    )
    .unwrap();
    assert_eq!(
        result.outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Indeterminate
    );
    assert!(result.reasons.contains(
        &rust_electroanalysis_cli::mechanism::temporal::TemporalJoinReasonCode::ScopeMismatch
    ));
}
#[test]
fn phase_b_temporal_aggregate_unknown_is_missing_evidence() {
    let preparation = prepared_sources(true);
    let left = EvidenceId("eis.parameter.0".into());
    let right = EvidenceId("transient.event.0.tau_fast_s".into());
    let config = phase_b_context().config;
    let result = rust_electroanalysis_cli::mechanism::temporal::evaluate_temporal_join(
        &rust_electroanalysis_cli::mechanism::temporal::TemporalJoinRequest {
            requirement_id: "aggregate".into(),
            left_evidence_id: left,
            right_evidence_id: right,
            mode: rust_electroanalysis_cli::mechanism::config::TemporalJoinMode::PointPoint,
        },
        &preparation.bundle,
        &preparation.temporal_metadata,
        &config.temporal,
    )
    .unwrap();
    assert_eq!(
        result.outcome,
        rust_electroanalysis_cli::mechanism::temporal::TemporalJoinOutcome::Indeterminate
    );
}
#[test]
fn phase_b_timescale_independent_pair_is_strong() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let pair = &h.pair_requirements[0];
    let left = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.left_requirement_id)
        .unwrap();
    let right = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.right_requirement_id)
        .unwrap();
    let a = rust_electroanalysis_cli::mechanism::timescale::evaluate_timescale_requirement(
        h,
        pair,
        (left, right),
        &ctx.preparation.bundle,
        &ctx.config.timescale,
    )
    .unwrap();
    assert_eq!(
        a.status,
        rust_electroanalysis_cli::mechanism::timescale::TimescaleStatus::Satisfied
    );
    assert_eq!(a.evidence_ids.len(), 2);
}
#[test]
fn phase_b_timescale_dependent_pair_with_covariance_is_strong() {
    let mut ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let pair = &h.pair_requirements[0];
    let left = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.left_requirement_id)
        .unwrap();
    let right = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.right_requirement_id)
        .unwrap();
    let key = rust_electroanalysis_cli::evidence::EvidencePairKey::canonical(
        left.support_evidence_ids[0].clone(),
        right.support_evidence_ids[0].clone(),
    )
    .unwrap();
    ctx.preparation
        .bundle
        .independence_assessments
        .iter_mut()
        .find(|x| x.pair == key)
        .unwrap()
        .classification =
        rust_electroanalysis_cli::evidence::EvidenceIndependence::PartiallyDependent;
    let source = ctx
        .preparation
        .bundle
        .records
        .iter()
        .find(|r| r.evidence_id == left.support_evidence_ids[0])
        .unwrap()
        .source
        .clone();
    ctx.preparation.bundle.timescale_pair_uncertainties.push(rust_electroanalysis_cli::evidence::TimescalePairUncertainty { pair: key, covariance: rust_electroanalysis_cli::evidence::TimescaleCrossCovariance::LogSpace { covariance_ln_tau: 0.0 }, source: rust_electroanalysis_cli::evidence::TimescalePairUncertaintySource { source_artifact: source.artifact, left_source_field_path: source.field_path.clone(), right_source_field_path: source.field_path.clone(), covariance_source_field_path: source.field_path, derivation: rust_electroanalysis_cli::evidence::PairCovarianceDerivation::PreservedProducerCovariance } });
    assert_eq!(
        rust_electroanalysis_cli::mechanism::timescale::evaluate_timescale_requirement(
            h,
            pair,
            (left, right),
            &ctx.preparation.bundle,
            &ctx.config.timescale
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::timescale::TimescaleStatus::Satisfied
    );
}
#[test]
fn phase_b_timescale_dependent_pair_without_covariance_is_not_assessed() {
    let mut ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let pair = &h.pair_requirements[0];
    let left = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.left_requirement_id)
        .unwrap();
    let right = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.right_requirement_id)
        .unwrap();
    let key = rust_electroanalysis_cli::evidence::EvidencePairKey::canonical(
        left.support_evidence_ids[0].clone(),
        right.support_evidence_ids[0].clone(),
    )
    .unwrap();
    ctx.preparation
        .bundle
        .independence_assessments
        .iter_mut()
        .find(|x| x.pair == key)
        .unwrap()
        .classification =
        rust_electroanalysis_cli::evidence::EvidenceIndependence::PartiallyDependent;
    assert_eq!(
        rust_electroanalysis_cli::mechanism::timescale::evaluate_timescale_requirement(
            h,
            pair,
            (left, right),
            &ctx.preparation.bundle,
            &ctx.config.timescale
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::timescale::TimescaleStatus::NotAssessed
    );
}
#[test]
fn phase_b_timescale_missing_covariance_is_not_assessed() {
    let mut ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let pair = &h.pair_requirements[0];
    let left = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.left_requirement_id)
        .unwrap();
    let right = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.right_requirement_id)
        .unwrap();
    let key = rust_electroanalysis_cli::evidence::EvidencePairKey::canonical(
        left.support_evidence_ids[0].clone(),
        right.support_evidence_ids[0].clone(),
    )
    .unwrap();
    ctx.preparation
        .bundle
        .independence_assessments
        .iter_mut()
        .find(|x| x.pair == key)
        .unwrap()
        .classification = rust_electroanalysis_cli::evidence::EvidenceIndependence::SameSource;
    assert_eq!(
        rust_electroanalysis_cli::mechanism::timescale::evaluate_timescale_requirement(
            h,
            pair,
            (left, right),
            &ctx.preparation.bundle,
            &ctx.config.timescale
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::timescale::TimescaleStatus::NotAssessed
    );
}
#[test]
fn phase_b_timescale_strong_boundary_is_inclusive() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let pair = &h.pair_requirements[0];
    let left = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.left_requirement_id)
        .unwrap();
    let right = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.right_requirement_id)
        .unwrap();
    assert_eq!(
        rust_electroanalysis_cli::mechanism::timescale::evaluate_timescale_requirement(
            h,
            pair,
            (left, right),
            &ctx.preparation.bundle,
            &ctx.config.timescale
        )
        .unwrap()
        .log_distance,
        Some(h.timescale_gate.as_ref().unwrap().maximum_log_distance)
    );
}
#[test]
fn phase_b_timescale_outside_domain_cannot_promote() {
    let mut ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let pair = &h.pair_requirements[0];
    let id = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == pair.left_requirement_id)
        .unwrap()
        .support_evidence_ids[0]
        .clone();
    ctx.preparation
        .bundle
        .records
        .iter_mut()
        .find(|r| r.evidence_id == id)
        .unwrap()
        .validity = rust_electroanalysis_cli::evidence::EvidenceValidity::OutsideDomain;
    let eligible =
        rust_electroanalysis_cli::mechanism::evidence::evaluate_hypothesis_evidence_eligibility(
            h,
            &ctx.bound,
            &ctx.preparation,
            &ctx.config,
        )
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
    assert_eq!(
        rust_electroanalysis_cli::mechanism::timescale::evaluate_timescale_requirement(
            h,
            pair,
            (left, right),
            &ctx.preparation.bundle,
            &ctx.config.timescale
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::timescale::TimescaleStatus::NotAssessed
    );
}
#[test]
fn phase_b_amplitude_expected_direction_passes() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::AmplitudeGate {
        predicted_requirement_id: "b-eis-tau".into(),
        observed_requirement_id: "b-transient-tau".into(),
        expected_effect: rust_electroanalysis_cli::mechanism::config::ExpectedEffect::SameSign,
        maximum_relative_error: 1.0,
        floor: rust_electroanalysis_cli::mechanism::config::AmplitudeThreshold {
            value: 0.1,
            unit: "s".into(),
        },
    };
    let p = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.predicted_requirement_id)
        .unwrap();
    let o = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.observed_requirement_id)
        .unwrap();
    assert_eq!(
        rust_electroanalysis_cli::mechanism::amplitude::evaluate_amplitude_requirement(
            h,
            &gate,
            (p, o),
            &ctx.preparation.bundle,
            &ctx.config.amplitude
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::amplitude::AmplitudeStatus::Satisfied
    );
}
#[test]
fn phase_b_amplitude_opposite_direction_fails() {
    let mut ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::AmplitudeGate {
        predicted_requirement_id: "b-eis-tau".into(),
        observed_requirement_id: "b-transient-tau".into(),
        expected_effect: rust_electroanalysis_cli::mechanism::config::ExpectedEffect::SameSign,
        maximum_relative_error: 1.0,
        floor: rust_electroanalysis_cli::mechanism::config::AmplitudeThreshold {
            value: 0.1,
            unit: "s".into(),
        },
    };
    let p = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.predicted_requirement_id)
        .unwrap();
    let o = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.observed_requirement_id)
        .unwrap();
    ctx.preparation
        .bundle
        .records
        .iter_mut()
        .find(|r| r.evidence_id == o.support_evidence_ids[0])
        .unwrap()
        .quantity
        .as_mut()
        .unwrap()
        .value = -1.0;
    assert_eq!(
        rust_electroanalysis_cli::mechanism::amplitude::evaluate_amplitude_requirement(
            h,
            &gate,
            (p, o),
            &ctx.preparation.bundle,
            &ctx.config.amplitude
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::amplitude::AmplitudeStatus::Contradicted
    );
}
#[test]
fn phase_b_amplitude_missing_observation_is_not_assessed() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::AmplitudeGate {
        predicted_requirement_id: "b-eis-tau".into(),
        observed_requirement_id: "b-transient-tau".into(),
        expected_effect: rust_electroanalysis_cli::mechanism::config::ExpectedEffect::SameSign,
        maximum_relative_error: 1.0,
        floor: rust_electroanalysis_cli::mechanism::config::AmplitudeThreshold {
            value: 0.1,
            unit: "s".into(),
        },
    };
    let p = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.predicted_requirement_id)
        .unwrap();
    let mut o = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.observed_requirement_id)
        .unwrap()
        .clone();
    o.support_evidence_ids.clear();
    assert_eq!(
        rust_electroanalysis_cli::mechanism::amplitude::evaluate_amplitude_requirement(
            h,
            &gate,
            (p, &o),
            &ctx.preparation.bundle,
            &ctx.config.amplitude
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::amplitude::AmplitudeStatus::NotAssessed
    );
}
#[test]
fn phase_b_amplitude_unit_threshold_and_direction() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::AmplitudeGate {
        predicted_requirement_id: "b-eis-tau".into(),
        observed_requirement_id: "b-transient-tau".into(),
        expected_effect: rust_electroanalysis_cli::mechanism::config::ExpectedEffect::SameSign,
        maximum_relative_error: 0.0,
        floor: rust_electroanalysis_cli::mechanism::config::AmplitudeThreshold {
            value: 1000.0,
            unit: "ms".into(),
        },
    };
    let p = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.predicted_requirement_id)
        .unwrap();
    let o = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == gate.observed_requirement_id)
        .unwrap();
    assert_eq!(
        rust_electroanalysis_cli::mechanism::amplitude::evaluate_amplitude_requirement(
            h,
            &gate,
            (p, o),
            &ctx.preparation.bundle,
            &ctx.config.amplitude
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::amplitude::AmplitudeStatus::Satisfied
    );
}
#[test]
fn phase_b_repeatability_independent_families_pass() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
        requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
        maximum_sample_standard_deviation_ln_tau: 0.0,
        minimum_independent_families: 2,
    };
    let rows = ctx
        .eligible
        .requirements
        .iter()
        .filter(|r| gate.requirement_ids.contains(&r.requirement_id))
        .collect::<Vec<_>>();
    let a = rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(
        h,
        &gate,
        &rows,
        &ctx.preparation.bundle,
        &ctx.config.repeatability,
    )
    .unwrap();
    assert_eq!(
        a.status,
        rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::Satisfied
    );
    assert_eq!(a.sample_standard_deviation_ln_tau, Some(0.0));
}
#[test]
fn phase_b_repeatability_shared_family_is_not_assessed() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
        requirement_ids: vec!["b-eis-tau".into()],
        maximum_sample_standard_deviation_ln_tau: 0.0,
        minimum_independent_families: 2,
    };
    let row = ctx
        .eligible
        .requirements
        .iter()
        .find(|r| r.requirement_id == "b-eis-tau")
        .unwrap();
    assert_eq!(
        rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(
            h,
            &gate,
            &[row, row],
            &ctx.preparation.bundle,
            &ctx.config.repeatability
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::NotAssessed
    );
}
#[test]
fn phase_b_repeatability_one_family_is_not_assessed() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
        requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
        maximum_sample_standard_deviation_ln_tau: 0.0,
        minimum_independent_families: 3,
    };
    let rows = ctx
        .eligible
        .requirements
        .iter()
        .filter(|r| gate.requirement_ids.contains(&r.requirement_id))
        .collect::<Vec<_>>();
    assert_eq!(
        rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(
            h,
            &gate,
            &rows,
            &ctx.preparation.bundle,
            &ctx.config.repeatability
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::NotAssessed
    );
}
#[test]
fn phase_b_repeatability_unknown_family_is_not_assessed() {
    let mut ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
        requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
        maximum_sample_standard_deviation_ln_tau: 0.0,
        minimum_independent_families: 2,
    };
    let rows = ctx
        .eligible
        .requirements
        .iter()
        .filter(|r| gate.requirement_ids.contains(&r.requirement_id))
        .collect::<Vec<_>>();
    let id = rows[0].support_evidence_ids[0].clone();
    ctx.preparation
        .bundle
        .records
        .iter_mut()
        .find(|r| r.evidence_id == id)
        .unwrap()
        .source
        .artifact = rust_electroanalysis_cli::evidence::EvidenceArtifactSource::LegacyUnknown {
        artifact_kind: rust_electroanalysis_cli::domain::ArtifactKind::EisFit,
        source_fingerprint: rust_electroanalysis_cli::evidence::LegacySourceFingerprint::from_bytes(
            b"unknown",
        ),
    };
    assert_eq!(
        rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(
            h,
            &gate,
            &rows,
            &ctx.preparation.bundle,
            &ctx.config.repeatability
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::NotAssessed
    );
}
#[test]
fn phase_b_repeatability_uses_sample_sd_and_independent_families() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gate = rust_electroanalysis_cli::mechanism::config::RepeatabilityGate {
        requirement_ids: vec!["b-eis-tau".into(), "b-transient-tau".into()],
        maximum_sample_standard_deviation_ln_tau: -0.1,
        minimum_independent_families: 2,
    };
    let rows = ctx
        .eligible
        .requirements
        .iter()
        .filter(|r| gate.requirement_ids.contains(&r.requirement_id))
        .collect::<Vec<_>>();
    assert_eq!(
        rust_electroanalysis_cli::mechanism::repeatability::evaluate_repeatability_requirement(
            h,
            &gate,
            &rows,
            &ctx.preparation.bundle,
            &ctx.config.repeatability
        )
        .unwrap()
        .status,
        rust_electroanalysis_cli::mechanism::repeatability::RepeatabilityStatus::Failed
    );
}
#[test]
fn phase_b_identifiability_covariate_satisfies() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let b = &h.identifiability_bindings[0];
    assert_eq!(rust_electroanalysis_cli::mechanism::identifiability::evaluate_identifiability_binding(h,b,&ctx.eligible,&ctx.preparation.bundle,&ctx.preparation.bundle.independence_assessments,&ctx.config.identifiability).unwrap().status,rust_electroanalysis_cli::mechanism::identifiability::IdentifiabilityAssessmentStatus::Satisfied);
}
#[test]
fn phase_b_identifiability_covariate_below_range_fails() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let mut b = h.identifiability_bindings[0].clone();
    b.threshold = 2.0;
    assert_eq!(rust_electroanalysis_cli::mechanism::identifiability::evaluate_identifiability_binding(h,&b,&ctx.eligible,&ctx.preparation.bundle,&ctx.preparation.bundle.independence_assessments,&ctx.config.identifiability).unwrap().status,rust_electroanalysis_cli::mechanism::identifiability::IdentifiabilityAssessmentStatus::NotSatisfied);
}
#[test]
fn phase_b_identifiability_missing_source_is_not_assessed() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let mut b = h.identifiability_bindings[0].clone();
    b.input.requirement_ids = vec!["missing".into()];
    assert_eq!(rust_electroanalysis_cli::mechanism::identifiability::evaluate_identifiability_binding(h,&b,&ctx.eligible,&ctx.preparation.bundle,&ctx.preparation.bundle.independence_assessments,&ctx.config.identifiability).unwrap().status,rust_electroanalysis_cli::mechanism::identifiability::IdentifiabilityAssessmentStatus::NotAssessed);
}
#[test]
fn phase_b_identifiability_custom_is_not_assessed() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let mut b = h.identifiability_bindings[0].clone();
    b.input.selection =
        rust_electroanalysis_cli::mechanism::config::IdentifiabilityInputSelection::AllEligible;
    assert_eq!(rust_electroanalysis_cli::mechanism::identifiability::evaluate_identifiability_binding(h,&b,&ctx.eligible,&ctx.preparation.bundle,&ctx.preparation.bundle.independence_assessments,&ctx.config.identifiability).unwrap().status,rust_electroanalysis_cli::mechanism::identifiability::IdentifiabilityAssessmentStatus::NotAssessed);
}
#[test]
fn phase_b_validation_passes_and_promotes_domain() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let a = rust_electroanalysis_cli::mechanism::validation::evaluate_validation_protocol(
        h,
        &ctx.eligible,
        &ctx.bound.role_bindings,
        &ctx.preparation.bundle,
        ctx.config.validation.as_ref(),
    )
    .unwrap();
    assert_eq!(a.status, ValidationProtocolStatus::Satisfied);
    assert_eq!(a.acquisition_family_ids.len(), 2);
}
#[test]
fn phase_b_validation_insufficient_families_is_typed() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let mut protocol = ctx.config.validation.clone().unwrap();
    protocol.minimum_acquisition_families = 3;
    assert_eq!(
        rust_electroanalysis_cli::mechanism::validation::evaluate_validation_protocol(
            h,
            &ctx.eligible,
            &ctx.bound.role_bindings,
            &ctx.preparation.bundle,
            Some(&protocol)
        )
        .unwrap()
        .status,
        ValidationProtocolStatus::NotSatisfied
    );
}
#[test]
fn phase_b_validation_unknown_family_is_typed() {
    let mut ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let id = ctx
        .bound
        .role_bindings
        .iter()
        .find(|x| {
            x.role == rust_electroanalysis_cli::mechanism::config::MechanismEvidenceRole::Validation
        })
        .unwrap()
        .evidence_id
        .clone();
    ctx.preparation
        .bundle
        .records
        .iter_mut()
        .find(|x| x.evidence_id == id)
        .unwrap()
        .source
        .artifact = rust_electroanalysis_cli::evidence::EvidenceArtifactSource::LegacyUnknown {
        artifact_kind: rust_electroanalysis_cli::domain::ArtifactKind::CalibrationObservations,
        source_fingerprint: rust_electroanalysis_cli::evidence::LegacySourceFingerprint::from_bytes(
            b"unknown",
        ),
    };
    assert_eq!(
        rust_electroanalysis_cli::mechanism::validation::evaluate_validation_protocol(
            h,
            &ctx.eligible,
            &ctx.bound.role_bindings,
            &ctx.preparation.bundle,
            ctx.config.validation.as_ref()
        )
        .unwrap()
        .status,
        ValidationProtocolStatus::NotSatisfied
    );
}
#[test]
fn phase_b_validation_training_overlap_is_typed() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let mut roles = ctx.bound.role_bindings.clone();
    let mut overlap = roles
        .iter()
        .find(|x| {
            x.role == rust_electroanalysis_cli::mechanism::config::MechanismEvidenceRole::Validation
        })
        .unwrap()
        .clone();
    overlap.role = rust_electroanalysis_cli::mechanism::config::MechanismEvidenceRole::Training;
    roles.push(overlap);
    assert_eq!(
        rust_electroanalysis_cli::mechanism::validation::evaluate_validation_protocol(
            h,
            &ctx.eligible,
            &roles,
            &ctx.preparation.bundle,
            ctx.config.validation.as_ref()
        )
        .unwrap()
        .status,
        ValidationProtocolStatus::NotSatisfied
    );
}
#[test]
fn phase_b_strong_critical_contradiction_blocks_before_support_filtering() {
    let ctx = phase_b_context();
    let h = &ctx.config.hypotheses[0];
    let gates = rust_electroanalysis_cli::mechanism::promotion::HypothesisGateAssessments {
        contradiction_summaries: vec![
            rust_electroanalysis_cli::mechanism::evidence::RequirementContradictionSummary {
                requirement_id: "b-eis-tau".into(),
                evidence_ids: vec![EvidenceId("critical".into())],
                contradiction_count: 1,
                strong_critical_count: 1,
            },
        ],
        timescale_assessments: vec![],
        amplitude_assessments: vec![],
        repeatability_assessments: vec![],
        identifiability_assessments: vec![],
        validation_assessment: None,
    };
    assert_eq!(
        rust_electroanalysis_cli::mechanism::promotion::assess_hypothesis(
            h,
            &ctx.eligible,
            &gates,
            &ctx.config
        )
        .unwrap()
        .evidence_level,
        HypothesisEvidenceLevel::Contradicted
    );
}
#[test]
fn phase_b_schema3_hypotheses_migrate_to_legacy_hypotheses() {
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true, None);
    let mut value = serde_json::to_value(report).unwrap();
    value["schema_version"] = serde_json::json!(3);
    value["artifact_kind"] = serde_json::json!("mechanism_analysis");
    let legacy = value
        .as_object_mut()
        .unwrap()
        .remove("legacy_hypotheses")
        .unwrap();
    value["hypotheses"] = legacy;
    let path = std::env::temp_dir().join(format!("phase-b-schema3-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    let report: rust_electroanalysis_cli::results::MechanismAnalysisReport =
        rust_electroanalysis_cli::domain::read_artifact(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(report.legacy_hypotheses.is_empty());
}
#[test]
fn phase_b_schema3_to_schema4_preserves_legacy_hypotheses() {
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true, None);
    let value = serde_json::to_value(report).unwrap();
    assert!(value.get("legacy_hypotheses").is_some());
}
#[test]
fn phase_b_schema4_writer_emits_legacy_hypotheses() {
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true, None);
    let value = serde_json::to_value(report).unwrap();
    assert!(value.get("hypotheses").is_none());
}
#[test]
fn phase_b_assessment_hash_rejects_non_finite_float() {
    let mut current = assessment(
        "non-finite",
        HypothesisEvidenceLevel::NotAssessed,
        ValidationProtocolStatus::NotAssessed,
        vec![],
    );
    current.timescale_assessments.push(
        rust_electroanalysis_cli::mechanism::timescale::TimescaleAssessment {
            pair_requirement_id: "pair".into(),
            status: rust_electroanalysis_cli::mechanism::timescale::TimescaleStatus::Failed,
            evidence_ids: vec![],
            log_distance: Some(f64::NAN),
        },
    );
    let mut gate_assessments = gates(None);
    gate_assessments.timescale_assessments = current.timescale_assessments.clone();
    let view =
        build_hypothesis_assessment_hash_view(&current, &gate_assessments, &[], &[]).unwrap();
    assert!(compute_assessment_hash(&view).is_err());
}
#[test]
fn phase_b_history_duplicate_suppression_uses_semantic_identity() {
    let current = assessment(
        "duplicate",
        HypothesisEvidenceLevel::ExperimentallySupported,
        ValidationProtocolStatus::NotAssessed,
        vec![],
    );
    let first = rust_electroanalysis_cli::mechanism::history::update_hypothesis_history(
        &[],
        &current,
        &gates(None),
        &[],
        &[],
    )
    .unwrap();
    let second = rust_electroanalysis_cli::mechanism::history::update_hypothesis_history(
        &first,
        &current,
        &gates(None),
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(first, second);
}
#[test]
fn phase_b_fx09_history_hash_matches_canonical_view() {
    let report = run_phase_b_cli("config/e2e_experimentally_supported.toml", false, None);
    assert_eq!(
        report.hypothesis_assessments[0].current.evidence_level,
        HypothesisEvidenceLevel::ExperimentallySupported
    );
    assert!(report.hypothesis_history.is_empty());
}
#[test]
fn phase_b_fx10_validation_payload_reaches_history_hash() {
    let prior = run_phase_b_cli("config/e2e_experimentally_supported.toml", false, None);
    let path = std::env::temp_dir().join(format!("phase-b-prior-{}.json", std::process::id()));
    rust_electroanalysis_cli::domain::write_artifact(&path, &prior).unwrap();
    let report = run_phase_b_cli("config/e2e_validated_for_domain.toml", true, Some(&path));
    std::fs::remove_file(path).unwrap();
    let current = &report.hypothesis_assessments[0].current;
    assert_eq!(
        current.evidence_level,
        HypothesisEvidenceLevel::ValidatedForDomain
    );
    assert_eq!(report.hypothesis_history.len(), 1);
    assert_eq!(
        report.hypothesis_history[0].prior_level,
        HypothesisEvidenceLevel::ExperimentallySupported
    );
    assert_eq!(
        report.hypothesis_history[0].new_level,
        HypothesisEvidenceLevel::ValidatedForDomain
    );
    assert!(
        report.hypothesis_history[0]
            .source_evidence_ids
            .iter()
            .any(|id| id.0 == "calibration.observation.0")
    );
}
#[test]
fn phase_b_fx10_history_hash_matches_canonical_validation_view() {
    let current = assessment(
        "history-order",
        HypothesisEvidenceLevel::ValidatedForDomain,
        ValidationProtocolStatus::Satisfied,
        vec![PhaseBHypothesisReasonCode::ValidationSatisfied],
    );
    let make_validation = |ids: Vec<EvidenceId>| ValidationAssessment {
        protocol_id: "protocol".into(),
        status: ValidationProtocolStatus::Satisfied,
        evidence_ids: ids,
        acquisition_family_ids: vec!["b".into(), "a".into()],
        passed_condition_ids: vec!["z".into(), "a".into()],
        reasons: vec![ValidationReasonCode::Passed],
    };
    let left = build_hypothesis_assessment_hash_view(
        &current,
        &gates(Some(make_validation(vec![
            EvidenceId("b".into()),
            EvidenceId("a".into()),
        ]))),
        &[],
        &[EvidenceId("a".into()), EvidenceId("b".into())],
    )
    .unwrap();
    let right = build_hypothesis_assessment_hash_view(
        &current,
        &gates(Some(make_validation(vec![
            EvidenceId("a".into()),
            EvidenceId("b".into()),
        ]))),
        &[],
        &[EvidenceId("b".into()), EvidenceId("a".into())],
    )
    .unwrap();
    assert_eq!(
        compute_assessment_hash(&left).unwrap(),
        compute_assessment_hash(&right).unwrap()
    );
}
