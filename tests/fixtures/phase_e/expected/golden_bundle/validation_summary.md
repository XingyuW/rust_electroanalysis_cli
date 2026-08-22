# MHI Validation Summary

## Identity

| key | value |
| --- | --- |
| report_id | sha256:4395c2ffba66df1efeb2e7eb7a76a28cd5349188a698613d50253516b79b2bae |
| protocol_id | phase_e_software_protocol |
| protocol_sha256 | a098c83e08f488d49f16be4c4fc27b09d87ca3752a7af7b50069ba6e9e09b47e |
| dataset_id | phase_e_software_dataset |
| dataset_source_file_sha256 | 3c4c3003f1da37d1cc0e71de1160619b44ea8df8d8e0845c7007a54590a36a7f |
| approval_record_id | NA |
| approval_trust_store_sha256 | NA |
| software_version | 0.1.0 |
| git_commit | NA |

## Cohort Coverage

| endpoint_id | stratum_id | endpoint_kind | cohort_role | declared_count | eligible_count | excluded_count | not_applicable_count | exclusion_rate | exclusion_lower | exclusion_upper | evaluable_count | indeterminate_count | data_quality_insufficient_count | coverage | coverage_lower | coverage_upper | indeterminate_rate | indeterminate_lower | indeterminate_upper | data_quality_insufficient_rate | data_quality_insufficient_lower | data_quality_insufficient_upper | outcome |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| health_endpoint | overall | health_aggregate | validation | 1 | 0 | 1 | 0 | 1.0 | 0.20654931437723745 | 1.0 | 0 | 0 | 0 | NA | NA | NA | NA | NA | NA | NA | NA | NA | indeterminate |
| mechanism_endpoint | overall | mechanism | validation | 1 | 0 | 1 | 0 | 1.0 | 0.20654931437723745 | 1.0 | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | indeterminate |

## Leakage

| endpoint_id | stratum_id | record_id | separation_status | not_evaluated_reason | compared_development_record_ids | shared_artifact_ids | shared_source_sha256s | shared_experiment_ids | shared_family_ids | unknown_reasons | decision |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| health_endpoint | overall | record_1 | NA | missing_endpoint_artifact_path | [] | [] | [] | [] | [] | [] | excluded |
| mechanism_endpoint | overall | record_1 | NA | missing_endpoint_artifact_path | [] | [] | [] | [] | [] | [] | excluded |

## Mechanism Endpoints

| endpoint_id | stratum_id | eligible_count | independent_family_count | support_count | critical_contradiction_count | declared_critical_falsification_count | not_assessed_or_other_count | support_fraction | support_lower | support_upper | contradiction_fraction | contradiction_lower | contradiction_upper | not_assessed_fraction | not_assessed_lower | not_assessed_upper | outcome |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| mechanism_endpoint | overall | 0 | 0 | 0 | 0 | 0 | 0 | NA | NA | NA | NA | NA | NA | NA | NA | NA | indeterminate |

## Health Endpoints

| endpoint_id | stratum_id | eligible_count | independent_family_count | tp | tn | fp | fn | indeterminate | data_quality_insufficient | evaluable | coverage | coverage_lower | coverage_upper | indeterminate_rate | indeterminate_lower | indeterminate_upper | data_quality_insufficient_rate | data_quality_insufficient_lower | data_quality_insufficient_upper | sensitivity | sensitivity_lower | sensitivity_upper | specificity | specificity_lower | specificity_upper | false_positive_rate | false_positive_lower | false_positive_upper | false_negative_rate | false_negative_lower | false_negative_upper | balanced_accuracy | outcome |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| health_endpoint | overall | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | NA | indeterminate |

## Exclusions

| endpoint_id | stratum_id | record_id | primary_reason | secondary_reasons | assessed_source_key | reference_endpoint_id |
| --- | --- | --- | --- | --- | --- | --- |
| health_endpoint | overall | record_1 | missing_endpoint_artifact_path | ["missing_reference_endpoint"] | NA | NA |
| mechanism_endpoint | overall | record_1 | missing_endpoint_artifact_path | ["missing_reference_endpoint"] | NA | NA |

## Release Claims

| claim_id | requested_level | statement | domain | supporting_endpoint_ids | approval_record_id | outcome |
| --- | --- | --- | --- | --- | --- | --- |
| software_claim | software | Software-only fixture | {"analyte":{"type":"any_declared"},"campaign":{"type":"any_declared"},"matrix":{"type":"any_declared"},"sensor":{"type":"any_declared"},"sensor_design":{"type":"any_declared"},"temperature":{"type":"any_declared"}} | ["health_endpoint","mechanism_endpoint"] | NA | indeterminate |

## Overall Status

outcome: indeterminate

## Limitations

- NONE
