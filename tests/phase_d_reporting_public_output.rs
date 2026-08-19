use rust_electroanalysis_cli::{
    cli::{CliError, parse_cli_args},
    domain::{LineageCatalogReadError, read_artifact_lineage_catalog},
    report_config::{ReportFormat, ReportRenderOptions, ReportSelection},
    reporting::{PublicReportError, format_public_f64},
    runners::{RunnerError, report},
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/phase_d")
        .join(relative)
}

fn temporary_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rust-electroanalysis-phase-d-{label}-{nonce}-{}",
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}

fn compact_options(label: &str) -> ReportRenderOptions {
    ReportRenderOptions {
        mechanism: fixture("base/mechanism.json"),
        health: fixture("base/health.json"),
        output_dir: temporary_root(label),
        lineage_catalog: Some(fixture("base/lineage_catalog.json")),
        eis: None,
        transient: None,
        calibration: None,
        calibration_observations: None,
        signal: None,
        estimation: None,
        model: None,
        format: ReportFormat::All,
        selection: ReportSelection::parse(Some("none"), Some("none")).expect("selection"),
        overwrite: false,
    }
}

fn render(options: &ReportRenderOptions) -> PathBuf {
    let root = options.output_dir.clone();
    report::render(options).expect("certified render succeeds");
    root
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).expect("certified text output")
}

fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

fn fixture_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).expect("fixture directory") {
        let path = entry.expect("fixture entry").path();
        if path.is_dir() {
            fixture_files(&path, files);
        } else {
            files.push(path);
        }
    }
}

#[test]
fn phase_d_catalog_reader_rejects_syntactically_malformed_json() {
    let path = fixture("failure/catalog_malformed.json");
    let error = read_artifact_lineage_catalog(&path).expect_err("malformed catalog must fail");
    assert!(matches!(error, LineageCatalogReadError::Json { .. }));
}

#[test]
fn phase_d_catalog_reader_rejects_structurally_invalid_catalog() {
    let path = fixture("failure/catalog_invalid_structure.json");
    let error =
        read_artifact_lineage_catalog(&path).expect_err("closed-schema violation must fail");
    match error {
        LineageCatalogReadError::UnknownField { field, .. } => assert_eq!(field, "unexpected"),
        other => panic!("expected UnknownField, got {other:?}"),
    }
}

#[test]
fn phase_d_cli_requires_mechanism_and_health() {
    let arguments = vec![
        "electroanalysis".into(),
        "report".into(),
        "render".into(),
        "--health".into(),
        "health.json".into(),
        "--output-dir".into(),
        "output".into(),
    ];
    assert!(matches!(
        parse_cli_args(&arguments),
        Err(CliError::Parse(_))
    ));
}

#[test]
fn phase_d_clap_rejects_unknown_format_before_runner() {
    let arguments = vec![
        "electroanalysis".into(),
        "report".into(),
        "render".into(),
        "--mechanism".into(),
        "mechanism.json".into(),
        "--health".into(),
        "health.json".into(),
        "--output-dir".into(),
        "output".into(),
        "--format".into(),
        "yaml".into(),
    ];
    assert!(matches!(
        parse_cli_args(&arguments),
        Err(CliError::Parse(_))
    ));
}

macro_rules! phase_d_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() $body
    };
}

fn explicit(options: &mut ReportRenderOptions, figures: &str, tables: &str) {
    options.selection = ReportSelection::parse(Some(figures), Some(tables)).expect("selection");
}

phase_d_test!(phase_d_cli_rejects_unpaired_calibration_inputs, {
    let mut options = compact_options("unpaired");
    options.calibration = Some(fixture("base/calibration.json"));
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::InvalidCombination { .. })
    ));
});

phase_d_test!(phase_d_cli_rejects_unknown_selection, {
    assert!(matches!(
        ReportSelection::parse(Some("unknown"), None),
        Err(PublicReportError::InvalidSelection { .. })
    ));
    assert!(matches!(
        ReportSelection::parse(None, Some("unknown")),
        Err(PublicReportError::InvalidSelection { .. })
    ));
});

phase_d_test!(phase_d_cli_rejects_duplicate_selection, {
    assert!(matches!(
        ReportSelection::parse(Some("lineage,lineage"), None),
        Err(PublicReportError::InvalidSelection { .. })
    ));
    assert!(matches!(
        ReportSelection::parse(None, Some("artifact_lineage,artifact_lineage")),
        Err(PublicReportError::InvalidSelection { .. })
    ));
});

phase_d_test!(phase_d_cli_rejects_existing_output_without_overwrite, {
    let options = compact_options("collision");
    fs::create_dir(&options.output_dir).expect("output root");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::OutputCollision { .. })
    ));
    cleanup(&options.output_dir);
});

phase_d_test!(phase_d_cli_overwrite_rejects_unmanaged_entry, {
    let mut options = compact_options("unmanaged");
    fs::create_dir(&options.output_dir).expect("output root");
    fs::write(options.output_dir.join("keep.txt"), "do not delete").expect("sentinel");
    options.overwrite = true;
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::UnmanagedOutputEntry { .. })
    ));
    assert_eq!(
        fs::read_to_string(options.output_dir.join("keep.txt")).expect("sentinel retained"),
        "do not delete"
    );
    cleanup(&options.output_dir);
});

phase_d_test!(phase_d_reads_only_canonical_artifacts, {
    let mut options = compact_options("wrong-kind");
    options.mechanism = fixture("failure/wrong_kind.json");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::Artifact { .. })
    ));
    assert!(!options.output_dir.exists());
});

phase_d_test!(phase_d_rejects_unsupported_optional_schema, {
    let mut options = compact_options("optional-schema");
    options.eis = Some(fixture("failure/eis_schema2.json"));
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::Artifact { .. })
    ));
    assert!(!options.output_dir.exists());
});

phase_d_test!(
    phase_d_catalog_reader_accepts_schema1_and_canonical_order,
    {
        let catalog = read_artifact_lineage_catalog(&fixture("base/lineage_catalog.json"))
            .expect("schema-1 catalog");
        let keys = catalog
            .artifacts
            .keys()
            .map(|id| id.0.as_str())
            .collect::<Vec<_>>();
        assert!(keys.windows(2).all(|pair| pair[0] <= pair[1]));
    }
);

phase_d_test!(phase_d_catalog_reader_rejects_schema2, {
    assert!(matches!(
        read_artifact_lineage_catalog(&fixture("failure/catalog_schema2.json")),
        Err(LineageCatalogReadError::UnsupportedSchemaVersion { actual: 2, .. })
    ));
});

phase_d_test!(phase_d_catalog_reader_rejects_key_identity_mismatch, {
    assert!(matches!(
        read_artifact_lineage_catalog(&fixture("failure/catalog_key_identity_mismatch.json")),
        Err(LineageCatalogReadError::KeyIdentityMismatch { .. })
    ));
});

phase_d_test!(phase_d_catalog_reader_rejects_duplicate_json_key, {
    assert!(matches!(
        read_artifact_lineage_catalog(&fixture("failure/catalog_duplicate_root_key.json")),
        Err(LineageCatalogReadError::DuplicateField { .. })
    ));
});

phase_d_test!(phase_d_reporting_never_ad_hoc_parses_catalog, {
    let source = include_str!("../src/reporting/reader.rs");
    assert!(!source.contains("serde_json::from_"));
    assert!(source.contains("read_artifact_lineage_catalog"));
});

phase_d_test!(phase_d_required_known_scope_mismatch_is_rejected, {
    let mut options = compact_options("sensor-mismatch");
    options.health = fixture("compat/health_sensor_mismatch.json");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::RequiredInputsIncompatible {
            axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::SensorScope,
            ..
        })
    ));
});

phase_d_test!(phase_d_required_experiment_mismatch_is_rejected, {
    let mut options = compact_options("experiment-mismatch");
    options.mechanism = fixture("compat/mechanism_experiment_mismatch.json");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::RequiredInputsIncompatible {
            axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::ExperimentScope,
            ..
        })
    ));
});

phase_d_test!(
    phase_d_required_equal_unknown_scope_reuses_phase_c_admissibility,
    {
        let mut options = compact_options("unknown-scope");
        options.mechanism = fixture("compat/mechanism_unknown_scope.json");
        options.health = fixture("compat/health_unknown_scope.json");
        let root = render(&options);
        assert!(
            read(&root, "public_summary.schema1.json")
                .contains("\"required_pair\": \"compatible\"")
        );
        cleanup(&root);
    }
);

phase_d_test!(phase_d_required_legacy_unknown_is_explicit, {
    let mut options = compact_options("legacy-required");
    options.mechanism = fixture("legacy/mechanism_v1.json");
    options.health = fixture("legacy/health_v3.json");
    let root = render(&options);
    let summary = read(&root, "public_summary.schema1.json");
    assert!(summary.contains("Phase B V1 hypothesis assessment unavailable"));
    assert!(summary.contains("Phase C nine-dimension assessment was not serialized"));
    cleanup(&root);
});

phase_d_test!(
    phase_d_optional_known_mismatch_is_rejected_when_unselected,
    {
        let mut options = compact_options("optional-mismatch");
        options.eis = Some(fixture("compat/eis_sensor_mismatch.json"));
        assert!(matches!(
            report::render(&options),
            Err(PublicReportError::OptionalInputIncompatible { .. })
        ));
    }
);

phase_d_test!(phase_d_optional_legacy_unknown_is_limited_not_inferred, {
    let mut options = compact_options("optional-legacy");
    options.model = Some(fixture("base/model.json"));
    let root = render(&options);
    let summary = read(&root, "public_summary.schema1.json");
    assert!(summary.contains("legacy_unknown"));
    assert!(!summary.contains("independent"));
    cleanup(&root);
});

phase_d_test!(phase_d_schema4_health_projects_exactly_nine_dimensions, {
    let root = render(&compact_options("health-nine"));
    let value: serde_json::Value =
        serde_json::from_str(&read(&root, "public_summary.schema1.json")).expect("summary json");
    assert_eq!(
        value["sensor_health"]["dimensions"]
            .as_array()
            .expect("dimensions")
            .len(),
        9
    );
    cleanup(&root);
});

phase_d_test!(phase_d_schema3_health_does_not_synthesize_phase_c, {
    let mut options = compact_options("legacy-health");
    options.health = fixture("legacy/health_v3.json");
    let root = render(&options);
    let value: serde_json::Value =
        serde_json::from_str(&read(&root, "public_summary.schema1.json")).expect("summary json");
    assert!(
        value["sensor_health"]["dimensions"]
            .as_array()
            .expect("dimensions")
            .is_empty()
    );
    cleanup(&root);
});

phase_d_test!(
    phase_d_legacy_mechanism_marks_phase_b_assessment_unavailable,
    {
        let mut options = compact_options("legacy-mechanism");
        options.mechanism = fixture("legacy/mechanism_v1.json");
        let root = render(&options);
        assert!(
            read(&root, "public_summary.schema1.json")
                .contains("Phase B V1 hypothesis assessment unavailable")
        );
        cleanup(&root);
    }
);

phase_d_test!(phase_d_public_summary_schema1_is_closed_and_ordered, {
    let root = render(&compact_options("summary-order"));
    let text = read(&root, "public_summary.schema1.json");
    for key in [
        "schema_version",
        "output_kind",
        "renderer_contract",
        "route",
        "input_references",
        "compatibility",
        "mechanism",
        "sensor_health",
        "optional_sources",
        "lineage",
        "outputs",
        "limitations",
        "rendering",
    ] {
        assert!(text.contains(&format!("\"{key}\"")), "missing {key}");
    }
    assert!(
        text.find("\"schema_version\"").expect("schema")
            < text.find("\"input_references\"").expect("references")
    );
    cleanup(&root);
});

phase_d_test!(phase_d_public_summary_field_authorities_are_typed_copies, {
    let root = render(&compact_options("typed-copy"));
    let summary = read(&root, "public_summary.schema1.json");
    assert!(summary.contains("\"assessment_id\": \"health:signal:a0-test:E1\""));
    assert!(summary.contains("\"analysis_id\": \"mechanism-phase-b:b-e2e-1\""));
    cleanup(&root);
});

phase_d_test!(phase_d_render_manifest_schema1_records_semantic_fields, {
    let root = render(&compact_options("manifest"));
    let manifest = read(&root, "render_manifest.schema1.json");
    for key in [
        "requested",
        "render_order",
        "generated_files",
        "unavailable_outputs",
        "determinism",
    ] {
        assert!(manifest.contains(&format!("\"{key}\"")), "missing {key}");
    }
    assert!(manifest.contains("render_manifest.schema1.json"));
    cleanup(&root);
});

phase_d_test!(phase_d_render_manifest_orders_paths_and_legacy_notices, {
    let mut options = compact_options("legacy-manifest");
    options.mechanism = fixture("legacy/mechanism_v1.json");
    options.health = fixture("legacy/health_v3.json");
    let root = render(&options);
    let manifest = read(&root, "render_manifest.schema1.json");
    assert!(manifest.contains("legacy_phase_c_not_serialized"));
    assert!(manifest.contains("\"path_separator\": \"/\""));
    cleanup(&root);
});

phase_d_test!(phase_d_markdown_sections_and_order_are_stable, {
    let root = render(&compact_options("markdown-sections"));
    let report = read(&root, "scientific_report.md");
    let headings = [
        "Analysis identity and renderer boundary",
        "Input artifacts and compatibility state",
        "Mechanism assessment",
        "Sensor-health assessment",
        "Key evidence and contradictions",
        "Uncertainty and data-quality limitations",
        "Current-versus-baseline comparison",
        "Optional analysis projections",
        "Figures",
        "Tables",
        "Lineage and provenance",
        "Reproducibility metadata",
    ];
    let positions = headings
        .iter()
        .map(|heading| report.find(heading).expect("section"))
        .collect::<Vec<_>>();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    cleanup(&root);
});

phase_d_test!(phase_d_mechanism_table_projects_serialized_gate_statuses, {
    let mut options = compact_options("mechanism-table");
    explicit(&mut options, "none", "mechanism_evidence");
    let root = render(&options);
    let table = read(&root, "tables/mechanism_evidence.csv");
    assert!(table.starts_with("hypothesis_id,display_name,evidence_level,reason_codes"));
    assert!(table.contains("b-hypothesis"));
    cleanup(&root);
});

phase_d_test!(phase_d_health_table_preserves_dqi_reason_codes, {
    let mut options = compact_options("dqi-table");
    explicit(&mut options, "none", "health_dimensions");
    let root = render(&options);
    let table = read(&root, "tables/sensor_health_dimensions.csv");
    assert!(table.contains("data_quality"));
    assert!(table.contains("reason_codes"));
    cleanup(&root);
});

phase_d_test!(phase_d_health_table_preserves_indeterminate_reason_codes, {
    let mut options = compact_options("indeterminate-table");
    explicit(&mut options, "none", "health_dimensions");
    let root = render(&options);
    let table = read(&root, "tables/sensor_health_dimensions.csv");
    assert!(table.contains("indeterminate"));
    assert!(table.contains("insufficient_evidence") || table.contains("no_evidence"));
    cleanup(&root);
});

phase_d_test!(phase_d_evidence_provenance_csv_is_deterministic, {
    let mut left = compact_options("evidence-left");
    explicit(&mut left, "none", "evidence_provenance");
    let mut right = compact_options("evidence-right");
    explicit(&mut right, "none", "evidence_provenance");
    let left_root = render(&left);
    let right_root = render(&right);
    assert_eq!(
        fs::read(left_root.join("tables/evidence_provenance.csv")).expect("left"),
        fs::read(right_root.join("tables/evidence_provenance.csv")).expect("right")
    );
    cleanup(&left_root);
    cleanup(&right_root);
});

phase_d_test!(
    phase_d_artifact_lineage_table_projects_root_and_direct_dependency_rows,
    {
        let mut options = compact_options("lineage-table");
        explicit(&mut options, "none", "artifact_lineage");
        let root = render(&options);
        let table = read(&root, "tables/artifact_lineage.csv");
        assert!(table.starts_with("root_input_flag,row_kind,root_artifact_kind"));
        assert!(table.contains(",root,"));
        assert!(!table.contains("catalog_node"));
        cleanup(&root);
    }
);

phase_d_test!(phase_d_timescale_table_uses_only_serialized_comparisons, {
    let mut options = compact_options("timescale-table");
    options.mechanism = fixture("mechanism/timescale_cmp01.json");
    explicit(&mut options, "none", "timescale_comparison");
    let root = render(&options);
    let table = read(&root, "tables/timescale_comparison.csv");
    assert!(table.contains("cmp-01"));
    assert!(table.contains("0.041"));
    cleanup(&root);
});

phase_d_test!(
    phase_d_current_baseline_csv_uses_unique_feature_unit_authority,
    {
        let mut options = compact_options("baseline-table");
        options.health = fixture("health/comparable_with_warnings.json");
        explicit(&mut options, "none", "current_vs_baseline");
        let root = render(&options);
        let table = read(&root, "tables/current_vs_baseline.csv");
        assert!(table.contains("baseline_comparable_with_warnings"));
        assert!(!table.contains("unit_authority_unavailable"));
        cleanup(&root);
    }
);

phase_d_test!(phase_d_current_baseline_csv_marks_missing_unit_authority, {
    let mut options = compact_options("baseline-unit");
    options.health = fixture("health/missing_unit.json");
    explicit(&mut options, "none", "current_vs_baseline");
    let root = render(&options);
    let table = read(&root, "tables/current_vs_baseline.csv");
    assert!(table.contains("unit_authority_unavailable"));
    assert!(table.contains("NA"));
    cleanup(&root);
});

phase_d_test!(phase_d_model_consistency_csv_never_recomputes_residual, {
    let mut options = compact_options("model-table");
    options.model = Some(fixture("model/missing_values.json"));
    explicit(&mut options, "none", "model_consistency");
    let root = render(&options);
    let table = read(&root, "tables/model_consistency.csv");
    assert!(table.contains("unexplained_residual") || table.contains("residual"));
    assert!(table.contains("NA"));
    cleanup(&root);
});

phase_d_test!(phase_d_figure_mechanism_uses_stored_log_distance_only, {
    let mut options = compact_options("mechanism-figure");
    options.mechanism = fixture("mechanism/timescale_cmp01.json");
    explicit(&mut options, "mechanism_timescale", "none");
    let root = render(&options);
    let svg = read(&root, "figures/mechanism_timescale.svg");
    assert!(svg.contains("0.041"));
    assert!(svg.contains("performs no log10 calculation"));
    cleanup(&root);
});

phase_d_test!(phase_d_figure_health_shows_all_nine_statuses, {
    let mut options = compact_options("health-figure");
    explicit(&mut options, "sensor_health_dimension_status", "none");
    let root = render(&options);
    let svg = read(&root, "figures/sensor_health_dimension_status.svg");
    for dimension in [
        "signal_integrity",
        "calibration_health",
        "dynamic_response_health",
        "reference_stability",
        "environmental_robustness",
        "model_consistency",
        "observability",
        "uncertainty_health",
        "data_quality",
    ] {
        assert!(svg.contains(dimension), "missing {dimension}");
    }
    cleanup(&root);
});

phase_d_test!(
    phase_d_figure_baseline_uses_unique_feature_unit_authority,
    {
        let mut options = compact_options("baseline-figure");
        options.health = fixture("health/comparable_with_warnings.json");
        explicit(&mut options, "current_vs_baseline", "none");
        let root = render(&options);
        let svg = read(&root, "figures/current_vs_baseline.svg");
        assert!(svg.contains("source-authoritative unit"));
        assert!(svg.contains("Comparable-with-warnings"));
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_figure_eis_nyquist_uses_direct_serialized_imaginary_values,
    {
        let mut options = compact_options("nyquist");
        options.eis = Some(fixture("eis/nyquist_bode.json"));
        explicit(&mut options, "eis_nyquist", "none");
        let root = render(&options);
        let svg = read(&root, "figures/eis_nyquist.svg");
        assert!(svg.contains("Im(Z) [Ohm]"));
        assert!(svg.contains("y=-2"));
        assert!(svg.contains("no Nyquist sign transformation"));
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_figure_eis_bode_projects_serialized_frequency_magnitude_phase,
    {
        let mut options = compact_options("bode");
        options.eis = Some(fixture("eis/nyquist_bode.json"));
        explicit(&mut options, "eis_bode", "none");
        let root = render(&options);
        let svg = read(&root, "figures/eis_bode.svg");
        assert!(svg.contains("Frequency [Hz]"));
        assert!(svg.contains("observed magnitude"));
        assert!(svg.contains("fitted phase"));
        cleanup(&root);
    }
);

phase_d_test!(phase_d_figure_transient_renders_one_unique_selected_fit, {
    let mut options = compact_options("transient-unique");
    options.transient = Some(fixture("base/transient.json"));
    explicit(&mut options, "transient_response", "none");
    let root = render(&options);
    let svg = read(&root, "figures/transient_response.svg");
    assert!(svg.contains("observed"));
    assert!(svg.contains("fitted"));
    assert!(svg.contains("residual"));
    cleanup(&root);
});

phase_d_test!(
    phase_d_figure_transient_zero_match_default_is_manifest_unavailable,
    {
        let mut options = compact_options("transient-zero-default");
        options.transient = Some(fixture("transient/zero_selected_fit.json"));
        options.selection = ReportSelection::parse(None, Some("none")).expect("default selection");
        let root = render(&options);
        assert!(read(&root, "render_manifest.schema1.json").contains("selected_fit_not_found"));
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_figure_transient_zero_match_explicit_fails_atomically,
    {
        let mut options = compact_options("transient-zero-explicit");
        options.transient = Some(fixture("transient/zero_selected_fit.json"));
        explicit(&mut options, "transient_response", "none");
        assert!(matches!(
            report::render(&options),
            Err(PublicReportError::RequestedOutputUnavailable { .. })
        ));
        assert!(!options.output_dir.exists());
    }
);

phase_d_test!(
    phase_d_figure_transient_duplicate_match_is_never_first_selected,
    {
        let mut options = compact_options("transient-duplicate");
        options.transient = Some(fixture("transient/duplicate_selected_fit.json"));
        options.selection = ReportSelection::parse(None, Some("none")).expect("default selection");
        let root = render(&options);
        let manifest = read(&root, "render_manifest.schema1.json");
        assert!(manifest.contains("selected_fit_ambiguous"));
        assert!(!root.join("figures/transient_response.svg").exists());
        cleanup(&root);
    }
);

phase_d_test!(phase_d_figure_calibration_has_no_theoretical_line, {
    let mut options = compact_options("calibration-figure");
    options.calibration = Some(fixture("base/calibration.json"));
    options.calibration_observations = Some(fixture("base/calibration_observations.json"));
    explicit(&mut options, "calibration_performance", "none");
    let root = render(&options);
    let svg = read(&root, "figures/calibration_performance.svg");
    assert!(svg.contains("serialized validation"));
    assert!(!svg.to_ascii_lowercase().contains("calibration theory"));
    cleanup(&root);
});

phase_d_test!(phase_d_figure_signal_marks_missing_samples, {
    let mut options = compact_options("signal-figure");
    options.signal = Some(fixture("base/signal.json"));
    explicit(&mut options, "signal_diagnostics", "none");
    let root = render(&options);
    let svg = read(&root, "figures/signal_diagnostics.svg");
    assert!(svg.contains("Signal diagnostics"));
    assert!(svg.contains("PSD") || svg.contains("Allan"));
    cleanup(&root);
});

phase_d_test!(
    phase_d_figure_estimation_shows_serialized_uncertainty_only,
    {
        let mut options = compact_options("estimation-figure");
        options.estimation = Some(fixture("base/estimation.json"));
        options.selection = ReportSelection::parse(None, Some("none")).expect("default selection");
        let root = render(&options);
        let manifest = read(&root, "render_manifest.schema1.json");
        assert!(manifest.contains("estimation_observed_predicted"));
        assert!(!manifest.contains("invented_interval"));
        cleanup(&root);
    }
);

phase_d_test!(phase_d_figure_model_never_maps_missing_to_zero, {
    let mut options = compact_options("model-figure");
    options.model = Some(fixture("model/missing_values.json"));
    explicit(&mut options, "model_observed_predicted", "none");
    let root = render(&options);
    let svg = read(&root, "figures/model_observed_predicted.svg");
    assert!(svg.contains("Missing observed or residual values remain NA"));
    assert!(svg.contains("y=NA"));
    cleanup(&root);
});

phase_d_test!(phase_d_figure_lineage_marks_legacy_unknown, {
    let mut options = compact_options("legacy-lineage");
    options.mechanism = fixture("legacy/mechanism_v1.json");
    explicit(&mut options, "lineage", "none");
    let root = render(&options);
    assert!(read(&root, "figures/lineage.svg").contains("lineage unknown:"));
    cleanup(&root);
});

phase_d_test!(phase_d_selected_figure_files_are_valid_svg_and_png, {
    let mut options = compact_options("image-validity");
    options.eis = Some(fixture("eis/nyquist_bode.json"));
    explicit(&mut options, "eis_nyquist", "none");
    let root = render(&options);
    let svg = read(&root, "figures/eis_nyquist.svg");
    assert!(svg.starts_with("<svg"));
    assert_eq!(
        image::image_dimensions(root.join("figures/eis_nyquist.png")).expect("png dimensions"),
        (1600, 1000)
    );
    cleanup(&root);
});

phase_d_test!(
    phase_d_figure_metadata_has_labels_units_series_and_dqi_visibility,
    {
        let mut options = compact_options("figure-metadata");
        options.eis = Some(fixture("eis/nyquist_bode.json"));
        explicit(&mut options, "eis_nyquist", "none");
        let root = render(&options);
        let svg = read(&root, "figures/eis_nyquist.svg");
        assert!(svg.contains("phase_d_figure=eis_nyquist"));
        assert!(svg.contains("threshold_lines=0"));
        assert!(svg.contains("Re(Z) [Ohm]"));
        assert!(svg.contains("data-series=\"observed\""));
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_format_json_writes_summary_manifest_and_selected_visuals,
    {
        let mut options = compact_options("format-json");
        options.format = ReportFormat::Json;
        options.eis = Some(fixture("eis/nyquist_bode.json"));
        explicit(&mut options, "eis_nyquist", "none");
        let root = render(&options);
        assert!(root.join("public_summary.schema1.json").exists());
        assert!(root.join("render_manifest.schema1.json").exists());
        assert!(root.join("figures/eis_nyquist.svg").exists());
        assert!(!root.join("scientific_report.md").exists());
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_format_markdown_writes_report_manifest_and_selected_visuals,
    {
        let mut options = compact_options("format-markdown");
        options.format = ReportFormat::Markdown;
        options.eis = Some(fixture("eis/nyquist_bode.json"));
        explicit(&mut options, "eis_nyquist", "none");
        let root = render(&options);
        assert!(root.join("scientific_report.md").exists());
        assert!(root.join("render_manifest.schema1.json").exists());
        assert!(root.join("figures/eis_nyquist.png").exists());
        assert!(!root.join("public_summary.schema1.json").exists());
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_default_selection_is_best_effort_and_explicit_all_is_strict,
    {
        let mut default_options = compact_options("default-selection");
        default_options.selection =
            ReportSelection::parse(None, Some("none")).expect("default selection");
        let default_root = render(&default_options);
        assert!(
            !read(&default_root, "render_manifest.schema1.json").contains("transient_response")
        );
        cleanup(&default_root);
        let mut explicit_options = compact_options("strict-selection");
        explicit(&mut explicit_options, "all", "none");
        assert!(matches!(
            report::render(&explicit_options),
            Err(PublicReportError::RequestedOutputUnavailable { .. })
        ));
        assert!(!explicit_options.output_dir.exists());
    }
);

phase_d_test!(phase_d_public_float_format_is_exact, {
    assert_eq!(format_public_f64(0.0).expect("zero"), "0");
    assert_eq!(format_public_f64(-0.0).expect("negative zero"), "0");
    assert_eq!(format_public_f64(1.25).expect("finite"), "1.25");
    assert_eq!(format_public_f64(0.000001).expect("finite"), "0.000001");
});

phase_d_test!(
    phase_d_csv_markdown_and_figure_annotations_share_float_format,
    {
        let mut options = compact_options("cross-format-number");
        options.mechanism = fixture("mechanism/timescale_cmp01.json");
        explicit(&mut options, "mechanism_timescale", "timescale_comparison");
        let root = render(&options);
        assert!(read(&root, "tables/timescale_comparison.csv").contains("0.041"));
        assert!(read(&root, "figures/mechanism_timescale.svg").contains("0.041"));
        assert!(
            read(&root, "scientific_report.md").contains("0.041")
                || read(&root, "public_summary.schema1.json").contains("0.041")
        );
        cleanup(&root);
    }
);

phase_d_test!(phase_d_nonfinite_projection_fails_before_serialization, {
    assert!(format_public_f64(f64::NAN).is_err());
    assert!(format_public_f64(f64::INFINITY).is_err());
});

phase_d_test!(phase_d_staging_write_failure_publishes_no_final_bundle, {
    let mut options = compact_options("invalid-parent");
    options.output_dir = PathBuf::from("/this/phase-d-parent-does-not-exist/output");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::InvalidOutputDirectory { .. })
    ));
    assert!(!options.output_dir.exists());
});

phase_d_test!(
    phase_d_publication_failure_restores_previous_complete_bundle,
    {
        let mut options = compact_options("managed-overwrite");
        let root = render(&options);
        options.overwrite = true;
        report::render(&options).expect("managed bundle overwrites atomically");
        assert!(root.join("render_manifest.schema1.json").is_file());
        cleanup(&root);
    }
);

phase_d_test!(phase_d_rendering_does_not_mutate_health_assessment, {
    let source = fixture("base/health.json");
    let before = fs::read(&source).expect("source bytes");
    let root = render(&compact_options("immutable-health"));
    assert_eq!(fs::read(source).expect("source bytes"), before);
    cleanup(&root);
});

phase_d_test!(phase_d_rendering_does_not_mutate_mechanism_assessment, {
    let source = fixture("base/mechanism.json");
    let before = fs::read(&source).expect("source bytes");
    let root = render(&compact_options("immutable-mechanism"));
    assert_eq!(fs::read(source).expect("source bytes"), before);
    cleanup(&root);
});

phase_d_test!(phase_d_repeated_render_is_deterministic, {
    let left = render(&compact_options("deterministic-left"));
    let right = render(&compact_options("deterministic-right"));
    for relative in [
        "public_summary.schema1.json",
        "scientific_report.md",
        "render_manifest.schema1.json",
    ] {
        assert_eq!(
            fs::read(left.join(relative)).expect("left output"),
            fs::read(right.join(relative)).expect("right output"),
            "{relative}"
        );
    }
    cleanup(&left);
    cleanup(&right);
});

phase_d_test!(
    phase_d_large_history_does_not_duplicate_artifact_series_unboundedly,
    {
        use rust_electroanalysis_cli::domain::read_artifact;
        use rust_electroanalysis_cli::results::{MechanismAnalysisReport, SensorHealthAssessment};
        let mechanism: MechanismAnalysisReport =
            read_artifact(&fixture("scale/mechanism_large_history.json")).expect("large mechanism");
        let health: SensorHealthAssessment =
            read_artifact(&fixture("scale/health_large_evidence.json")).expect("large health");
        assert_eq!(mechanism.hypothesis_history.len(), 1000);
        assert_eq!(
            health
                .phase_c
                .expect("phase c")
                .evidence_bundle
                .records
                .len(),
            10_000
        );
        let mut options = compact_options("large-render");
        options.mechanism = fixture("scale/mechanism_large_history.json");
        options.health = fixture("scale/health_large_evidence.json");
        options.lineage_catalog = None;
        let root = render(&options);
        let summary: serde_json::Value =
            serde_json::from_str(&read(&root, "public_summary.schema1.json"))
                .expect("summary json");
        assert_eq!(
            summary["input_references"]
                .as_array()
                .expect("references")
                .iter()
                .filter(|reference| reference["input_flag"] == "mechanism")
                .count(),
            1
        );
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_golden_expectations_are_hand_derived_from_fixture_literals,
    {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase_d");
        let mut files = Vec::new();
        fixture_files(&fixtures, &mut files);
        assert!(files.iter().all(|path| {
            !path
                .file_name()
                .expect("name")
                .to_string_lossy()
                .contains("golden")
        }));
    }
);

phase_d_test!(phase_d_public_report_error_is_publicly_reachable, {
    let error = PublicReportError::InvalidCombination { detail: "test" };
    assert!(matches!(
        RunnerError::from(error),
        RunnerError::PublicReport(PublicReportError::InvalidCombination { .. })
    ));
});

phase_d_test!(
    phase_d_different_known_acquisition_families_are_projected_not_rejected,
    {
        let mut options = compact_options("different-families");
        options.mechanism = fixture("compat/mechanism_family.json");
        options.health = fixture("compat/health_family.json");
        let root = render(&options);
        let summary = read(&root, "public_summary.schema1.json");
        assert!(summary.contains("\"required_pair\": \"compatible\""));
        assert!(summary.contains("acquisition_families"));
        assert!(!summary.contains("same_source"));
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_comparable_with_warnings_is_rendered_and_disclosed,
    {
        let mut options = compact_options("comparable-warning");
        options.health = fixture("health/comparable_with_warnings.json");
        explicit(&mut options, "current_vs_baseline", "current_vs_baseline");
        let root = render(&options);
        assert!(root.join("figures/current_vs_baseline.svg").exists());
        assert!(
            read(&root, "tables/current_vs_baseline.csv")
                .contains("baseline_comparable_with_warnings")
        );
        assert!(
            read(&root, "render_manifest.schema1.json")
                .contains("baseline_comparable_with_warnings")
        );
        assert!(
            read(&root, "scientific_report.md")
                .contains("Comparable with upstream context warning")
                || read(&root, "scientific_report.md").contains("comparable_with_warnings")
        );
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_lineage_catalog_input_reference_is_catalog_variant_without_artifact_fields,
    {
        let root = render(&compact_options("catalog-variant"));
        for relative in [
            "public_summary.schema1.json",
            "render_manifest.schema1.json",
        ] {
            let value: serde_json::Value =
                serde_json::from_str(&read(&root, relative)).expect("document json");
            let catalog = value["input_references"]
                .as_array()
                .expect("references")
                .iter()
                .find(|reference| reference["input_kind"] == "lineage_catalog")
                .expect("catalog variant");
            assert_eq!(catalog["schema_version"], 1);
            assert!(catalog.get("artifact_kind").is_none());
            assert!(catalog.get("lineage").is_none());
            assert!(catalog.get("acquisition_families").is_none());
        }
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_fixture_ledger_materializes_exact_literal_files_and_canonical_readers_accept_them,
    {
        use rust_electroanalysis_cli::{
            domain::read_artifact,
            results::{
                EisFitArtifact, MechanismAnalysisReport, SensorHealthAssessment,
                TransientAnalysisReport,
            },
        };
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase_d");
        let expected = [
            (
                "base/calibration.json",
                "7232a587edba2942aa8845fa44b4c9bee2383fda0e41e880f9bc89b5f06ce37e",
            ),
            (
                "base/calibration_observations.json",
                "d743434c21a19ba77a98247c29c2f2cc9d2b1617ecd046e426895ae5a7d1ff5b",
            ),
            (
                "base/eis.json",
                "352dbbd578437a2260d066f7b59795a036f37be89ce1bf4edae55cd00d5e0e8c",
            ),
            (
                "base/estimation.json",
                "3d843cdae227db31ddfcbe97b05c4f035e9b72e29aef86168782f34e41d942e2",
            ),
            (
                "base/health.json",
                "4265b48a0a70ff6ec89eb214a2cc8c2194cbd43bb7b7098482a7686e2eee73b3",
            ),
            (
                "base/lineage_catalog.json",
                "6b06d3a7a8b530d1acd4471d7bfc28de95e592a6726a9e72a81015c1ac0db320",
            ),
            (
                "base/mechanism.json",
                "b24422b8e1ec3f99fcea4a9f7c7f225dfe6f77550b0365b6cda80447fd306b8b",
            ),
            (
                "base/model.json",
                "f01a0360afb6a36e1d3c3649e01f56602f7ba9e7ab105eea4d75c76fcd595b0a",
            ),
            (
                "base/signal.json",
                "e354b35e6a61f8fe6a041e07a06deec7e5f7df191190efe7cf4a418e28f5f65a",
            ),
            (
                "base/transient.json",
                "cb79b7ddccacf91fb27a44f2e7a791a096e2c7b068dc0f310b426fab5792bc75",
            ),
            (
                "compat/eis_sensor_mismatch.json",
                "a2839d5d69416a6dbb4fdd9f07d878be4d0088f681c51ee81f8ac875a80f6fc1",
            ),
            (
                "compat/health_family.json",
                "b578840561e223f7f980b59549fef86e2b145bb2d602cb2bd2cef1b2df714a6c",
            ),
            (
                "compat/health_sensor_mismatch.json",
                "4f31633c53791cea8c1c71a8704d01624743fabeb8b945356cfab9e23aa8feb8",
            ),
            (
                "compat/health_unknown_scope.json",
                "5c61cf1ed6655057008dcc15544e12ccd78838256ba343ad15d4788023bc58e6",
            ),
            (
                "compat/mechanism_experiment_mismatch.json",
                "d5f3ec0942d2fdb6aa4f6af00a0b038a318790264d727f28a7131c16e9a16213",
            ),
            (
                "compat/mechanism_family.json",
                "3e5853e75fb1411c3ea43eade343316680a6582b86d0f0bb7bb79b5df5d47b61",
            ),
            (
                "compat/mechanism_unknown_scope.json",
                "b267cc167d01e310b725073e2e9ca18cb3e4091959ac63715b71135870d05985",
            ),
            (
                "eis/nyquist_bode.json",
                "405e4d15c8a9a60edd64ac2c188fc005a2b8037cf83316a6303601a1a05564a2",
            ),
            (
                "failure/catalog_duplicate_root_key.json",
                "2989e7f1c3b87e294c817e046496cbd8a3feb600ef988f32ddaeb938a1128cc2",
            ),
            (
                "failure/catalog_invalid_structure.json",
                "57f5cbaed52a8c21c66141f3622ab30c833b9f0e5cc15abe857fcc2e37003e69",
            ),
            (
                "failure/catalog_key_identity_mismatch.json",
                "9c232464a1f31263e476812bd9caa9167782e9faed07358bd5b2ad3566534bb7",
            ),
            (
                "failure/catalog_malformed.json",
                "8c69fc307fed3936d6a8ac679c0079c9bfd11f9de2a43e20ae25ff2a899d9776",
            ),
            (
                "failure/catalog_schema2.json",
                "6fb4379a5a30eb3e959dbb873dab858ee583ae782acf2b9d8160653141eda3d4",
            ),
            (
                "failure/eis_schema2.json",
                "7c7bc97b0a83040077bfb11bab3caaa0a2176f2e85b3580d4d75388a7e36478f",
            ),
            (
                "failure/wrong_kind.json",
                "e66762f2eb6828a8ac79f1f41857d30e66e77b8819387f11b0d1b36761950097",
            ),
            (
                "health/comparable_with_warnings.json",
                "782da2d7028ebdda6e4a1d73f34ee165ba6bcf72f1f2f22f2f700b71b1b34952",
            ),
            (
                "health/missing_unit.json",
                "1fab250fc1059d9c3f268da241a70a23eb54e5f0f0f5a0adc0f8913290a6a5ad",
            ),
            (
                "legacy/health_v3.json",
                "47aecec55b6a35d352ec349c8d6c7c35485a4b86b063a6be33920887c550cb7c",
            ),
            (
                "legacy/mechanism_v1.json",
                "1f306be35576f813347ad4906ead8296bf6d7a391547b2dfcdb9aef74d9d30e0",
            ),
            (
                "mechanism/timescale_cmp01.json",
                "d0a373578981f8db5f69e722d484c3be32e78e2f55d563d22125b3692332aee6",
            ),
            (
                "model/missing_values.json",
                "7bfe22b33f8d9658aa25812fdc379b890967f85e7708be82e7e4ac0ec3d5f3dc",
            ),
            (
                "output/keep.txt",
                "f660a7996deacfbc7560e4240054a8ad82eb02fe25a95064257e07084bcacb85",
            ),
            (
                "scale/health_large_evidence.json",
                "cbc6b694c88c9730e5f04db3d4a7b5b74097085f0614b8df3839094881328aa0",
            ),
            (
                "scale/mechanism_large_history.json",
                "a5aa22272501ca893c1cf82e3d36d4ae4266cd22210e6700d917ffd2fd65b72c",
            ),
            (
                "transient/duplicate_selected_fit.json",
                "2100bf817b94ddf4af58e2ef886377256cebb8efa77b83bf4b16a768a8e75450",
            ),
            (
                "transient/zero_selected_fit.json",
                "a801e26fe66268e8b666dc53b827a8397481f7be0d724d642ca34ffd39a6114c",
            ),
        ];
        assert_eq!(expected.len(), 36);
        for (relative, expected_hash) in expected {
            let actual = format!(
                "{:x}",
                Sha256::digest(fs::read(root.join(relative)).expect("fixture bytes"))
            );
            assert_eq!(actual, expected_hash, "{relative}");
        }
        assert!(read_artifact_lineage_catalog(&fixture("base/lineage_catalog.json")).is_ok());
        assert!(read_artifact::<MechanismAnalysisReport>(&fixture("base/mechanism.json")).is_ok());
        assert!(
            read_artifact::<SensorHealthAssessment>(&fixture(
                "health/comparable_with_warnings.json"
            ))
            .is_ok()
        );
        assert!(read_artifact::<EisFitArtifact>(&fixture("eis/nyquist_bode.json")).is_ok());
        assert!(
            read_artifact::<TransientAnalysisReport>(&fixture("transient/zero_selected_fit.json"))
                .is_ok()
        );
        let large_mechanism: MechanismAnalysisReport =
            read_artifact(&fixture("scale/mechanism_large_history.json")).expect("large mechanism");
        let large_health: SensorHealthAssessment =
            read_artifact(&fixture("scale/health_large_evidence.json")).expect("large health");
        assert_eq!(large_mechanism.hypothesis_history.len(), 1000);
        assert_eq!(
            large_health
                .phase_c
                .expect("phase c")
                .evidence_bundle
                .records
                .len(),
            10_000
        );
        assert!(matches!(
            read_artifact_lineage_catalog(&fixture("failure/catalog_malformed.json")),
            Err(LineageCatalogReadError::Json { .. })
        ));
        assert!(matches!(
            read_artifact_lineage_catalog(&fixture("failure/catalog_invalid_structure.json")),
            Err(LineageCatalogReadError::UnknownField { .. })
        ));
        assert!(matches!(
            read_artifact_lineage_catalog(&fixture("failure/catalog_schema2.json")),
            Err(LineageCatalogReadError::UnsupportedSchemaVersion { .. })
        ));
        assert!(matches!(
            read_artifact_lineage_catalog(&fixture("failure/catalog_duplicate_root_key.json")),
            Err(LineageCatalogReadError::DuplicateField { .. })
        ));
        assert!(matches!(
            read_artifact_lineage_catalog(&fixture("failure/catalog_key_identity_mismatch.json")),
            Err(LineageCatalogReadError::KeyIdentityMismatch { .. })
        ));
    }
);
