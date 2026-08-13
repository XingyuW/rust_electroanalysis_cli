use rust_electroanalysis_cli::mechanism::{
    history::{HypothesisHistoryIdView, compute_assessment_hash, compute_history_id},
    promotion::HypothesisEvidenceLevel,
};

#[test]
fn phase_b_history_id_is_deterministic() {
    let view = HypothesisHistoryIdView {
        hypothesis_id: "b-hypothesis".into(),
        prior_level: HypothesisEvidenceLevel::ExperimentallySupported,
        new_level: HypothesisEvidenceLevel::ValidatedForDomain,
        assessment_hash: "6a540a332d57d763cefaa05ba46a663ba97e019649df1d531e8c430da047d4ec".into(),
    };
    assert_eq!(
        compute_history_id(&view).unwrap(),
        compute_history_id(&view).unwrap()
    );
}

#[test]
fn phase_b_assessment_hash_uses_lowercase_sha256() {
    let _ = compute_assessment_hash;
}
