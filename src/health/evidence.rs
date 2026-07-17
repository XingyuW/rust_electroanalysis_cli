use crate::results::{HealthConfidence, HealthEvidence, HealthFinding};
pub fn lower_confidence(finding: &mut HealthFinding, contradictory: Vec<HealthEvidence>) {
    finding.contradictory_evidence.extend(contradictory);
    finding.confidence = match finding.confidence {
        HealthConfidence::High => HealthConfidence::Moderate,
        HealthConfidence::Moderate => HealthConfidence::Low,
        other => other,
    };
}
