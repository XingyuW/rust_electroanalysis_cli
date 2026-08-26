//! Filesystem-free reconstruction of frozen Phase-B/Phase-C outcomes.

use super::{
    MhiValidationError, MhiValidationProtocolV1,
    partition::{self, EndpointPartitionSpec, EndpointSource, PartitionRowV1},
    reader::ValidationInputs,
    statistics::{MetricValueV1, wilson_95_checked},
};
use crate::{
    domain::{ArtifactAcquisitionFamilies, ArtifactLineageState},
    mechanism::promotion::HypothesisEvidenceLevel,
    results::{
        ApprovalAuthorityV1, ArtifactSourceExpectationV1, BalancedAccuracyV1, CohortRowV1,
        CompatibilityResultV1, CompatibilityRowV1, CompatibilitySourceRoleV1, DatasetAuthorityV1,
        DatasetSourceReferenceV1, ExclusionRowV1, HealthResultV1, ImmutableDocumentReferenceV1,
        MechanismReferenceOutcomeV1, MechanismResultV1, MhiValidationReportV1, OutcomeReasonV1,
        ProtocolAuthorityV1, RecordAccountingRowV1, ReferenceEndpointV1, ReferenceUncertaintyV1,
        ReleaseClaimResultV1, RuleActualV1, RuleEvaluationV1, SourceReferenceV1,
        ValidationProvenanceV1, ValidationWarningCodeV1, ValidationWarningV1,
    },
    validation_config::{
        AcceptanceRuleV1, CohortRoleV1, CountMetricV1, EndpointKindV1, HealthEndpointV1,
        HealthTargetV1, MechanismEndpointV1, RateMetricV1, RateTargetV1, RecordDecisionV1,
        RequestedValidationLevelV1, RuleEvaluationResultV1, SeparationStatusV1,
        ValidationOutcomeV1,
    },
};
use std::collections::BTreeSet;

pub fn evaluate_mhi_validation(
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
) -> Result<MhiValidationReportV1, MhiValidationError> {
    for (_, source) in &inputs.mechanism_sources {
        super::reader::validate_phase_b_assessment_integrity(&source.artifact)?;
    }
    let physical_endpoints = protocol
        .release_scope
        .iter()
        .filter(|claim| claim.requested_level == RequestedValidationLevelV1::Physical)
        .flat_map(|claim| claim.supporting_endpoint_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let mut accounting = Vec::new();
    let mut cohorts = Vec::new();
    let mut leakage = Vec::new();
    let mut mechanism_results = Vec::new();
    let mut health_results = Vec::new();
    let mut endpoint_outcomes = Vec::new();
    for endpoint in &protocol.mechanism_endpoints {
        let result = evaluate_mechanism(
            endpoint,
            inputs,
            physical_endpoints.contains(&endpoint.endpoint_id),
        )?;
        endpoint_outcomes.push((endpoint.endpoint_id.clone(), result.overall));
        accounting.extend(result.accounting);
        cohorts.extend(result.cohorts);
        leakage.extend(result.leakage);
        mechanism_results.extend(result.results);
    }
    for endpoint in &protocol.health_endpoints {
        let result = evaluate_health(
            endpoint,
            inputs,
            physical_endpoints.contains(&endpoint.endpoint_id),
        )?;
        endpoint_outcomes.push((endpoint.endpoint_id.clone(), result.overall));
        accounting.extend(result.accounting);
        cohorts.extend(result.cohorts);
        leakage.extend(result.leakage);
        health_results.extend(result.results);
    }
    accounting.sort_by(row_key);
    leakage.sort_by(leakage_key);
    cohorts.sort_by(cohort_key);
    mechanism_results.sort_by(mechanism_key);
    health_results.sort_by(health_key);
    let exclusions = accounting
        .iter()
        .filter_map(|row| {
            Some(ExclusionRowV1 {
                endpoint_id: row.endpoint_id.clone(),
                stratum_id: row.stratum_id.clone(),
                record_id: row.record_id.clone(),
                primary_reason: row.primary_reason?,
                secondary_reasons: row.secondary_reasons.clone(),
                assessed_source_key: row.assessed_source_key.clone(),
                reference_endpoint_id: row.reference_endpoint_id.clone(),
            })
        })
        .collect::<Vec<_>>();
    let release_claims = protocol
        .release_scope
        .iter()
        .map(|claim| {
            let outcome = compose(claim.supporting_endpoint_ids.iter().filter_map(|id| {
                endpoint_outcomes
                    .iter()
                    .find(|(endpoint, _)| endpoint == id)
                    .map(|(_, value)| *value)
            }));
            ReleaseClaimResultV1 {
                claim_id: claim.claim_id.clone(),
                requested_level: claim.requested_level,
                statement: claim.statement.clone(),
                domain: claim.domain.clone(),
                supporting_endpoint_ids: claim.supporting_endpoint_ids.clone(),
                outcome: match (claim.requested_level, outcome) {
                    (RequestedValidationLevelV1::Software, ValidationOutcomeV1::MeetsProtocol) => {
                        crate::validation_config::ReleaseClaimOutcomeV1::SoftwareValidatedOnly
                    }
                    (RequestedValidationLevelV1::Physical, ValidationOutcomeV1::MeetsProtocol) => {
                        crate::validation_config::ReleaseClaimOutcomeV1::PhysicallyValidated
                    }
                    (_, ValidationOutcomeV1::DoesNotMeetProtocol) => {
                        crate::validation_config::ReleaseClaimOutcomeV1::DoesNotMeetProtocol
                    }
                    (_, ValidationOutcomeV1::Indeterminate) => {
                        crate::validation_config::ReleaseClaimOutcomeV1::Indeterminate
                    }
                },
            }
        })
        .collect::<Vec<_>>();
    let dataset_source = dataset_source(inputs)?;
    let mut report = MhiValidationReportV1 {
        schema_version: 1,
        artifact_kind: "mhi_validation_report".into(),
        report_id: String::new(),
        protocol: ProtocolAuthorityV1 {
            protocol_id: protocol.protocol_id.clone(),
            schema_version: 1,
            source_file_sha256: inputs.protocol_sha256.clone(),
            registration: protocol.registration.clone(),
            physical_approval_authority: protocol.physical_approval_authority.clone(),
            normalized_protocol: protocol.clone(),
        },
        dataset: DatasetAuthorityV1 {
            dataset_id: inputs.dataset.artifact.dataset_id.clone(),
            schema_version: 1,
            protocol_sha256: inputs.protocol_sha256.clone(),
            cohort_semantic_sha256: inputs.dataset.artifact.cohort_semantic_sha256.clone(),
            source: dataset_source.clone(),
        },
        approval: approval_authority(protocol, inputs)?,
        compatibility: compatibility(inputs),
        record_accounting: accounting,
        cohorts,
        leakage_assessment: leakage,
        mechanism_results,
        health_results,
        exclusions,
        release_claims,
        overall_status: compose(endpoint_outcomes.iter().map(|(_, outcome)| *outcome)),
        lineage: inputs.dataset.artifact.lineage.clone(),
        provenance: ValidationProvenanceV1 {
            software_version: env!("CARGO_PKG_VERSION").into(),
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
            protocol_sha256: inputs.dataset.artifact.protocol_sha256.clone(),
            dataset_source,
            consumed_sources: consumed_sources(inputs),
        },
        warnings: warnings(inputs),
    };
    report.report_id = report.computed_report_id()?;
    report.validate_structure()?;
    Ok(report)
}

impl MhiValidationReportV1 {
    /// Authority-assisted replay.  Unlike `validate_structure`, this reruns
    /// the closed partition and the frozen Phase-B/C mappings from the exact
    /// reader-retained inputs, then compares every typed field.
    pub fn validate_against(
        &self,
        protocol: &MhiValidationProtocolV1,
        inputs: &ValidationInputs,
    ) -> Result<(), MhiValidationError> {
        self.validate_structure()?;
        protocol.validate()?;
        if self.protocol.normalized_protocol != *protocol
            || self.protocol.source_file_sha256 != inputs.protocol_sha256
            || self.dataset.dataset_id != inputs.dataset.artifact.dataset_id
            || self.dataset.cohort_semantic_sha256 != inputs.dataset.artifact.cohort_semantic_sha256
        {
            return Err(MhiValidationError::Dataset(
                "report authority does not bind the replay inputs".into(),
            ));
        }
        let physical = protocol
            .release_scope
            .iter()
            .any(|claim| claim.requested_level == RequestedValidationLevelV1::Physical);
        if physical && (inputs.owner_approval.is_none() || self.approval.is_none()) {
            return Err(MhiValidationError::Approval(
                "physical report replay requires verified approval authority".into(),
            ));
        }
        if let (Some(approval), Some(verified)) = (&self.approval, &inputs.owner_approval)
            && approval.trust_store_sha256 != verified.trust_store_sha256()
        {
            return Err(MhiValidationError::Approval(
                "report approval trust-store hash differs from the replay authority".into(),
            ));
        }
        let rebuilt = evaluate_mhi_validation(protocol, inputs)?;
        if &rebuilt != self {
            return Err(MhiValidationError::Dataset(
                "report differs from authority-assisted replay".into(),
            ));
        }
        Ok(())
    }
}

struct EndpointEvaluation<T> {
    accounting: Vec<RecordAccountingRowV1>,
    cohorts: Vec<CohortRowV1>,
    leakage: Vec<crate::results::LeakageRowV1>,
    results: Vec<T>,
    overall: ValidationOutcomeV1,
}

fn evaluate_mechanism(
    endpoint: &MechanismEndpointV1,
    inputs: &ValidationInputs,
    physical: bool,
) -> Result<EndpointEvaluation<MechanismResultV1>, MhiValidationError> {
    let partition = partition::partition_endpoint(
        inputs,
        EndpointPartitionSpec {
            endpoint_id: &endpoint.endpoint_id,
            cohort_role: endpoint.cohort_role,
            domain: &endpoint.domain,
            required_strata: &endpoint.required_strata,
            reference_rule: &endpoint.reference_rule,
            source: EndpointSource::Mechanism,
            physical,
        },
    )?;
    let mut results = Vec::new();
    let mut cohorts = Vec::new();
    for view in endpoint_views(&endpoint.required_strata) {
        let rows = rows_for(&partition.rows, &view);
        let eligible = ids(rows
            .iter()
            .copied()
            .filter(|row| row.decision == RecordDecisionV1::Eligible));
        let declared = ids(rows
            .iter()
            .copied()
            .filter(|row| row.decision != RecordDecisionV1::NotApplicable));
        let excluded = ids(rows
            .iter()
            .copied()
            .filter(|row| row.decision == RecordDecisionV1::Excluded));
        let mut support = Vec::new();
        let mut contradiction = Vec::new();
        let mut other = Vec::new();
        let mut declared_falsifications = Vec::new();
        let mut families = BTreeSet::new();
        let mut limitations = Vec::new();
        for row in &rows {
            if row.decision != RecordDecisionV1::NotApplicable
                && let Some((_, source)) = inputs
                    .mechanism_sources
                    .iter()
                    .find(|(id, _)| id == &row.record_id)
            {
                let level = phase_b_level(source, &endpoint.hypothesis_id)?;
                let reference =
                    record(&inputs.dataset.artifact.records, &row.record_id).and_then(|record| {
                        partition::matching_reference(record, &endpoint.endpoint_id)
                    });
                let contradicted = level == HypothesisEvidenceLevel::Contradicted
                    || matches!(
                        reference,
                        Some(ReferenceEndpointV1::Mechanism {
                            outcome: MechanismReferenceOutcomeV1::Contradicts,
                            ..
                        })
                    );
                if contradicted {
                    declared_falsifications.push(row.record_id.clone());
                }
            }
            if row.decision != RecordDecisionV1::Eligible {
                continue;
            }
            let record = record(&inputs.dataset.artifact.records, &row.record_id)
                .expect("partition row record");
            add_families(record, &mut families);
            let source = &inputs
                .mechanism_sources
                .iter()
                .find(|(id, _)| id == &row.record_id)
                .ok_or_else(|| {
                    MhiValidationError::Dataset("strict mechanism source was not retained".into())
                })?
                .1;
            let level = phase_b_level(source, &endpoint.hypothesis_id)?;
            let reference = partition::matching_reference_exact(record, &endpoint.endpoint_id)?
                .ok_or_else(|| {
                    MhiValidationError::Dataset("eligible mechanism row lacks reference".into())
                })?;
            let ReferenceEndpointV1::Mechanism {
                hypothesis_id,
                outcome,
                limitations: source_limitations,
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
            limitations.extend(source_limitations.iter().cloned());
            if level == HypothesisEvidenceLevel::Contradicted
                || *outcome == MechanismReferenceOutcomeV1::Contradicts
            {
                contradiction.push(row.record_id.clone());
            } else if *outcome == MechanismReferenceOutcomeV1::Supports
                && endpoint
                    .support_levels
                    .iter()
                    .any(|allowed| allowed == level_token(level.clone()))
            {
                support.push(row.record_id.clone());
            } else {
                other.push(row.record_id.clone());
            }
        }
        dedup(&mut support);
        dedup(&mut contradiction);
        dedup(&mut other);
        dedup(&mut declared_falsifications);
        dedup(&mut limitations);
        let eligible_count = eligible.len() as u64;
        let support_fraction = rate(support.len() as u64, eligible_count, "denominator_zero")?;
        let contradiction_fraction = rate(
            contradiction.len() as u64,
            eligible_count,
            "denominator_zero",
        )?;
        let other_fraction = rate(other.len() as u64, eligible_count, "denominator_zero")?;
        let unavailable_references =
            unavailable_reference_ids(&rows, inputs, &endpoint.endpoint_id);
        let (rule_false, rule_unavailable, rule_evaluations) = evaluate_rules(
            &endpoint.acceptance_rules,
            &[
                (CountMetricV1::DeclaredCount, declared.len() as u64),
                (CountMetricV1::EligibleCount, eligible_count),
                (CountMetricV1::ExcludedCount, excluded.len() as u64),
                (CountMetricV1::IndependentFamilyCount, families.len() as u64),
                (CountMetricV1::SupportCount, support.len() as u64),
                (
                    CountMetricV1::CriticalContradictionCount,
                    contradiction.len() as u64,
                ),
                (CountMetricV1::NotAssessedOrOtherCount, other.len() as u64),
            ],
            &[
                (
                    RateMetricV1::ExclusionRate,
                    rate(
                        excluded.len() as u64,
                        declared.len() as u64,
                        "denominator_zero",
                    )?,
                ),
                (RateMetricV1::SupportFraction, support_fraction.clone()),
                (
                    RateMetricV1::ContradictionFraction,
                    contradiction_fraction.clone(),
                ),
                (RateMetricV1::NotAssessedFraction, other_fraction.clone()),
            ],
            None,
        );
        let reasons = endpoint_reasons(OutcomeContext {
            rows: &rows,
            holdout: endpoint.cohort_role == CohortRoleV1::Holdout,
            falsifications: &declared_falsifications,
            unavailable_references: &unavailable_references,
            eligible_count,
            minimum_eligible: endpoint.minimum_eligible_records,
            family_count: families.len() as u64,
            minimum_families: endpoint.minimum_independent_families,
            rule_unavailable,
            rule_false,
            rules: &rule_evaluations,
        });
        let outcome = outcome_for(&reasons);
        let independent_family_count = families.len() as u64;
        let result = MechanismResultV1 {
            endpoint_id: endpoint.endpoint_id.clone(),
            stratum_id: view.clone(),
            eligible_record_ids: eligible.clone(),
            eligible_family_ids: families.into_iter().collect(),
            support_record_ids: support.clone(),
            critical_contradiction_record_ids: contradiction.clone(),
            declared_critical_falsification_record_ids: declared_falsifications.clone(),
            not_assessed_or_other_record_ids: other.clone(),
            eligible_count,
            independent_family_count,
            support_count: support.len() as u64,
            critical_contradiction_count: contradiction.len() as u64,
            declared_critical_falsification_count: declared_falsifications.len() as u64,
            not_assessed_or_other_count: other.len() as u64,
            support_fraction,
            contradiction_fraction,
            not_assessed_fraction: other_fraction,
            rule_evaluations,
            outcome_reasons: reasons.clone(),
            limitations,
            outcome,
        };
        cohorts.push(cohort_row(
            inputs,
            CohortDescriptor {
                endpoint_id: &endpoint.endpoint_id,
                endpoint_kind: EndpointKindV1::Mechanism,
                cohort_role: endpoint.cohort_role,
                domain: &endpoint.domain,
                strata: &endpoint.required_strata,
            },
            view,
            &rows,
            outcome,
            None,
        )?);
        results.push(result);
    }
    propagate_required_strata(&mut results, &mut cohorts);
    let overall = results
        .iter()
        .find(|result| result.stratum_id == "overall")
        .expect("overall")
        .outcome;
    Ok(EndpointEvaluation {
        accounting: partition.rows.iter().map(accounting_row).collect(),
        leakage: partition.rows.iter().map(leakage_row).collect(),
        cohorts,
        results,
        overall,
    })
}

fn evaluate_health(
    endpoint: &HealthEndpointV1,
    inputs: &ValidationInputs,
    physical: bool,
) -> Result<EndpointEvaluation<HealthResultV1>, MhiValidationError> {
    let partition = partition::partition_endpoint(
        inputs,
        EndpointPartitionSpec {
            endpoint_id: &endpoint.endpoint_id,
            cohort_role: endpoint.cohort_role,
            domain: &endpoint.domain,
            required_strata: &endpoint.required_strata,
            reference_rule: &endpoint.reference_rule,
            source: EndpointSource::Health,
            physical,
        },
    )?;
    let mut results = Vec::new();
    let mut cohorts = Vec::new();
    for view in endpoint_views(&endpoint.required_strata) {
        let rows = rows_for(&partition.rows, &view);
        let eligible = ids(rows
            .iter()
            .copied()
            .filter(|row| row.decision == RecordDecisionV1::Eligible));
        let declared = ids(rows
            .iter()
            .copied()
            .filter(|row| row.decision != RecordDecisionV1::NotApplicable));
        let excluded = ids(rows
            .iter()
            .copied()
            .filter(|row| row.decision == RecordDecisionV1::Excluded));
        let mut tp = Vec::new();
        let mut tn = Vec::new();
        let mut fp = Vec::new();
        let mut fn_ids = Vec::new();
        let mut indeterminate = Vec::new();
        let mut dqi = Vec::new();
        let mut families = BTreeSet::new();
        let mut limitations = Vec::new();
        for row in rows
            .iter()
            .filter(|row| row.decision == RecordDecisionV1::Eligible)
        {
            let record = record(&inputs.dataset.artifact.records, &row.record_id)
                .expect("partition row record");
            add_families(record, &mut families);
            let source = &inputs
                .health_sources
                .iter()
                .find(|(id, _)| id == &row.record_id)
                .ok_or_else(|| {
                    MhiValidationError::Dataset("strict health source was not retained".into())
                })?
                .1;
            let status = phase_c_status(source, &endpoint.target)?;
            let reference = partition::matching_reference_exact(record, &endpoint.endpoint_id)?
                .ok_or_else(|| {
                    MhiValidationError::Dataset("eligible health row lacks reference".into())
                })?;
            let ReferenceEndpointV1::Health {
                target,
                label,
                limitations: source_limitations,
                ..
            } = reference
            else {
                return Err(MhiValidationError::Dataset(
                    "ReferenceEndpointBindingMismatch".into(),
                ));
            };
            if target != &endpoint.target || !endpoint.reference_label_universe.contains(label) {
                return Err(MhiValidationError::Dataset(
                    "ReferenceEndpointBindingMismatch".into(),
                ));
            }
            limitations.extend(source_limitations.iter().cloned());
            if status == "indeterminate" {
                indeterminate.push(row.record_id.clone());
                continue;
            }
            if status == "data_quality_insufficient" {
                dqi.push(row.record_id.clone());
                continue;
            }
            let predicted_positive = endpoint
                .predicted_positive_statuses
                .iter()
                .any(|value| value == status);
            let reference_positive = endpoint.reference_positive_labels.contains(label);
            match (predicted_positive, reference_positive) {
                (true, true) => tp.push(row.record_id.clone()),
                (true, false) => fp.push(row.record_id.clone()),
                (false, true) => fn_ids.push(row.record_id.clone()),
                (false, false) => tn.push(row.record_id.clone()),
            }
        }
        for ids in [
            &mut tp,
            &mut tn,
            &mut fp,
            &mut fn_ids,
            &mut indeterminate,
            &mut dqi,
        ] {
            dedup(ids);
        }
        dedup(&mut limitations);
        let total = eligible.len() as u64;
        let tp_count = tp.len() as u64;
        let tn_count = tn.len() as u64;
        let fp_count = fp.len() as u64;
        let fn_count = fn_ids.len() as u64;
        let indeterminate_count = indeterminate.len() as u64;
        let dqi_count = dqi.len() as u64;
        let evaluable = tp_count + tn_count + fp_count + fn_count;
        let coverage = rate(evaluable, total, "denominator_zero")?;
        let indeterminate_rate = rate(indeterminate_count, total, "denominator_zero")?;
        let dqi_rate = rate(dqi_count, total, "denominator_zero")?;
        let sensitivity = rate(
            tp_count,
            tp_count + fn_count,
            "positive_class_denominator_zero",
        )?;
        let specificity = rate(
            tn_count,
            tn_count + fp_count,
            "negative_class_denominator_zero",
        )?;
        let fpr = rate(
            fp_count,
            fp_count + tn_count,
            "negative_class_denominator_zero",
        )?;
        let fnr = rate(
            fn_count,
            fn_count + tp_count,
            "positive_class_denominator_zero",
        )?;
        let balanced = balanced_accuracy(&sensitivity, &specificity);
        let unavailable_references =
            unavailable_reference_ids(&rows, inputs, &endpoint.endpoint_id);
        let (rule_false, rule_unavailable, rule_evaluations) = evaluate_rules(
            &endpoint.acceptance_rules,
            &[
                (CountMetricV1::DeclaredCount, declared.len() as u64),
                (CountMetricV1::EligibleCount, total),
                (CountMetricV1::ExcludedCount, excluded.len() as u64),
                (CountMetricV1::IndependentFamilyCount, families.len() as u64),
                (CountMetricV1::Tp, tp_count),
                (CountMetricV1::Tn, tn_count),
                (CountMetricV1::Fp, fp_count),
                (CountMetricV1::Fn, fn_count),
                (CountMetricV1::IndeterminateCount, indeterminate_count),
                (CountMetricV1::DataQualityInsufficientCount, dqi_count),
                (CountMetricV1::EvaluableCount, evaluable),
            ],
            &[
                (
                    RateMetricV1::ExclusionRate,
                    rate(
                        excluded.len() as u64,
                        declared.len() as u64,
                        "denominator_zero",
                    )?,
                ),
                (RateMetricV1::Coverage, coverage.clone()),
                (RateMetricV1::IndeterminateRate, indeterminate_rate.clone()),
                (RateMetricV1::DataQualityInsufficientRate, dqi_rate.clone()),
                (RateMetricV1::Sensitivity, sensitivity.clone()),
                (RateMetricV1::Specificity, specificity.clone()),
                (RateMetricV1::FalsePositiveRate, fpr.clone()),
                (RateMetricV1::FalseNegativeRate, fnr.clone()),
            ],
            Some(balanced.clone()),
        );
        let reasons = endpoint_reasons(OutcomeContext {
            rows: &rows,
            holdout: endpoint.cohort_role == CohortRoleV1::Holdout,
            falsifications: &[],
            unavailable_references: &unavailable_references,
            eligible_count: total,
            minimum_eligible: endpoint.minimum_eligible_records,
            family_count: families.len() as u64,
            minimum_families: endpoint.minimum_independent_families,
            rule_unavailable,
            rule_false,
            rules: &rule_evaluations,
        });
        let outcome = outcome_for(&reasons);
        let result = HealthResultV1 {
            endpoint_id: endpoint.endpoint_id.clone(),
            stratum_id: view.clone(),
            eligible_record_ids: eligible.clone(),
            eligible_family_ids: families.iter().cloned().collect(),
            tp_record_ids: tp.clone(),
            tn_record_ids: tn.clone(),
            fp_record_ids: fp.clone(),
            fn_record_ids: fn_ids.clone(),
            indeterminate_record_ids: indeterminate.clone(),
            data_quality_insufficient_record_ids: dqi.clone(),
            eligible_count: total,
            independent_family_count: families.len() as u64,
            tp: tp_count,
            tn: tn_count,
            fp: fp_count,
            r#fn: fn_count,
            indeterminate: indeterminate_count,
            data_quality_insufficient: dqi_count,
            evaluable,
            coverage: coverage.clone(),
            indeterminate_rate: indeterminate_rate.clone(),
            data_quality_insufficient_rate: dqi_rate.clone(),
            sensitivity,
            specificity,
            false_positive_rate: fpr,
            false_negative_rate: fnr,
            balanced_accuracy: balanced.clone(),
            rule_evaluations,
            outcome_reasons: reasons,
            limitations,
            outcome,
        };
        cohorts.push(cohort_row(
            inputs,
            CohortDescriptor {
                endpoint_id: &endpoint.endpoint_id,
                endpoint_kind: match endpoint.target {
                    HealthTargetV1::Aggregate => EndpointKindV1::HealthAggregate,
                    HealthTargetV1::Dimension { .. } => EndpointKindV1::HealthDimension,
                },
                cohort_role: endpoint.cohort_role,
                domain: &endpoint.domain,
                strata: &endpoint.required_strata,
            },
            view,
            &rows,
            outcome,
            Some((
                evaluable,
                indeterminate_count,
                dqi_count,
                coverage,
                indeterminate_rate,
                dqi_rate,
            )),
        )?);
        results.push(result);
    }
    propagate_required_strata_health(&mut results, &mut cohorts);
    let overall = results
        .iter()
        .find(|result| result.stratum_id == "overall")
        .expect("overall")
        .outcome;
    Ok(EndpointEvaluation {
        accounting: partition.rows.iter().map(accounting_row).collect(),
        leakage: partition.rows.iter().map(leakage_row).collect(),
        cohorts,
        results,
        overall,
    })
}

struct OutcomeContext<'a> {
    rows: &'a [&'a PartitionRowV1],
    holdout: bool,
    falsifications: &'a [String],
    unavailable_references: &'a [String],
    eligible_count: u64,
    minimum_eligible: u64,
    family_count: u64,
    minimum_families: u64,
    rule_unavailable: bool,
    rule_false: bool,
    rules: &'a [RuleEvaluationV1],
}

fn endpoint_reasons(context: OutcomeContext<'_>) -> Vec<OutcomeReasonV1> {
    let mut reasons = Vec::new();
    for row in context
        .rows
        .iter()
        .filter(|row| row.decision != RecordDecisionV1::NotApplicable)
    {
        match row.separation_status {
            Some(SeparationStatusV1::KnownOverlap) if context.holdout => {
                reasons.push(OutcomeReasonV1::HoldoutKnownOverlap {
                    record_id: row.record_id.clone(),
                })
            }
            Some(SeparationStatusV1::UnknownSeparation) if context.holdout => {
                reasons.push(OutcomeReasonV1::HoldoutUnknownSeparation {
                    record_id: row.record_id.clone(),
                })
            }
            _ => {}
        }
    }
    reasons.extend(
        context
            .unavailable_references
            .iter()
            .cloned()
            .map(|record_id| OutcomeReasonV1::ReferenceUncertaintyUnavailable { record_id }),
    );
    reasons.extend(
        context
            .falsifications
            .iter()
            .cloned()
            .map(|record_id| OutcomeReasonV1::DeclaredCriticalFalsification { record_id }),
    );
    if context.eligible_count == 0 {
        reasons.push(OutcomeReasonV1::EmptyView);
    }
    if context.eligible_count < context.minimum_eligible {
        reasons.push(OutcomeReasonV1::EligibleRecordMinimumNotMet {
            actual: context.eligible_count,
            minimum: context.minimum_eligible,
        });
    }
    if context.family_count < context.minimum_families {
        reasons.push(OutcomeReasonV1::IndependentFamilyMinimumNotMet {
            actual: context.family_count,
            minimum: context.minimum_families,
        });
    }
    if context.rule_unavailable {
        reasons.extend(
            context
                .rules
                .iter()
                .filter(|rule| rule.result == RuleEvaluationResultV1::Unavailable)
                .map(|rule| OutcomeReasonV1::RequiredRuleUnavailable {
                    rule_id: rule_id(&rule.rule).into(),
                }),
        );
    }
    if context.rule_false {
        reasons.extend(
            context
                .rules
                .iter()
                .filter(|rule| rule.result == RuleEvaluationResultV1::False)
                .map(|rule| OutcomeReasonV1::RequiredRuleFalse {
                    rule_id: rule_id(&rule.rule).into(),
                }),
        );
    }
    reasons.sort_by(outcome_reason_key);
    reasons.dedup();
    reasons
}

fn outcome_for(reasons: &[OutcomeReasonV1]) -> ValidationOutcomeV1 {
    if reasons.iter().any(|reason| {
        matches!(
            reason,
            OutcomeReasonV1::HoldoutKnownOverlap { .. }
                | OutcomeReasonV1::DeclaredCriticalFalsification { .. }
        )
    }) {
        ValidationOutcomeV1::DoesNotMeetProtocol
    } else if reasons.iter().any(|reason| {
        matches!(
            reason,
            OutcomeReasonV1::HoldoutUnknownSeparation { .. }
                | OutcomeReasonV1::EmptyView
                | OutcomeReasonV1::EligibleRecordMinimumNotMet { .. }
                | OutcomeReasonV1::IndependentFamilyMinimumNotMet { .. }
                | OutcomeReasonV1::RequiredStratumIndeterminate { .. }
                | OutcomeReasonV1::ReferenceUncertaintyUnavailable { .. }
                | OutcomeReasonV1::RequiredRuleUnavailable { .. }
        )
    }) {
        ValidationOutcomeV1::Indeterminate
    } else if reasons
        .iter()
        .any(|reason| matches!(reason, OutcomeReasonV1::RequiredRuleFalse { .. }))
    {
        ValidationOutcomeV1::DoesNotMeetProtocol
    } else {
        ValidationOutcomeV1::MeetsProtocol
    }
}

fn unavailable_reference_ids(
    rows: &[&PartitionRowV1],
    inputs: &ValidationInputs,
    endpoint_id: &str,
) -> Vec<String> {
    let mut ids = rows
        .iter()
        .filter(|row| row.decision == RecordDecisionV1::Eligible)
        .filter_map(|row| {
            let reference = record(&inputs.dataset.artifact.records, &row.record_id)
                .and_then(|record| partition::matching_reference(record, endpoint_id))?;
            let uncertainty = match reference {
                ReferenceEndpointV1::Mechanism { uncertainty, .. }
                | ReferenceEndpointV1::Health { uncertainty, .. } => uncertainty,
            };
            matches!(uncertainty, ReferenceUncertaintyV1::Unavailable { .. })
                .then(|| row.record_id.clone())
        })
        .collect::<Vec<_>>();
    dedup(&mut ids);
    ids
}

struct CohortDescriptor<'a> {
    endpoint_id: &'a str,
    endpoint_kind: EndpointKindV1,
    cohort_role: CohortRoleV1,
    domain: &'a crate::validation_config::DomainSelectorV1,
    strata: &'a [crate::validation_config::RequiredStratumV1],
}

fn cohort_row(
    inputs: &ValidationInputs,
    descriptor: CohortDescriptor<'_>,
    stratum_id: String,
    rows: &[&PartitionRowV1],
    outcome: ValidationOutcomeV1,
    health: Option<(u64, u64, u64, MetricValueV1, MetricValueV1, MetricValueV1)>,
) -> Result<CohortRowV1, MhiValidationError> {
    let declared = ids(rows
        .iter()
        .copied()
        .filter(|row| row.decision != RecordDecisionV1::NotApplicable));
    let eligible = ids(rows
        .iter()
        .copied()
        .filter(|row| row.decision == RecordDecisionV1::Eligible));
    let excluded = ids(rows
        .iter()
        .copied()
        .filter(|row| row.decision == RecordDecisionV1::Excluded));
    let not_applicable = ids(rows
        .iter()
        .copied()
        .filter(|row| row.decision == RecordDecisionV1::NotApplicable));
    let view = descriptor
        .strata
        .iter()
        .find(|stratum| stratum.stratum_id == stratum_id);
    let roles = |role| {
        inputs
            .dataset
            .artifact
            .records
            .iter()
            .filter(|record| {
                record.cohort_role == role
                    && descriptor.domain.contains(&record.domain)
                    && view.is_none_or(|stratum| {
                        stratum
                            .predicates
                            .iter()
                            .all(|predicate| predicate.contains(&record.domain))
                    })
            })
            .map(|record| record.record_id.clone())
            .collect::<Vec<_>>()
    };
    let (evaluable_count, indeterminate_count, dqi_count, coverage, indeterminate_rate, dqi_rate) =
        health
            .map(|values| {
                (
                    Some(values.0),
                    Some(values.1),
                    Some(values.2),
                    Some(values.3),
                    Some(values.4),
                    Some(values.5),
                )
            })
            .unwrap_or((None, None, None, None, None, None));
    Ok(CohortRowV1 {
        endpoint_id: descriptor.endpoint_id.into(),
        stratum_id,
        endpoint_kind: descriptor.endpoint_kind,
        cohort_role: descriptor.cohort_role,
        declared_record_ids: declared.clone(),
        eligible_record_ids: eligible.clone(),
        excluded_record_ids: excluded.clone(),
        not_applicable_record_ids: not_applicable,
        development_record_ids: roles(CohortRoleV1::Development),
        validation_record_ids: roles(CohortRoleV1::Validation),
        holdout_record_ids: roles(CohortRoleV1::Holdout),
        declared_count: declared.len() as u64,
        eligible_count: eligible.len() as u64,
        excluded_count: excluded.len() as u64,
        not_applicable_count: rows
            .iter()
            .filter(|row| row.decision == RecordDecisionV1::NotApplicable)
            .count() as u64,
        exclusion_rate: rate(
            excluded.len() as u64,
            declared.len() as u64,
            "denominator_zero",
        )?,
        evaluable_count,
        indeterminate_count,
        data_quality_insufficient_count: dqi_count,
        coverage,
        indeterminate_rate,
        data_quality_insufficient_rate: dqi_rate,
        outcome,
    })
}

fn propagate_required_strata(results: &mut [MechanismResultV1], cohorts: &mut [CohortRowV1]) {
    let indeterminate = results
        .iter()
        .filter(|result| {
            result.stratum_id != "overall" && result.outcome == ValidationOutcomeV1::Indeterminate
        })
        .map(|result| result.stratum_id.clone())
        .collect::<Vec<_>>();
    if let Some(overall) = results
        .iter_mut()
        .find(|result| result.stratum_id == "overall")
    {
        for stratum_id in indeterminate {
            overall
                .outcome_reasons
                .push(OutcomeReasonV1::RequiredStratumIndeterminate { stratum_id });
        }
        overall.outcome_reasons.sort_by(outcome_reason_key);
        overall.outcome_reasons.dedup();
        overall.outcome = outcome_for(&overall.outcome_reasons);
    }
    if let (Some(cohort), Some(result)) = (
        cohorts.iter_mut().find(|row| row.stratum_id == "overall"),
        results.iter().find(|result| result.stratum_id == "overall"),
    ) {
        cohort.outcome = result.outcome;
    }
}
fn propagate_required_strata_health(results: &mut [HealthResultV1], cohorts: &mut [CohortRowV1]) {
    let indeterminate = results
        .iter()
        .filter(|result| {
            result.stratum_id != "overall" && result.outcome == ValidationOutcomeV1::Indeterminate
        })
        .map(|result| result.stratum_id.clone())
        .collect::<Vec<_>>();
    if let Some(overall) = results
        .iter_mut()
        .find(|result| result.stratum_id == "overall")
    {
        for stratum_id in indeterminate {
            overall
                .outcome_reasons
                .push(OutcomeReasonV1::RequiredStratumIndeterminate { stratum_id });
        }
        overall.outcome_reasons.sort_by(outcome_reason_key);
        overall.outcome = outcome_for(&overall.outcome_reasons);
    }
    if let (Some(cohort), Some(result)) = (
        cohorts.iter_mut().find(|row| row.stratum_id == "overall"),
        results.iter().find(|result| result.stratum_id == "overall"),
    ) {
        cohort.outcome = result.outcome;
    }
}

fn accounting_row(row: &PartitionRowV1) -> RecordAccountingRowV1 {
    RecordAccountingRowV1 {
        endpoint_id: row.endpoint_id.clone(),
        stratum_id: row.stratum_id.clone(),
        record_id: row.record_id.clone(),
        decision: row.decision,
        primary_reason: row.primary_reason,
        secondary_reasons: row.secondary_reasons.clone(),
        assessed_source_key: row.assessed_source_key.clone(),
        reference_endpoint_id: row.reference_endpoint_id.clone(),
    }
}
fn leakage_row(row: &PartitionRowV1) -> crate::results::LeakageRowV1 {
    crate::results::LeakageRowV1 {
        endpoint_id: row.endpoint_id.clone(),
        stratum_id: row.stratum_id.clone(),
        record_id: row.record_id.clone(),
        separation_status: row.separation_status,
        not_evaluated_reason: row.not_evaluated_reason,
        compared_development_record_ids: row.compared_development_record_ids.clone(),
        shared_artifact_ids: row.shared_artifact_ids.clone(),
        shared_source_sha256s: row.shared_source_sha256s.clone(),
        shared_experiment_ids: row.shared_experiment_ids.clone(),
        shared_family_ids: row.shared_family_ids.clone(),
        unknown_reasons: row.unknown_reasons.clone(),
        decision: row.decision,
    }
}
fn rows_for<'a>(rows: &'a [PartitionRowV1], view: &str) -> Vec<&'a PartitionRowV1> {
    rows.iter().filter(|row| row.stratum_id == view).collect()
}
fn endpoint_views(strata: &[crate::validation_config::RequiredStratumV1]) -> Vec<String> {
    std::iter::once("overall".into())
        .chain(strata.iter().map(|stratum| stratum.stratum_id.clone()))
        .collect()
}
fn ids<'a>(rows: impl Iterator<Item = &'a PartitionRowV1>) -> Vec<String> {
    rows.map(|row| row.record_id.clone()).collect()
}
fn record<'a>(
    records: &'a [crate::results::ValidationRecordV1],
    id: &str,
) -> Option<&'a crate::results::ValidationRecordV1> {
    records.iter().find(|record| record.record_id == id)
}
fn add_families(record: &crate::results::ValidationRecordV1, families: &mut BTreeSet<String>) {
    if let ArtifactAcquisitionFamilies::Known(values) = &record.declared_scope.acquisition_families
    {
        families.extend(values.iter().map(|value| value.0.clone()));
    }
}
fn dedup(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}
fn rate(
    numerator: u64,
    denominator: u64,
    reason: &str,
) -> Result<MetricValueV1, MhiValidationError> {
    Ok(
        match wilson_95_checked(numerator, denominator)
            .map_err(|message| MhiValidationError::Dataset(message.into()))?
        {
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
        },
    )
}
fn balanced_accuracy(
    sensitivity: &MetricValueV1,
    specificity: &MetricValueV1,
) -> BalancedAccuracyV1 {
    match (sensitivity, specificity) {
        (
            MetricValueV1::Available {
                point_estimate: sensitivity,
                ..
            },
            MetricValueV1::Available {
                point_estimate: specificity,
                ..
            },
        ) => BalancedAccuracyV1::Available {
            sensitivity_metric_id: "sensitivity".into(),
            specificity_metric_id: "specificity".into(),
            point_estimate: (sensitivity + specificity) / 2.0,
        },
        _ => BalancedAccuracyV1::Unavailable {
            sensitivity_metric_id: "sensitivity".into(),
            specificity_metric_id: "specificity".into(),
            reason: "sensitivity_or_specificity_unavailable".into(),
        },
    }
}
fn evaluate_rules(
    rules: &[AcceptanceRuleV1],
    counts: &[(CountMetricV1, u64)],
    rates: &[(RateMetricV1, MetricValueV1)],
    balanced: Option<BalancedAccuracyV1>,
) -> (bool, bool, Vec<RuleEvaluationV1>) {
    let mut false_seen = false;
    let mut unavailable_seen = false;
    let values = rules
        .iter()
        .map(|rule| {
            let (actual, state) = match rule {
                AcceptanceRuleV1::Count {
                    metric,
                    comparator,
                    threshold_u64,
                    ..
                } => {
                    let value = counts
                        .iter()
                        .find(|(name, _)| name == metric)
                        .map(|(_, value)| *value);
                    let state = value.and_then(|value| {
                        compare(value as f64, *threshold_u64 as f64, *comparator)
                    });
                    (
                        RuleActualV1::Count {
                            value: value.unwrap_or_default(),
                        },
                        state,
                    )
                }
                AcceptanceRuleV1::Rate {
                    metric,
                    target: _,
                    comparator,
                    threshold,
                    ..
                } if *metric == RateMetricV1::BalancedAccuracy => {
                    let actual = balanced.clone().unwrap_or(BalancedAccuracyV1::Unavailable {
                        sensitivity_metric_id: "sensitivity".into(),
                        specificity_metric_id: "specificity".into(),
                        reason: "sensitivity_or_specificity_unavailable".into(),
                    });
                    let state = match &actual {
                        BalancedAccuracyV1::Available { point_estimate, .. } => {
                            compare(*point_estimate, *threshold, *comparator)
                        }
                        _ => None,
                    };
                    (RuleActualV1::BalancedAccuracy { value: actual }, state)
                }
                AcceptanceRuleV1::Rate {
                    metric,
                    target,
                    comparator,
                    threshold,
                    ..
                } => {
                    let value = rates
                        .iter()
                        .find(|(name, _)| name == metric)
                        .map(|(_, value)| value.clone())
                        .unwrap_or(MetricValueV1::Unavailable {
                            numerator: 0,
                            denominator: 0,
                            reason: "denominator_zero".into(),
                        });
                    let state = rate_target(&value, *target)
                        .and_then(|actual| compare(actual, *threshold, *comparator));
                    (RuleActualV1::BinomialRate { value }, state)
                }
            };
            let result = match state {
                Some(true) => RuleEvaluationResultV1::True,
                Some(false) => {
                    false_seen = true;
                    RuleEvaluationResultV1::False
                }
                None => {
                    unavailable_seen = true;
                    RuleEvaluationResultV1::Unavailable
                }
            };
            RuleEvaluationV1 {
                rule: rule.clone(),
                actual,
                result,
            }
        })
        .collect();
    (false_seen, unavailable_seen, values)
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
fn compare(
    actual: f64,
    threshold: f64,
    comparator: crate::validation_config::ComparatorV1,
) -> Option<bool> {
    actual.is_finite().then_some(match comparator {
        crate::validation_config::ComparatorV1::GreaterThanOrEqual => actual >= threshold,
        crate::validation_config::ComparatorV1::LessThanOrEqual => actual <= threshold,
    })
}
fn rule_id(rule: &AcceptanceRuleV1) -> &str {
    match rule {
        AcceptanceRuleV1::Count { rule_id, .. } | AcceptanceRuleV1::Rate { rule_id, .. } => rule_id,
    }
}
fn phase_b_level(
    report: &crate::domain::StrictArtifactRead<crate::results::MechanismAnalysisReport>,
    hypothesis_id: &str,
) -> Result<HypothesisEvidenceLevel, MhiValidationError> {
    let rows = report
        .artifact
        .hypothesis_assessments
        .iter()
        .filter(|row| row.definition.hypothesis_id == hypothesis_id)
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Ok(HypothesisEvidenceLevel::NotAssessed);
    }
    if rows.len() != 1 || rows[0].current.hypothesis_id != hypothesis_id {
        return Err(MhiValidationError::Dataset(
            "Phase-B hypothesis ID mismatch or duplicate".into(),
        ));
    }
    Ok(rows[0].current.evidence_level.clone())
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
                        .and_then(serde_json::Value::as_str)
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
fn dataset_source(
    inputs: &ValidationInputs,
) -> Result<DatasetSourceReferenceV1, MhiValidationError> {
    match &inputs.dataset.artifact.lineage {
        ArtifactLineageState::Known { identity, .. } => Ok(DatasetSourceReferenceV1::Known {
            dataset_id: inputs.dataset.artifact.dataset_id.clone(),
            schema: 1,
            artifact_id: identity.artifact_id.clone(),
            semantic_sha256: identity.semantic_sha256.clone(),
            source_file_sha256: inputs.dataset.source_file_sha256.clone(),
        }),
        ArtifactLineageState::LegacyUnknown {
            source_schema_version: _,
            reason,
        } => Ok(DatasetSourceReferenceV1::LegacyUnknown {
            dataset_id: inputs.dataset.artifact.dataset_id.clone(),
            schema: 1,
            legacy_fingerprint: inputs.dataset.source_file_sha256.clone(),
            source_file_sha256: inputs.dataset.source_file_sha256.clone(),
            reason: match reason {
                crate::domain::UnknownLineageReason::FieldAbsentInLegacyArtifact => {
                    crate::results::LegacyLineageReasonV1::FieldAbsentInLegacyArtifact
                }
                crate::domain::UnknownLineageReason::ExternalArtifactWithoutLineage => {
                    crate::results::LegacyLineageReasonV1::ExternalArtifactWithoutLineage
                }
                crate::domain::UnknownLineageReason::MigrationInformationUnavailable => {
                    crate::results::LegacyLineageReasonV1::MigrationInformationUnavailable
                }
            },
        }),
    }
}
fn approval_authority(
    protocol: &MhiValidationProtocolV1,
    inputs: &ValidationInputs,
) -> Result<Option<ApprovalAuthorityV1>, MhiValidationError> {
    let physical = protocol
        .release_scope
        .iter()
        .any(|claim| claim.requested_level == RequestedValidationLevelV1::Physical);
    if !physical {
        return Ok(None);
    }
    let approval = inputs.owner_approval.as_ref().ok_or_else(|| {
        MhiValidationError::Approval(
            "physical evaluation requires a verified owner approval".into(),
        )
    })?;
    let source = inputs
        .dataset
        .artifact
        .owner_approval_source
        .as_ref()
        .ok_or_else(|| {
            MhiValidationError::Approval(
                "physical evaluation requires owner_approval_source".into(),
            )
        })?;
    Ok(Some(ApprovalAuthorityV1 {
        approval_source_file_sha256: source.source_file_sha256.clone(),
        approval_record_id: approval.approval_record_id().into(),
        trust_store_id: approval.evidence().trust_store_id.clone(),
        approval_purpose: approval.evidence().approval_purpose.clone(),
        trust_store_sha256: approval.trust_store_sha256().into(),
        trust_root_id: approval.evidence().trust_root_id.clone(),
        project_owner_authority_id: approval.evidence().project_owner_authority_id.clone(),
        registry_authority_id: approval.evidence().registry_authority_id.clone(),
        owner_authority_document: ImmutableDocumentReferenceV1 {
            immutable_reference_uri: approval
                .evidence()
                .owner_authority_document
                .immutable_reference_uri
                .clone(),
            document_sha256: approval
                .evidence()
                .owner_authority_document
                .document_sha256
                .clone(),
        },
        registry_record: ImmutableDocumentReferenceV1 {
            immutable_reference_uri: approval
                .evidence()
                .registry_record
                .immutable_reference_uri
                .clone(),
            document_sha256: approval.evidence().registry_record.document_sha256.clone(),
        },
        owner_signature_verified: true,
        registry_signature_verified: true,
        binding_status: "verified".into(),
        limitations: approval.evidence().limitations.clone(),
    }))
}
fn compatibility(inputs: &ValidationInputs) -> Vec<CompatibilityRowV1> {
    let mut rows = vec![
        CompatibilityRowV1 {
            record_id: None,
            source_role: CompatibilitySourceRoleV1::Protocol,
            relative_path: "@protocol".into(),
            expected_kind: None,
            actual_kind: None,
            expected_schema: 1,
            actual_schema: 1,
            expected_file_sha256: inputs.dataset.artifact.protocol_sha256.clone(),
            actual_file_sha256: inputs.dataset.artifact.protocol_sha256.clone(),
            expected_artifact_id: None,
            actual_artifact_id: None,
            expected_semantic_sha256: None,
            actual_semantic_sha256: None,
            result: CompatibilityResultV1::Compatible,
        },
        CompatibilityRowV1 {
            record_id: None,
            source_role: CompatibilitySourceRoleV1::Dataset,
            relative_path: "@dataset".into(),
            expected_kind: Some(crate::domain::ArtifactKind::MhiValidationDataset),
            actual_kind: Some(crate::domain::ArtifactKind::MhiValidationDataset),
            expected_schema: 1,
            actual_schema: 1,
            expected_file_sha256: inputs.dataset.source_file_sha256.clone(),
            actual_file_sha256: inputs.dataset.source_file_sha256.clone(),
            expected_artifact_id: None,
            actual_artifact_id: None,
            expected_semantic_sha256: None,
            actual_semantic_sha256: None,
            result: CompatibilityResultV1::Compatible,
        },
        CompatibilityRowV1 {
            record_id: None,
            source_role: CompatibilitySourceRoleV1::LineageCatalog,
            relative_path: inputs
                .dataset
                .artifact
                .lineage_catalog_source
                .relative_path
                .clone(),
            expected_kind: None,
            actual_kind: None,
            expected_schema: 1,
            actual_schema: 1,
            expected_file_sha256: inputs
                .dataset
                .artifact
                .lineage_catalog_source
                .source_file_sha256
                .clone(),
            actual_file_sha256: inputs.lineage_catalog.source_file_sha256.clone(),
            expected_artifact_id: None,
            actual_artifact_id: None,
            expected_semantic_sha256: None,
            actual_semantic_sha256: None,
            result: CompatibilityResultV1::Compatible,
        },
    ];
    for (record_id, source) in &inputs.mechanism_sources {
        if let Some(expectation) = record(&inputs.dataset.artifact.records, record_id)
            .and_then(|record| record.mechanism_source.as_ref())
        {
            rows.push(compatibility_source(
                record_id,
                expectation,
                &source.artifact.lineage,
                &source.source_file_sha256,
                CompatibilitySourceRoleV1::MechanismSource,
            ));
        }
    }
    for (record_id, source) in &inputs.health_sources {
        if let Some(expectation) = record(&inputs.dataset.artifact.records, record_id)
            .and_then(|record| record.health_source.as_ref())
        {
            rows.push(compatibility_source(
                record_id,
                expectation,
                &source.artifact.lineage,
                &source.source_file_sha256,
                CompatibilitySourceRoleV1::HealthSource,
            ));
        }
    }
    rows.sort_by(|a, b| {
        (
            a.source_role,
            a.record_id.clone().unwrap_or_default(),
            a.relative_path.clone(),
        )
            .cmp(&(
                b.source_role,
                b.record_id.clone().unwrap_or_default(),
                b.relative_path.clone(),
            ))
    });
    rows
}
fn consumed_sources(inputs: &ValidationInputs) -> Vec<SourceReferenceV1> {
    let mut values = vec![SourceReferenceV1::LineageCatalog {
        schema: 1,
        source_file_sha256: inputs.lineage_catalog.source_file_sha256.clone(),
    }];
    values.extend(
        inputs
            .dataset
            .artifact
            .reference_sources
            .iter()
            .map(|source| SourceReferenceV1::ReferenceAuthority {
                reference_source_id: source.reference_source_id.clone(),
                source_file_sha256: source.source_file_sha256.clone(),
                origin: source.evidence_origin,
            }),
    );
    if let (Some(approval), Some(source)) = (
        &inputs.owner_approval,
        &inputs.dataset.artifact.owner_approval_source,
    ) {
        values.push(SourceReferenceV1::ApprovalTrustStore {
            trust_store_id: approval.evidence().trust_store_id.clone(),
            source_file_sha256: approval.trust_store_sha256().into(),
        });
        values.push(SourceReferenceV1::OwnerApproval {
            approval_record_id: approval.approval_record_id().into(),
            source_file_sha256: source.source_file_sha256.clone(),
            registry_record_sha256: approval.evidence().registry_record.document_sha256.clone(),
        });
    }
    for (record_id, _source) in &inputs.mechanism_sources {
        if let Some(expectation) = record(&inputs.dataset.artifact.records, record_id)
            .and_then(|record| record.mechanism_source.as_ref())
        {
            values.push(source_reference(expectation));
        }
    }
    for (record_id, _source) in &inputs.health_sources {
        if let Some(expectation) = record(&inputs.dataset.artifact.records, record_id)
            .and_then(|record| record.health_source.as_ref())
        {
            values.push(source_reference(expectation));
        }
    }
    values.sort_by_key(source_key);
    values.dedup_by(|left, right| source_key(left) == source_key(right));
    values
}
fn source_key(value: &SourceReferenceV1) -> (u8, String, String, String) {
    match value {
        SourceReferenceV1::KnownArtifact {
            kind,
            artifact_id,
            source_file_sha256,
            ..
        } => (
            0,
            kind.as_str().into(),
            artifact_id.0.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::LegacyArtifact {
            kind,
            legacy_fingerprint,
            source_file_sha256,
            ..
        } => (
            1,
            kind.as_str().into(),
            legacy_fingerprint.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::LineageCatalog {
            source_file_sha256, ..
        } => (
            2,
            "lineage_catalog".into(),
            String::new(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::ReferenceAuthority {
            reference_source_id,
            source_file_sha256,
            ..
        } => (
            3,
            "reference_authority".into(),
            reference_source_id.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::ApprovalTrustStore {
            trust_store_id,
            source_file_sha256,
        } => (
            4,
            "approval_trust_store".into(),
            trust_store_id.clone(),
            source_file_sha256.clone(),
        ),
        SourceReferenceV1::OwnerApproval {
            approval_record_id,
            source_file_sha256,
            ..
        } => (
            5,
            "owner_approval".into(),
            approval_record_id.clone(),
            source_file_sha256.clone(),
        ),
    }
}
fn compatibility_source(
    record_id: &str,
    expectation: &ArtifactSourceExpectationV1,
    lineage: &ArtifactLineageState,
    actual_file_sha256: &str,
    source_role: CompatibilitySourceRoleV1,
) -> CompatibilityRowV1 {
    let (actual_artifact_id, actual_semantic_sha256, result) = match lineage {
        ArtifactLineageState::Known { identity, .. } => (
            Some(identity.artifact_id.clone()),
            Some(identity.semantic_sha256.clone()),
            CompatibilityResultV1::Compatible,
        ),
        ArtifactLineageState::LegacyUnknown {
            source_schema_version,
            ..
        } => (
            None,
            None,
            if *source_schema_version == Some(4) {
                CompatibilityResultV1::CurrentLegacyUnknownExcluded
            } else {
                CompatibilityResultV1::ReadableLegacyExcluded
            },
        ),
    };
    let (expected_artifact_id, expected_semantic_sha256) = match &expectation.expected_lineage {
        crate::results::ExpectedLineageV1::Known {
            artifact_id,
            semantic_sha256,
        } => (Some(artifact_id.clone()), Some(semantic_sha256.clone())),
        crate::results::ExpectedLineageV1::LegacyUnknown { .. } => (None, None),
    };
    CompatibilityRowV1 {
        record_id: Some(record_id.into()),
        source_role,
        relative_path: expectation.relative_path.clone(),
        expected_kind: Some(expectation.expected_artifact_kind),
        actual_kind: Some(expectation.expected_artifact_kind),
        expected_schema: expectation.expected_schema_version,
        actual_schema: expectation.expected_schema_version,
        expected_file_sha256: expectation.source_file_sha256.clone(),
        actual_file_sha256: actual_file_sha256.into(),
        expected_artifact_id,
        actual_artifact_id,
        expected_semantic_sha256,
        actual_semantic_sha256,
        result,
    }
}
fn source_reference(expectation: &ArtifactSourceExpectationV1) -> SourceReferenceV1 {
    match &expectation.expected_lineage {
        crate::results::ExpectedLineageV1::Known {
            artifact_id,
            semantic_sha256,
        } => SourceReferenceV1::KnownArtifact {
            kind: expectation.expected_artifact_kind,
            schema: expectation.expected_schema_version,
            artifact_id: artifact_id.clone(),
            semantic_sha256: semantic_sha256.clone(),
            source_file_sha256: expectation.source_file_sha256.clone(),
        },
        crate::results::ExpectedLineageV1::LegacyUnknown {
            schema_version,
            legacy_source_fingerprint,
            reason,
        } => SourceReferenceV1::LegacyArtifact {
            kind: expectation.expected_artifact_kind,
            schema: *schema_version,
            legacy_fingerprint: legacy_source_fingerprint.clone(),
            source_file_sha256: expectation.source_file_sha256.clone(),
            reason: reason.clone(),
        },
    }
}
fn warnings(inputs: &ValidationInputs) -> Vec<ValidationWarningV1> {
    let mut values = inputs
        .dataset
        .artifact
        .records
        .iter()
        .filter_map(|record| {
            [
                record.mechanism_source.as_ref(),
                record.health_source.as_ref(),
            ]
            .into_iter()
            .flatten()
            .find(|source| {
                !matches!(
                    source.expected_lineage,
                    crate::results::ExpectedLineageV1::Known { .. }
                )
            })
            .map(|_| ValidationWarningV1 {
                code: ValidationWarningCodeV1::LegacySourceExcluded,
                related_id: record.record_id.clone(),
                detail: "source is not a scoreable Known Phase-B/C authority".into(),
            })
        })
        .collect::<Vec<_>>();
    values.sort_by(|a, b| {
        (a.code.clone(), a.related_id.clone(), a.detail.clone()).cmp(&(
            b.code.clone(),
            b.related_id.clone(),
            b.detail.clone(),
        ))
    });
    values
}
fn row_key(a: &RecordAccountingRowV1, b: &RecordAccountingRowV1) -> std::cmp::Ordering {
    (
        a.endpoint_id.clone(),
        view_key(&a.stratum_id),
        a.record_id.clone(),
    )
        .cmp(&(
            b.endpoint_id.clone(),
            view_key(&b.stratum_id),
            b.record_id.clone(),
        ))
}
fn leakage_key(
    a: &crate::results::LeakageRowV1,
    b: &crate::results::LeakageRowV1,
) -> std::cmp::Ordering {
    (
        a.endpoint_id.clone(),
        view_key(&a.stratum_id),
        a.record_id.clone(),
    )
        .cmp(&(
            b.endpoint_id.clone(),
            view_key(&b.stratum_id),
            b.record_id.clone(),
        ))
}
fn cohort_key(a: &CohortRowV1, b: &CohortRowV1) -> std::cmp::Ordering {
    (a.endpoint_id.clone(), view_key(&a.stratum_id))
        .cmp(&(b.endpoint_id.clone(), view_key(&b.stratum_id)))
}
fn mechanism_key(a: &MechanismResultV1, b: &MechanismResultV1) -> std::cmp::Ordering {
    (a.endpoint_id.clone(), view_key(&a.stratum_id))
        .cmp(&(b.endpoint_id.clone(), view_key(&b.stratum_id)))
}
fn health_key(a: &HealthResultV1, b: &HealthResultV1) -> std::cmp::Ordering {
    (a.endpoint_id.clone(), view_key(&a.stratum_id))
        .cmp(&(b.endpoint_id.clone(), view_key(&b.stratum_id)))
}
fn view_key(value: &str) -> (u8, String) {
    (u8::from(value != "overall"), value.into())
}
fn outcome_reason_key(a: &OutcomeReasonV1, b: &OutcomeReasonV1) -> std::cmp::Ordering {
    outcome_reason_order(a)
        .cmp(&outcome_reason_order(b))
        .then_with(|| format!("{a:?}").cmp(&format!("{b:?}")))
}
fn outcome_reason_order(value: &OutcomeReasonV1) -> u8 {
    match value {
        OutcomeReasonV1::HoldoutKnownOverlap { .. } => 2,
        OutcomeReasonV1::HoldoutUnknownSeparation { .. } => 3,
        OutcomeReasonV1::DeclaredCriticalFalsification { .. } => 4,
        OutcomeReasonV1::EmptyView
        | OutcomeReasonV1::EligibleRecordMinimumNotMet { .. }
        | OutcomeReasonV1::IndependentFamilyMinimumNotMet { .. }
        | OutcomeReasonV1::RequiredStratumIndeterminate { .. } => 5,
        OutcomeReasonV1::ReferenceUncertaintyUnavailable { .. } => 6,
        OutcomeReasonV1::RequiredRuleUnavailable { .. } => 7,
        OutcomeReasonV1::RequiredRuleFalse { .. } => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_e_exclusions_and_acceptance_use_complete_ordered_precedence() {
        assert_eq!(
            outcome_for(&[
                OutcomeReasonV1::RequiredRuleUnavailable {
                    rule_id: "unavailable".into(),
                },
                OutcomeReasonV1::RequiredRuleFalse {
                    rule_id: "false".into(),
                },
            ]),
            ValidationOutcomeV1::Indeterminate
        );
        assert_eq!(
            outcome_for(&[
                OutcomeReasonV1::HoldoutKnownOverlap {
                    record_id: "record".into(),
                },
                OutcomeReasonV1::RequiredRuleUnavailable {
                    rule_id: "unavailable".into(),
                },
            ]),
            ValidationOutcomeV1::DoesNotMeetProtocol
        );
    }
}
