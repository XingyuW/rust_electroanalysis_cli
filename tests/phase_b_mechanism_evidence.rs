use rust_electroanalysis_cli::{
    evidence::EvidenceId,
    mechanism::{
        history::{
            build_hypothesis_assessment_hash_view, compute_assessment_hash, compute_history_id,
            HypothesisHistoryIdView,
        },
        promotion::{
            HypothesisEvidenceLevel, HypothesisGateAssessments, PhaseBHypothesisReasonCode,
        },
        validation::{ValidationAssessment, ValidationProtocolStatus, ValidationReasonCode},
    },
    results::PhaseBHypothesisAssessment,
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
    assert_eq!(hash.0, "7dc65e83a79a145ef083c78750674eb27927af7757c9a42f504270ecdc544290");
    assert_eq!(
        compute_history_id(&HypothesisHistoryIdView {
            hypothesis_id: "pb-hash-01".into(),
            prior_level: HypothesisEvidenceLevel::Hypothesized,
            new_level: HypothesisEvidenceLevel::NotAssessed,
            assessment_hash: hash.0,
        })
        .unwrap(),
        "7a1d581a1e9bccc4cf21503b4f9f4766a19a11086b8faba8c257edff4ef54d0f"
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
    let view = build_hypothesis_assessment_hash_view(&current, &gates(Some(validation)), &[], &source)
        .unwrap();
    let hash = compute_assessment_hash(&view).unwrap();
    assert_eq!(hash.0, "6a540a332d57d763cefaa05ba46a663ba97e019649df1d531e8c430da047d4ec");
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
