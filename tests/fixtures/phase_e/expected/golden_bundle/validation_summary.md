# MHI Validation Summary

## Identity

| key | value |
| --- | --- |
| report_id | sha256:4960a05cd3e8b228dde475408101a9a161ebe2990d6746011aada14d371d0f3e |
| protocol_id | phase_e_software_protocol |
| protocol_sha256 | c141c0f28246bb16edcca79e05caee065516c7b06770d08a536b31a43857cf17 |
| dataset_id | phase_e_software_dataset |
| dataset_source_file_sha256 | 59955a38c93193740ffeb7abc9b4b2a2e5df37ea715c041d1978c7819a1ff657 |
| approval_record_id | NA |
| approval_trust_store_sha256 | NA |
| software_version | 0.1.0 |
| git_commit | NA |

## Cohort Coverage

| endpoint_id | stratum_id | endpoint_kind | cohort_role | declared_count | eligible_count | excluded_count | not_applicable_count | exclusion_rate | exclusion_lower | exclusion_upper | evaluable_count | indeterminate_count | data_quality_insufficient_count | coverage | coverage_lower | coverage_upper | indeterminate_rate | indeterminate_lower | indeterminate_upper | data_quality_insufficient_rate | data_quality_insufficient_lower | data_quality_insufficient_upper | outcome |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| health_endpoint | overall | health_dimension | validation | 2 | 2 | 0 | 0 | 0.0 | 0.0 | 0.6576197724933469 | 2 | 0 | 0 | 1.0 | 0.34238022750665303 | 1.0 | 0.0 | 0.0 | 0.6576197724933469 | 0.0 | 0.0 | 0.6576197724933469 | meets_protocol |
| mechanism_endpoint | overall | mechanism | validation | 2 | 2 | 0 | 0 | 0.0 | 0.0 | 0.6576197724933469 | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | meets_protocol |

## Leakage

| endpoint_id | stratum_id | record_id | separation_status | not_evaluated_reason | compared_development_record_ids | shared_artifact_ids | shared_source_sha256s | shared_experiment_ids | shared_family_ids | unknown_reasons | decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| health_endpoint | overall | record_1 | known_separated | NA | [] | [] | [] | [] | [] | [] | eligible |
| health_endpoint | overall | record_2 | known_separated | NA | [] | [] | [] | [] | [] | [] | eligible |
| mechanism_endpoint | overall | record_1 | known_separated | NA | [] | [] | [] | [] | [] | [] | eligible |
| mechanism_endpoint | overall | record_2 | known_separated | NA | [] | [] | [] | [] | [] | [] | eligible |

## Mechanism Endpoints

| endpoint_id | stratum_id | eligible_count | independent_family_count | support_count | critical_contradiction_count | declared_critical_falsification_count | not_assessed_or_other_count | support_fraction | support_lower | support_upper | contradiction_fraction | contradiction_lower | contradiction_upper | not_assessed_fraction | not_assessed_lower | not_assessed_upper | outcome |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mechanism_endpoint | overall | 2 | 2 | 2 | 0 | 0 | 0 | 1.0 | 0.34238022750665303 | 1.0 | 0.0 | 0.0 | 0.6576197724933469 | 0.0 | 0.0 | 0.6576197724933469 | meets_protocol |

## Health Endpoints

| endpoint_id | stratum_id | eligible_count | independent_family_count | tp | tn | fp | fn | indeterminate | data_quality_insufficient | evaluable | coverage | coverage_lower | coverage_upper | indeterminate_rate | indeterminate_lower | indeterminate_upper | data_quality_insufficient_rate | data_quality_insufficient_lower | data_quality_insufficient_upper | sensitivity | sensitivity_lower | sensitivity_upper | specificity | specificity_lower | specificity_upper | false_positive_rate | false_positive_lower | false_positive_upper | false_negative_rate | false_negative_lower | false_negative_upper | balanced_accuracy | outcome |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| health_endpoint | overall | 2 | 2 | 1 | 1 | 0 | 0 | 0 | 0 | 2 | 1.0 | 0.34238022750665303 | 1.0 | 0.0 | 0.0 | 0.6576197724933469 | 0.0 | 0.0 | 0.6576197724933469 | 1.0 | 0.20654931437723745 | 1.0 | 1.0 | 0.20654931437723745 | 1.0 | 0.0 | 0.0 | 0.7934506856227626 | 0.0 | 0.0 | 0.7934506856227626 | 1.0 | meets_protocol |

## Exclusions

| endpoint_id | stratum_id | record_id | primary_reason | secondary_reasons | assessed_source_key | reference_endpoint_id |
| --- | --- | --- | --- | --- | --- | --- |

## Release Claims

| claim_id | requested_level | statement | domain | supporting_endpoint_ids | approval_record_id | outcome |
| --- | --- | --- | --- | --- | --- | --- |
| software_claim | software | Software-only fixture | {"analyte":{"type":"any_declared"},"campaign":{"type":"any_declared"},"matrix":{"type":"any_declared"},"sensor":{"type":"any_declared"},"sensor_design":{"type":"any_declared"},"temperature":{"type":"any_declared"}} | ["health_endpoint","mechanism_endpoint"] | NA | software_validated_only |

## Overall Status

outcome: meets_protocol

## Limitations

- NONE
