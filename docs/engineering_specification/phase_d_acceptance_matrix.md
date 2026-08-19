# Phase D implementation acceptance matrix

This working matrix is the implementation-side index for the authoritative Phase-D plan. Each row maps directly to plan section 18.12 and its E## output ledger; all fixtures are the sealed entries under `tests/fixtures/phase_d/`. “Implemented”, “test”, and “passes” reflect `tests/phase_d_reporting_public_output.rs`.

| # | Exact mandatory test | R / AC / E | Fixture or bundle authority | Required production behavior | Implemented | Test | Passes | Blocker |
|---:|---|---|---|---|---|---|---|---|
| 1 | `phase_d_cli_requires_mechanism_and_health` | R01 / AC01 / E01 | none | Clap rejects missing required flags before runner | yes | yes | yes | none |
| 2 | `phase_d_clap_rejects_unknown_format_before_runner` | R01 / AC02 / E02 | none | Clap rejects unknown format before runner | yes | yes | yes | none |
| 3 | `phase_d_cli_rejects_unpaired_calibration_inputs` | R02 / AC03 / E03 | N-F07/N-F05 | Calibration pair is all-or-nothing | yes | yes | yes | none |
| 4 | `phase_d_cli_rejects_unknown_selection` | R02 / AC04 / E04 | none | Unknown selector fails closed | yes | yes | yes | none |
| 5 | `phase_d_cli_rejects_duplicate_selection` | R02 / AC05 / E04 | none | Duplicate selector fails closed | yes | yes | yes | none |
| 6 | `phase_d_cli_rejects_existing_output_without_overwrite` | R03 / AC06 / E05 | base bundle | Existing root is not overwritten | yes | yes | yes | none |
| 7 | `phase_d_cli_overwrite_rejects_unmanaged_entry` | R03 / AC07 / E06 | N-F28 | Unmanaged entry is left unchanged | yes | yes | yes | none |
| 8 | `phase_d_reads_only_canonical_artifacts` | R04 / AC08 / E07 | N-F14 | Canonical type reader rejects wrong kind | yes | yes | yes | none |
| 9 | `phase_d_rejects_unsupported_optional_schema` | R04 / AC09 / E08 | N-F13 | Optional schema policy rejects schema 2 | yes | yes | yes | none |
| 10 | `phase_d_catalog_reader_accepts_schema1_and_canonical_order` | R05 / AC10 / E09 | N-F10 | Catalog reader verifies schema/order | yes | yes | yes | none |
| 11 | `phase_d_catalog_reader_rejects_schema2` | R05 / AC11 / E10 | N-X03 | Catalog schema 2 fails closed | yes | yes | yes | none |
| 12 | `phase_d_catalog_reader_rejects_key_identity_mismatch` | R05 / AC12 / E10 | N-X05 | Catalog key/identity mismatch fails | yes | yes | yes | none |
| 13 | `phase_d_catalog_reader_rejects_duplicate_json_key` | R05 / AC13 / E10 | N-X04 | Duplicate root JSON key fails | yes | yes | yes | none |
| 14 | `phase_d_reporting_never_ad_hoc_parses_catalog` | R05 / AC14 / E11 | source guard | Reporting delegates catalog parsing to domain | yes | yes | yes | none |
| 15 | `phase_d_required_known_scope_mismatch_is_rejected` | R06 / AC15 / E12 | sensor mismatch | First known scope mismatch fails | yes | yes | yes | none |
| 16 | `phase_d_required_experiment_mismatch_is_rejected` | R06 / AC16 / E12 | experiment mismatch | Experiment axis mismatch fails | yes | yes | yes | none |
| 17 | `phase_d_required_equal_unknown_scope_reuses_phase_c_admissibility` | R06 / AC17 / E13 | unknown scope | Equal unknown scope remains compatible | yes | yes | yes | none |
| 18 | `phase_d_required_legacy_unknown_is_explicit` | R06 / AC18 / E14 | legacy required | Legacy state is disclosed, not inferred | yes | yes | yes | none |
| 19 | `phase_d_optional_known_mismatch_is_rejected_when_unselected` | R07 / AC19 / E15 | optional mismatch | Supplied optional mismatch fails before selection | yes | yes | yes | none |
| 20 | `phase_d_optional_legacy_unknown_is_limited_not_inferred` | R07 / AC20 / E16 | legacy optional | Legacy optional has no inferred scope/family claim | yes | yes | yes | none |
| 21 | `phase_d_schema4_health_projects_exactly_nine_dimensions` | R08 / AC21 / E17 | N-F02 | Nine serialized dimensions project in order | yes | yes | yes | none |
| 22 | `phase_d_schema3_health_does_not_synthesize_phase_c` | R08 / AC22 / E18 | N-F11 | Legacy health projects no invented dimensions | yes | yes | yes | none |
| 23 | `phase_d_legacy_mechanism_marks_phase_b_assessment_unavailable` | R08 / AC23 / E18 | N-F12 | Legacy mechanism limitation is explicit | yes | yes | yes | none |
| 24 | `phase_d_public_summary_schema1_is_closed_and_ordered` | R09 / AC24 / E19 | base bundle | Typed summary schema/order is stable | yes | yes | yes | none |
| 25 | `phase_d_public_summary_field_authorities_are_typed_copies` | R09 / AC25 / E19 | base bundle | Summary fields copy serialized authority | yes | yes | yes | none |
| 26 | `phase_d_render_manifest_schema1_records_semantic_fields` | R10 / AC26 / E19 | base bundle | Manifest records semantic route/output state | yes | yes | yes | none |
| 27 | `phase_d_render_manifest_orders_paths_and_legacy_notices` | R10 / AC27 / E20 | legacy bundle | Manifest paths/notices are deterministic | yes | yes | yes | none |
| 28 | `phase_d_markdown_sections_and_order_are_stable` | R11 / AC28 / E21 | base bundle | Twelve Markdown sections keep fixed order | yes | yes | yes | none |
| 29 | `phase_d_mechanism_table_projects_serialized_gate_statuses` | R12 / AC29 / E21 | base bundle | Mechanism table copies gates/statuses | yes | yes | yes | none |
| 30 | `phase_d_health_table_preserves_dqi_reason_codes` | R12 / AC30 / E17 | N-F02 | DQI evidence/reasons stay visible | yes | yes | yes | none |
| 31 | `phase_d_health_table_preserves_indeterminate_reason_codes` | R12 / AC31 / E17 | N-F02 | Indeterminate evidence/reasons stay visible | yes | yes | yes | none |
| 32 | `phase_d_evidence_provenance_csv_is_deterministic` | R12 / AC32 / E21 | base bundle | Provenance CSV ordering is stable | yes | yes | yes | none |
| 33 | `phase_d_artifact_lineage_table_projects_root_and_direct_dependency_rows` | R12 / AC33 / E21 | base/legacy | Roots plus direct dependencies only | yes | yes | yes | none |
| 34 | `phase_d_timescale_table_uses_only_serialized_comparisons` | R12 / AC34 / E22 | N-F31 | Timescale table does not rematch | yes | yes | yes | none |
| 35 | `phase_d_current_baseline_csv_uses_unique_feature_unit_authority` | R12 / AC35 / E23 | warning baseline | Unique matching feature unit is required | yes | yes | yes | none |
| 36 | `phase_d_current_baseline_csv_marks_missing_unit_authority` | R12 / AC36 / E24 | N-F22 | Missing/ambiguous unit is unavailable | yes | yes | yes | none |
| 37 | `phase_d_model_consistency_csv_never_recomputes_residual` | R12 / AC37 / E25 | model missing | Serialized residuals are copied | yes | yes | yes | none |
| 38 | `phase_d_figure_mechanism_uses_stored_log_distance_only` | R13 / AC38 / E22 | N-F31 | Stored log distance only | yes | yes | yes | none |
| 39 | `phase_d_figure_health_shows_all_nine_statuses` | R13 / AC39 / E17 | N-F02 | All nine categorical statuses shown | yes | yes | yes | none |
| 40 | `phase_d_figure_baseline_uses_unique_feature_unit_authority` | R13 / AC40 / E23 | warning baseline | Unit authority gates baseline figure | yes | yes | yes | none |
| 41 | `phase_d_figure_eis_nyquist_uses_direct_serialized_imaginary_values` | R13 / AC41 / E26 | N-F24 | Direct serialized imaginary values | yes | yes | yes | none |
| 42 | `phase_d_figure_eis_bode_projects_serialized_frequency_magnitude_phase` | R13 / AC42 / E26 | N-F24 | Serialized Bode channels only | yes | yes | yes | none |
| 43 | `phase_d_figure_transient_renders_one_unique_selected_fit` | R13 / AC43 / E27 | base transient | Unique selected fit only | yes | yes | yes | none |
| 44 | `phase_d_figure_transient_zero_match_default_is_manifest_unavailable` | R13 / AC44 / E28 | N-F25 | Default zero match is unavailable | yes | yes | yes | none |
| 45 | `phase_d_figure_transient_zero_match_explicit_fails_atomically` | R13 / AC45 / E28 | N-F25 | Explicit zero match publishes nothing | yes | yes | yes | none |
| 46 | `phase_d_figure_transient_duplicate_match_is_never_first_selected` | R13 / AC46 / E29 | N-F26 | Ambiguous fit is never selected | yes | yes | yes | none |
| 47 | `phase_d_figure_calibration_has_no_theoretical_line` | R13 / AC47 / E30 | base calibration | Validation values only | yes | yes | yes | none |
| 48 | `phase_d_figure_signal_marks_missing_samples` | R13 / AC48 / E31 | base signal | Serialized signal diagnostics only | yes | yes | yes | none |
| 49 | `phase_d_figure_estimation_shows_serialized_uncertainty_only` | R13 / AC49 / E32 | base estimation | No variance-derived intervals | yes | yes | yes | none |
| 50 | `phase_d_figure_model_never_maps_missing_to_zero` | R13 / AC50 / E25 | N-F27 | Missing values remain NA | yes | yes | yes | none |
| 51 | `phase_d_figure_lineage_marks_legacy_unknown` | R13 / AC51 / E33 | legacy bundle | Legacy lineage wording is explicit | yes | yes | yes | none |
| 52 | `phase_d_selected_figure_files_are_valid_svg_and_png` | R14 / AC52 / E34 | EIS plot | SVG/PNG parse and dimensions validate | yes | yes | yes | none |
| 53 | `phase_d_figure_metadata_has_labels_units_series_and_dqi_visibility` | R14 / AC53 / E34 | EIS/health | Labels, units, series, DQI text visible | yes | yes | yes | none |
| 54 | `phase_d_format_json_writes_summary_manifest_and_selected_visuals` | R15 / AC54 / E35 | JSON bundle | JSON+manifest, no Markdown | yes | yes | yes | none |
| 55 | `phase_d_format_markdown_writes_report_manifest_and_selected_visuals` | R15 / AC55 / E35 | Markdown bundle | Markdown+manifest, no summary | yes | yes | yes | none |
| 56 | `phase_d_default_selection_is_best_effort_and_explicit_all_is_strict` | R15 / AC56 / E36 | no transient | Default records; explicit fails | yes | yes | yes | none |
| 57 | `phase_d_public_float_format_is_exact` | R16 / AC57 / E37 | numeric matrix | Exact finite/negative-zero formatter | yes | yes | yes | none |
| 58 | `phase_d_csv_markdown_and_figure_annotations_share_float_format` | R16 / AC58 / E37 | N-F31 | Public number spelling agrees | yes | yes | yes | none |
| 59 | `phase_d_nonfinite_projection_fails_before_serialization` | R16 / AC59 / E37 | injected NaN | Nonfinite values fail closed | yes | yes | yes | none |
| 60 | `phase_d_staging_write_failure_publishes_no_final_bundle` | R17 / AC60 / E38 | write failure | No final partial bundle | yes | yes | yes | none |
| 61 | `phase_d_publication_failure_restores_previous_complete_bundle` | R17 / AC61 / E38 | managed overwrite | Managed bundle remains complete | yes | yes | yes | none |
| 62 | `phase_d_rendering_does_not_mutate_health_assessment` | R18 / AC62 / E39 | base health | Input bytes unchanged | yes | yes | yes | none |
| 63 | `phase_d_rendering_does_not_mutate_mechanism_assessment` | R18 / AC63 / E39 | base mechanism | Input bytes unchanged | yes | yes | yes | none |
| 64 | `phase_d_repeated_render_is_deterministic` | R19 / AC64 / E39 | base bundle | Public text bytes identical | yes | yes | yes | none |
| 65 | `phase_d_large_history_does_not_duplicate_artifact_series_unboundedly` | R20 / AC65 / E40 | N-F29/N-F30 | Large fields read once and preserved | yes | yes | yes | none |
| 66 | `phase_d_golden_expectations_are_hand_derived_from_fixture_literals` | R21 / AC66 / E41 | sealed ledger | No renderer-generated golden files | yes | yes | yes | none |
| 67 | `phase_d_public_report_error_is_publicly_reachable` | R22 / AC67 / E42 | public API | Public error converts to RunnerError | yes | yes | yes | none |
| 68 | `phase_d_catalog_reader_rejects_syntactically_malformed_json` | R05 / AC68 / E43 | N-X01 | Malformed bytes raise Json | yes | yes | yes | none |
| 69 | `phase_d_catalog_reader_rejects_structurally_invalid_catalog` | R05 / AC69 / E43 | N-X02 | Closed unknown field is not Json | yes | yes | yes | none |
| 70 | `phase_d_different_known_acquisition_families_are_projected_not_rejected` | R06 / AC70 / E44 | family bundle | Families project; no equality gate | yes | yes | yes | none |
| 71 | `phase_d_comparable_with_warnings_is_rendered_and_disclosed` | R13 / AC71 / E45 | N-F23 | Figure/table/manifest disclose warning | yes | yes | yes | none |
| 72 | `phase_d_lineage_catalog_input_reference_is_catalog_variant_without_artifact_fields` | R09/R10 / AC72 / E46 | N-F10 | Catalog tagged variant has no artifact fields | yes | yes | yes | none |
| 73 | `phase_d_fixture_ledger_materializes_exact_literal_files_and_canonical_readers_accept_them` | R23 / AC73 / E47 | all 36 | Sealed hashes/readers/errors/scales verify | yes | yes | yes | none |
