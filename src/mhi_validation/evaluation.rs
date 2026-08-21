//! Filesystem-free projection of frozen Phase-B and Phase-C outcomes.
//!
//! The reader supplies already-validated artifacts.  This module consumes
//! their serialized states and reference labels; it never calls an assessor.

use super::{MhiValidationError, MhiValidationProtocolV1};
use crate::{
    mechanism::promotion::HypothesisEvidenceLevel,
    mhi_validation::{
        reader::ValidationInputs,
        statistics::{MetricValueV1, balanced_accuracy, wilson_95_checked},
    },
    results::{
        MechanismReferenceOutcomeV1, ReferenceEndpointV1, ReferenceUncertaintyV1,
        ReleaseClaimResultV1, ValidationRecordV1,
    },
    validation_config::{
        AcceptanceRuleV1, BlindingRuleV1, CohortRoleV1, ComparatorV1, CountMetricV1,
        HealthEndpointV1, HealthTargetV1, MechanismEndpointV1, RateMetricV1, RateTargetV1,
        ReferenceAuthorityRuleV1, ReferenceUncertaintyRuleV1, ReleaseClaimOutcomeV1,
        RequestedValidationLevelV1, ValidationOutcomeV1,
    },
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub fn evaluate_mhi_validation(
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
) -> Result<crate::results::MhiValidationReportV1, MhiValidationError> {
    let mut outcomes = Vec::new();
    let mut endpoint_results = Vec::new();
    for endpoint in &protocol.mechanism_endpoints {
        let (outcome, projection) = mechanism_endpoint(endpoint, inputs)?;
        outcomes.push((endpoint.endpoint_id.clone(), outcome));
        endpoint_results.push(projection);
    }
    for endpoint in &protocol.health_endpoints {
        let (outcome, projection) = health_endpoint(endpoint, inputs)?;
        outcomes.push((endpoint.endpoint_id.clone(), outcome));
        endpoint_results.push(projection);
    }
    let release_claims = protocol
        .release_scope
        .iter()
        .map(|claim| {
            let status = compose(claim.supporting_endpoint_ids.iter().filter_map(|id| {
                outcomes
                    .iter()
                    .find(|(endpoint_id, _)| endpoint_id == id)
                    .map(|(_, outcome)| *outcome)
            }));
            ReleaseClaimResultV1 {
                claim_id: claim.claim_id.clone(),
                requested_level: claim.requested_level,
                outcome: match (claim.requested_level, status) {
                    (RequestedValidationLevelV1::Software, ValidationOutcomeV1::MeetsProtocol) => {
                        ReleaseClaimOutcomeV1::SoftwareValidatedOnly
                    }
                    (RequestedValidationLevelV1::Physical, ValidationOutcomeV1::MeetsProtocol) => {
                        ReleaseClaimOutcomeV1::PhysicallyValidated
                    }
                    (_, ValidationOutcomeV1::DoesNotMeetProtocol) => {
                        ReleaseClaimOutcomeV1::DoesNotMeetProtocol
                    }
                    (_, ValidationOutcomeV1::Indeterminate) => ReleaseClaimOutcomeV1::Indeterminate,
                },
            }
        })
        .collect::<Vec<_>>();
    let overall_status = compose(outcomes.iter().map(|(_, outcome)| *outcome));
    let dataset = &inputs.dataset.artifact;
    let payload = json!({
        "endpoint_results": endpoint_results,
        "lineage_catalog_source_file_sha256": inputs.lineage_catalog.source_file_sha256,
        "mechanism_source_count": inputs.mechanism_sources.len(),
        "health_source_count": inputs.health_sources.len(),
    });
    let preimage = serde_jcs::to_vec(&json!({
        "identity_domain": "mhi_validation_report_v1",
        "protocol_sha256": dataset.protocol_sha256,
        "dataset_id": dataset.dataset_id,
        "payload": payload,
        "release_claims": release_claims,
        "overall_status": overall_status,
    }))
    .map_err(|error| MhiValidationError::Dataset(error.to_string()))?;
    let mut hash = Sha256::new();
    hash.update(preimage);
    let report = crate::results::MhiValidationReportV1 {
        schema_version: 1,
        artifact_kind: "mhi_validation_report".into(),
        report_id: format!("sha256:{:x}", hash.finalize()),
        protocol_sha256: dataset.protocol_sha256.clone(),
        dataset_id: dataset.dataset_id.clone(),
        dataset_source_file_sha256: inputs.dataset.source_file_sha256.clone(),
        approval_trust_store_sha256: None,
        release_claims,
        overall_status,
        payload,
        lineage: dataset.lineage.clone(),
        provenance: json!({ "software_version": env!("CARGO_PKG_VERSION") }),
        warnings: Vec::new(),
    };
    report.validate_structure()?;
    Ok(report)
}

fn mechanism_endpoint(
    endpoint: &MechanismEndpointV1,
    inputs: &ValidationInputs,
) -> Result<(ValidationOutcomeV1, Value), MhiValidationError> {
    let mut eligible = Vec::new();
    let mut excluded = Vec::new();
    let mut support = Vec::new();
    let mut contradictions = Vec::new();
    let mut other = Vec::new();
    let mut declared_falsifications = Vec::new();
    let mut families = BTreeSet::new();
    let mut accounting = Vec::new();
    let declared = members(endpoint.cohort_role, &endpoint.domain, inputs);
    for record in &declared {
        let Some(reference) = matching_reference(record, &endpoint.endpoint_id) else {
            exclude(
                record,
                "missing_reference_endpoint",
                &mut excluded,
                &mut accounting,
            );
            continue;
        };
        if record.mechanism_source.is_none() {
            exclude(
                record,
                "missing_endpoint_artifact_path",
                &mut excluded,
                &mut accounting,
            );
            continue;
        }
        if let Some(reason) = reference_exclusion(&endpoint.reference_rule, reference)? {
            exclude(record, reason, &mut excluded, &mut accounting);
            continue;
        }
        let ReferenceEndpointV1::Mechanism {
            hypothesis_id,
            outcome: reference_outcome,
            ..
        } = reference
        else {
            return Err(MhiValidationError::Dataset(
                "ReferenceEndpointBindingMismatch".into(),
            ));
        };
        if hypothesis_id != &endpoint.hypothesis_id {
            return Err(MhiValidationError::Dataset(
                "ReferenceEndpointBindingMismatch".into(),
            ));
        }
        if *reference_outcome == MechanismReferenceOutcomeV1::Unavailable {
            exclude(
                record,
                "reference_outcome_unavailable",
                &mut excluded,
                &mut accounting,
            );
            continue;
        }
        let Some((_, report)) = inputs
            .mechanism_sources
            .iter()
            .find(|(id, _)| id == &record.record_id)
        else {
            return Err(MhiValidationError::Dataset(
                "strict mechanism source was not retained".into(),
            ));
        };
        let level = phase_b_level(report, &endpoint.hypothesis_id)?;
        eligible.push(record.record_id.clone());
        record_families(record, &mut families);
        accounting.push(json!({"record_id":record.record_id,"decision":"eligible","primary_reason":Value::Null,"secondary_reasons":[]}));
        let critical = level == HypothesisEvidenceLevel::Contradicted
            || *reference_outcome == MechanismReferenceOutcomeV1::Contradicts;
        if critical {
            contradictions.push(record.record_id.clone());
            declared_falsifications.push(record.record_id.clone());
        } else if endpoint
            .support_levels
            .iter()
            .any(|token| token == level_token(level.clone()))
            && *reference_outcome == MechanismReferenceOutcomeV1::Supports
        {
            support.push(record.record_id.clone());
        } else {
            other.push(record.record_id.clone());
        }
    }
    let total = eligible.len() as u64;
    let support_rate = rate(support.len() as u64, total)?;
    let contradiction_rate = rate(contradictions.len() as u64, total)?;
    let other_rate = rate(other.len() as u64, total)?;
    let metrics = json!({
        "support_fraction": support_rate,
        "contradiction_fraction": contradiction_rate,
        "not_assessed_fraction": other_rate,
    });
    let (rule_false, rule_unavailable, rule_evaluations) = evaluate_rules(
        &endpoint.acceptance_rules,
        &[
            (CountMetricV1::DeclaredCount, declared.len() as u64),
            (CountMetricV1::EligibleCount, total),
            (CountMetricV1::ExcludedCount, excluded.len() as u64),
            (CountMetricV1::IndependentFamilyCount, families.len() as u64),
            (CountMetricV1::SupportCount, support.len() as u64),
            (
                CountMetricV1::CriticalContradictionCount,
                contradictions.len() as u64,
            ),
            (CountMetricV1::NotAssessedOrOtherCount, other.len() as u64),
        ],
        &[
            (
                RateMetricV1::SupportFraction,
                rate(support.len() as u64, total)?,
            ),
            (
                RateMetricV1::ContradictionFraction,
                rate(contradictions.len() as u64, total)?,
            ),
            (
                RateMetricV1::NotAssessedFraction,
                rate(other.len() as u64, total)?,
            ),
            (
                RateMetricV1::ExclusionRate,
                rate(excluded.len() as u64, declared.len() as u64)?,
            ),
        ],
        None,
    );
    let outcome = if !declared_falsifications.is_empty() {
        ValidationOutcomeV1::DoesNotMeetProtocol
    } else if total < endpoint.minimum_eligible_records
        || (families.len() as u64) < endpoint.minimum_independent_families
        || rule_unavailable
    {
        ValidationOutcomeV1::Indeterminate
    } else if rule_false {
        ValidationOutcomeV1::DoesNotMeetProtocol
    } else {
        ValidationOutcomeV1::MeetsProtocol
    };
    Ok((
        outcome,
        json!({
            "endpoint_id":endpoint.endpoint_id,
            "endpoint_kind":"mechanism",
            "declared_record_ids": ids(&declared),
            "eligible_record_ids":eligible,
            "excluded_record_ids":excluded,
            "record_accounting":accounting,
            "independent_family_ids":families,
            "support_record_ids":support,
            "critical_contradiction_record_ids":contradictions,
            "declared_critical_falsification_record_ids":declared_falsifications,
            "not_assessed_or_other_record_ids":other,
            "metrics":metrics,"rule_evaluations":rule_evaluations,
            "outcome":outcome,
        }),
    ))
}

fn health_endpoint(
    endpoint: &HealthEndpointV1,
    inputs: &ValidationInputs,
) -> Result<(ValidationOutcomeV1, Value), MhiValidationError> {
    let mut eligible = Vec::new();
    let mut excluded = Vec::new();
    let mut tp = Vec::new();
    let mut tn = Vec::new();
    let mut fp = Vec::new();
    let mut fn_ids = Vec::new();
    let mut indeterminate = Vec::new();
    let mut dqi = Vec::new();
    let mut families = BTreeSet::new();
    let mut accounting = Vec::new();
    let declared = members(endpoint.cohort_role, &endpoint.domain, inputs);
    for record in &declared {
        let Some(reference) = matching_reference(record, &endpoint.endpoint_id) else {
            exclude(
                record,
                "missing_reference_endpoint",
                &mut excluded,
                &mut accounting,
            );
            continue;
        };
        if record.health_source.is_none() {
            exclude(
                record,
                "missing_endpoint_artifact_path",
                &mut excluded,
                &mut accounting,
            );
            continue;
        }
        if let Some(reason) = reference_exclusion(&endpoint.reference_rule, reference)? {
            exclude(record, reason, &mut excluded, &mut accounting);
            continue;
        }
        let ReferenceEndpointV1::Health { target, label, .. } = reference else {
            return Err(MhiValidationError::Dataset(
                "ReferenceEndpointBindingMismatch".into(),
            ));
        };
        if target != &endpoint.target
            || !endpoint
                .reference_label_universe
                .iter()
                .any(|allowed| allowed == label)
        {
            return Err(MhiValidationError::Dataset(
                "ReferenceEndpointBindingMismatch".into(),
            ));
        }
        let Some((_, assessment)) = inputs
            .health_sources
            .iter()
            .find(|(id, _)| id == &record.record_id)
        else {
            return Err(MhiValidationError::Dataset(
                "strict health source was not retained".into(),
            ));
        };
        let status = phase_c_status(assessment, &endpoint.target)?;
        eligible.push(record.record_id.clone());
        record_families(record, &mut families);
        accounting.push(json!({"record_id":record.record_id,"decision":"eligible","primary_reason":Value::Null,"secondary_reasons":[]}));
        if status == "indeterminate" {
            indeterminate.push(record.record_id.clone());
            continue;
        }
        if status == "data_quality_insufficient" {
            dqi.push(record.record_id.clone());
            continue;
        }
        let predicted_positive = endpoint
            .predicted_positive_statuses
            .iter()
            .any(|value| value == status);
        let reference_positive = endpoint
            .reference_positive_labels
            .iter()
            .any(|value| value == label);
        match (predicted_positive, reference_positive) {
            (true, true) => tp.push(record.record_id.clone()),
            (true, false) => fp.push(record.record_id.clone()),
            (false, true) => fn_ids.push(record.record_id.clone()),
            (false, false) => tn.push(record.record_id.clone()),
        }
    }
    let total = eligible.len() as u64;
    let tp_n = tp.len() as u64;
    let tn_n = tn.len() as u64;
    let fp_n = fp.len() as u64;
    let fn_n = fn_ids.len() as u64;
    let coverage = rate(tp_n + tn_n + fp_n + fn_n, total)?;
    let indeterminate_rate = rate(indeterminate.len() as u64, total)?;
    let dqi_rate = rate(dqi.len() as u64, total)?;
    let sensitivity = rate_with_reason(tp_n, tp_n + fn_n, "positive_class_denominator_zero")?;
    let specificity = rate_with_reason(tn_n, tn_n + fp_n, "negative_class_denominator_zero")?;
    let false_positive_rate =
        rate_with_reason(fp_n, fp_n + tn_n, "negative_class_denominator_zero")?;
    let false_negative_rate =
        rate_with_reason(fn_n, fn_n + tp_n, "positive_class_denominator_zero")?;
    let metrics = json!({
        "coverage":coverage,"indeterminate_rate":indeterminate_rate,"data_quality_insufficient_rate":dqi_rate,
        "sensitivity":sensitivity,"specificity":specificity,"false_positive_rate":false_positive_rate,"false_negative_rate":false_negative_rate,
    });
    let balanced = balanced_accuracy(tp_n, tn_n, fp_n, fn_n).ok();
    let (rule_false, rule_unavailable, rule_evaluations) = evaluate_rules(
        &endpoint.acceptance_rules,
        &[
            (CountMetricV1::DeclaredCount, declared.len() as u64),
            (CountMetricV1::EligibleCount, total),
            (CountMetricV1::ExcludedCount, excluded.len() as u64),
            (CountMetricV1::IndependentFamilyCount, families.len() as u64),
            (CountMetricV1::Tp, tp_n),
            (CountMetricV1::Tn, tn_n),
            (CountMetricV1::Fp, fp_n),
            (CountMetricV1::Fn, fn_n),
            (
                CountMetricV1::IndeterminateCount,
                indeterminate.len() as u64,
            ),
            (
                CountMetricV1::DataQualityInsufficientCount,
                dqi.len() as u64,
            ),
            (CountMetricV1::EvaluableCount, tp_n + tn_n + fp_n + fn_n),
        ],
        &[
            (
                RateMetricV1::Coverage,
                rate(tp_n + tn_n + fp_n + fn_n, total)?,
            ),
            (
                RateMetricV1::IndeterminateRate,
                rate(indeterminate.len() as u64, total)?,
            ),
            (
                RateMetricV1::DataQualityInsufficientRate,
                rate(dqi.len() as u64, total)?,
            ),
            (
                RateMetricV1::Sensitivity,
                rate_with_reason(tp_n, tp_n + fn_n, "positive_class_denominator_zero")?,
            ),
            (
                RateMetricV1::Specificity,
                rate_with_reason(tn_n, tn_n + fp_n, "negative_class_denominator_zero")?,
            ),
            (
                RateMetricV1::FalsePositiveRate,
                rate_with_reason(fp_n, fp_n + tn_n, "negative_class_denominator_zero")?,
            ),
            (
                RateMetricV1::FalseNegativeRate,
                rate_with_reason(fn_n, fn_n + tp_n, "positive_class_denominator_zero")?,
            ),
            (
                RateMetricV1::ExclusionRate,
                rate(excluded.len() as u64, declared.len() as u64)?,
            ),
        ],
        balanced,
    );
    let outcome = if total < endpoint.minimum_eligible_records
        || (families.len() as u64) < endpoint.minimum_independent_families
        || rule_unavailable
    {
        ValidationOutcomeV1::Indeterminate
    } else if rule_false {
        ValidationOutcomeV1::DoesNotMeetProtocol
    } else {
        ValidationOutcomeV1::MeetsProtocol
    };
    Ok((
        outcome,
        json!({
            "endpoint_id":endpoint.endpoint_id,"endpoint_kind":"health","declared_record_ids":ids(&declared),
            "eligible_record_ids":eligible,"excluded_record_ids":excluded,"record_accounting":accounting,"independent_family_ids":families,
            "tp_record_ids":tp,"tn_record_ids":tn,"fp_record_ids":fp,"fn_record_ids":fn_ids,
            "indeterminate_record_ids":indeterminate,"data_quality_insufficient_record_ids":dqi,
            "metrics":metrics,"balanced_accuracy":balanced,"rule_evaluations":rule_evaluations,"outcome":outcome,
        }),
    ))
}

fn members<'a>(
    role: CohortRoleV1,
    domain: &crate::validation_config::DomainSelectorV1,
    inputs: &'a ValidationInputs,
) -> Vec<&'a ValidationRecordV1> {
    inputs
        .dataset
        .artifact
        .records
        .iter()
        .filter(|record| record.cohort_role == role && domain.contains(&record.domain))
        .collect()
}
fn ids(records: &[&ValidationRecordV1]) -> Vec<String> {
    records
        .iter()
        .map(|record| record.record_id.clone())
        .collect()
}
fn record_families(record: &ValidationRecordV1, families: &mut BTreeSet<String>) {
    if let crate::domain::ArtifactAcquisitionFamilies::Known(values) =
        &record.declared_scope.acquisition_families
    {
        families.extend(values.iter().map(|value| value.0.clone()));
    }
}
fn exclude(
    record: &ValidationRecordV1,
    reason: &str,
    excluded: &mut Vec<String>,
    accounting: &mut Vec<Value>,
) {
    excluded.push(record.record_id.clone());
    accounting.push(json!({"record_id":record.record_id,"decision":"excluded","primary_reason":reason,"secondary_reasons":[]}));
}
fn matching_reference<'a>(
    record: &'a ValidationRecordV1,
    endpoint_id: &str,
) -> Option<&'a ReferenceEndpointV1> {
    record
        .reference_endpoints
        .iter()
        .find(|reference| match reference {
            ReferenceEndpointV1::Mechanism {
                endpoint_id: id, ..
            }
            | ReferenceEndpointV1::Health {
                endpoint_id: id, ..
            } => id == endpoint_id,
        })
}

fn phase_b_level(
    report: &crate::domain::StrictArtifactRead<crate::results::MechanismAnalysisReport>,
    hypothesis_id: &str,
) -> Result<HypothesisEvidenceLevel, MhiValidationError> {
    let found = report
        .artifact
        .hypothesis_assessments
        .iter()
        .filter(|row| row.definition.hypothesis_id == hypothesis_id)
        .collect::<Vec<_>>();
    if found.is_empty() {
        return Ok(HypothesisEvidenceLevel::NotAssessed);
    }
    if found.len() != 1 || found[0].current.hypothesis_id != hypothesis_id {
        return Err(MhiValidationError::Dataset(
            "Phase-B hypothesis ID mismatch or duplicate".into(),
        ));
    }
    Ok(found[0].current.evidence_level.clone())
}
fn phase_c_status(
    assessment: &crate::domain::StrictArtifactRead<crate::results::SensorHealthAssessment>,
    target: &HealthTargetV1,
) -> Result<&'static str, MhiValidationError> {
    use crate::results::OverallHealthStatus;
    let status = match target {
        HealthTargetV1::Aggregate => assessment.artifact.overall_status,
        HealthTargetV1::Dimension { dimension_id } => {
            let phase_c = assessment.artifact.phase_c.as_ref().ok_or_else(|| {
                MhiValidationError::Dataset("schema-4 health source lacks Phase-C evidence".into())
            })?;
            let rows = phase_c
                .dimension_assessments
                .iter()
                .filter(|row| {
                    serde_json::to_value(row.dimension)
                        .ok()
                        .as_ref()
                        .and_then(Value::as_str)
                        == Some(dimension_id)
                })
                .collect::<Vec<_>>();
            if rows.len() != 1 {
                return Err(MhiValidationError::Dataset(
                    "health target lacks unique Phase-C dimension".into(),
                ));
            }
            rows[0].status
        }
    };
    Ok(match status {
        OverallHealthStatus::WithinBaseline => "within_baseline",
        OverallHealthStatus::Watch => "watch",
        OverallHealthStatus::Degraded => "degraded",
        OverallHealthStatus::Critical => "critical",
        OverallHealthStatus::DataQualityInsufficient => "data_quality_insufficient",
        OverallHealthStatus::Indeterminate => "indeterminate",
    })
}
fn level_token(level: HypothesisEvidenceLevel) -> &'static str {
    match level {
        HypothesisEvidenceLevel::NotAssessed => "not_assessed",
        HypothesisEvidenceLevel::Hypothesized => "hypothesized",
        HypothesisEvidenceLevel::ExperimentallySupported => "experimentally_supported",
        HypothesisEvidenceLevel::ValidatedForDomain => "validated_for_domain",
        HypothesisEvidenceLevel::Contradicted => "contradicted",
    }
}

fn reference_exclusion(
    rule: &ReferenceAuthorityRuleV1,
    reference: &ReferenceEndpointV1,
) -> Result<Option<&'static str>, MhiValidationError> {
    let (methods, authorities, blinding_rule, uncertainty_rule) = match rule {
        ReferenceAuthorityRuleV1::Mechanism {
            allowed_methods,
            allowed_authority_ids,
            blinding_rule,
            uncertainty_rule,
        }
        | ReferenceAuthorityRuleV1::Health {
            allowed_methods,
            allowed_authority_ids,
            blinding_rule,
            uncertainty_rule,
        } => (
            allowed_methods,
            allowed_authority_ids,
            blinding_rule,
            uncertainty_rule,
        ),
    };
    let (method_id, method_version, authority_id, blinding, uncertainty) = match reference {
        ReferenceEndpointV1::Mechanism {
            method_id,
            method_version,
            authority_id,
            blinding_state,
            uncertainty,
            ..
        }
        | ReferenceEndpointV1::Health {
            method_id,
            method_version,
            authority_id,
            blinding_state,
            uncertainty,
            ..
        } => (
            method_id,
            method_version,
            authority_id,
            blinding_state,
            uncertainty,
        ),
    };
    if !methods
        .iter()
        .any(|method| method.method_id == *method_id && method.method_version == *method_version)
    {
        return Ok(Some("reference_method_not_allowed"));
    }
    if !authorities
        .iter()
        .any(|authority| authority == authority_id)
    {
        return Ok(Some("reference_authority_not_allowed"));
    }
    if *blinding_rule == BlindingRuleV1::RequireBlinded
        && *blinding != crate::validation_config::BlindingStateV1::BlindedToAssessment
    {
        return Ok(Some("reference_blinding_not_allowed"));
    }
    match (uncertainty_rule, uncertainty) {
        (
            ReferenceUncertaintyRuleV1::RequireQuantified { .. },
            ReferenceUncertaintyV1::Unavailable { .. },
        ) => Ok(Some("reference_uncertainty_unavailable")),
        (
            ReferenceUncertaintyRuleV1::RequireQuantified { measure_id, .. },
            ReferenceUncertaintyV1::Quantified {
                measure_id: actual, ..
            },
        ) if measure_id != actual => Ok(Some("reference_uncertainty_measure_mismatch")),
        (
            ReferenceUncertaintyRuleV1::RequireQuantified { unit, .. },
            ReferenceUncertaintyV1::Quantified { unit: actual, .. },
        ) if unit != actual => Ok(Some("reference_uncertainty_unit_mismatch")),
        (
            ReferenceUncertaintyRuleV1::RequireQuantified {
                maximum_inclusive, ..
            },
            ReferenceUncertaintyV1::Quantified { value, .. },
        ) if value > maximum_inclusive => Ok(Some("reference_uncertainty_above_maximum")),
        _ => Ok(None),
    }
}
fn rate(numerator: u64, denominator: u64) -> Result<MetricValueV1, MhiValidationError> {
    wilson_95_checked(numerator, denominator)
        .map_err(|reason| MhiValidationError::Dataset(reason.into()))
}
fn rate_with_reason(
    numerator: u64,
    denominator: u64,
    reason: &str,
) -> Result<MetricValueV1, MhiValidationError> {
    Ok(match rate(numerator, denominator)? {
        MetricValueV1::Unavailable {
            numerator,
            denominator,
            ..
        } => MetricValueV1::Unavailable {
            numerator,
            denominator,
            reason: reason.into(),
        },
        value => value,
    })
}

fn evaluate_rules(
    rules: &[AcceptanceRuleV1],
    counts: &[(CountMetricV1, u64)],
    rates: &[(RateMetricV1, MetricValueV1)],
    balanced_accuracy: Option<f64>,
) -> (bool, bool, Vec<Value>) {
    let mut false_seen = false;
    let mut unavailable_seen = false;
    let evaluations = rules
        .iter()
        .map(|rule| {
            let (actual, result) = match rule {
                AcceptanceRuleV1::Count {
                    metric,
                    comparator,
                    threshold_u64,
                    ..
                } => {
                    let value = counts
                        .iter()
                        .find(|(kind, _)| kind == metric)
                        .map(|(_, value)| *value);
                    match value {
                        Some(value) => (
                            json!({"type":"count","value":value}),
                            compare(value as f64, *threshold_u64 as f64, *comparator),
                        ),
                        None => (json!({"type":"unavailable"}), None),
                    }
                }
                AcceptanceRuleV1::Rate {
                    metric,
                    target,
                    comparator,
                    threshold,
                    ..
                } => {
                    if *metric == RateMetricV1::BalancedAccuracy {
                        match balanced_accuracy {
                            Some(value) => (
                                json!({"type":"balanced_accuracy","value":value}),
                                compare(value, *threshold, *comparator),
                            ),
                            None => (json!({"type":"unavailable"}), None),
                        }
                    } else {
                        let metric_value = rates
                            .iter()
                            .find(|(kind, _)| kind == metric)
                            .map(|(_, value)| value);
                        let value = metric_value.and_then(|value| rate_target(value, *target));
                        match value {
                            Some(value) => (
                                json!({"type":"binomial_rate","value":value}),
                                compare(value, *threshold, *comparator),
                            ),
                            None => (json!({"type":"unavailable"}), None),
                        }
                    }
                }
            };
            let result = match result {
                Some(true) => "true",
                Some(false) => {
                    false_seen = true;
                    "false"
                }
                None => {
                    unavailable_seen = true;
                    "unavailable"
                }
            };
            json!({"rule":rule,"actual":actual,"result":result})
        })
        .collect();
    (false_seen, unavailable_seen, evaluations)
}

fn rate_target(value: &MetricValueV1, target: RateTargetV1) -> Option<f64> {
    match value {
        MetricValueV1::Available {
            point_estimate,
            lower_confidence_bound,
            upper_confidence_bound,
            ..
        } => Some(match target {
            RateTargetV1::PointEstimate => *point_estimate,
            RateTargetV1::LowerConfidenceBound => *lower_confidence_bound,
            RateTargetV1::UpperConfidenceBound => *upper_confidence_bound,
        }),
        MetricValueV1::Unavailable { .. } => None,
    }
}

fn compare(actual: f64, threshold: f64, comparator: ComparatorV1) -> Option<bool> {
    if !actual.is_finite() || !threshold.is_finite() {
        return None;
    }
    Some(match comparator {
        ComparatorV1::GreaterThanOrEqual => actual >= threshold,
        ComparatorV1::LessThanOrEqual => actual <= threshold,
    })
}

fn compose(outcomes: impl Iterator<Item = ValidationOutcomeV1>) -> ValidationOutcomeV1 {
    let outcomes = outcomes.collect::<Vec<_>>();
    if outcomes.contains(&ValidationOutcomeV1::Indeterminate) {
        ValidationOutcomeV1::Indeterminate
    } else if outcomes.contains(&ValidationOutcomeV1::DoesNotMeetProtocol) {
        ValidationOutcomeV1::DoesNotMeetProtocol
    } else {
        ValidationOutcomeV1::MeetsProtocol
    }
}
