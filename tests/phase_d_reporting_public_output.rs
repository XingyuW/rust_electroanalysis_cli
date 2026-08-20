use rust_electroanalysis_cli::{
    cli::{CliError, parse_cli_args},
    domain::{LineageCatalogReadError, read_artifact_lineage_catalog},
    report_config::{ReportFormat, ReportRenderOptions, ReportSelection},
    reporting::{PublicReportError, format_public_f64},
    runners::RunnerError,
};

mod report {
    use rust_electroanalysis_cli::{
        report_config::ReportRenderOptions,
        reporting::PublicReportError,
        runners::report::{ReportRenderOutcome, run},
    };

    pub fn render(options: &ReportRenderOptions) -> Result<ReportRenderOutcome, PublicReportError> {
        run(options)
    }
}
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
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

fn full_options(label: &str) -> ReportRenderOptions {
    let mut options = compact_options(label);
    options.eis = Some(fixture("base/eis.json"));
    options.transient = Some(fixture("base/transient.json"));
    options.calibration = Some(fixture("base/calibration.json"));
    options.calibration_observations = Some(fixture("base/calibration_observations.json"));
    options.signal = Some(fixture("base/signal.json"));
    options.estimation = Some(fixture("base/estimation.json"));
    options.model = Some(fixture("base/model.json"));
    options.selection = ReportSelection::parse(None, None).expect("default selection");
    options
}

fn render(options: &ReportRenderOptions) -> PathBuf {
    let root = options.output_dir.clone();
    report::render(options).expect("certified render succeeds");
    assert_certified_bundle_integrity(&root);
    root
}

fn read(root: &Path, relative: &str) -> String {
    fs::read_to_string(root.join(relative)).expect("certified text output")
}

fn cleanup(root: &Path) {
    let parent = root.parent().map(Path::to_path_buf);
    let _ = fs::remove_dir_all(root);
    if let (Some(parent), Some(name)) = (parent, root.file_name().and_then(|name| name.to_str()))
        && let Ok(entries) = fs::read_dir(parent)
    {
        for entry in entries.flatten() {
            let candidate = entry.path();
            if candidate
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| {
                    value.starts_with(&format!(".{name}.phase-d-staging-"))
                        || value.starts_with(&format!(".{name}.phase-d-backup-"))
                })
            {
                let _ = fs::remove_dir_all(candidate);
            }
        }
    }
}

fn json(root: &Path, relative: &str) -> serde_json::Value {
    serde_json::from_str(&read(root, relative)).expect("closed JSON output")
}

fn exact_keys(value: &serde_json::Value, expected: &[&str]) {
    let actual = value
        .as_object()
        .expect("JSON object")
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

fn relative_files(root: &Path) -> Vec<String> {
    fn visit(root: &Path, directory: &Path, files: &mut Vec<String>) {
        for entry in fs::read_dir(directory).expect("bundle directory") {
            let entry = entry.expect("bundle entry");
            let path = entry.path();
            assert!(!entry.file_type().expect("entry type").is_symlink());
            if path.is_dir() {
                visit(root, &path, files);
            } else {
                files.push(
                    path.strip_prefix(root)
                        .expect("relative path")
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    let mut files = Vec::new();
    visit(root, root, &mut files);
    files.sort();
    files
}

fn generated_paths(manifest: &serde_json::Value) -> Vec<String> {
    manifest["generated_files"]
        .as_array()
        .expect("generated files")
        .iter()
        .map(|file| {
            assert_eq!(file["status"], "written");
            file["relative_path"]
                .as_str()
                .expect("relative path")
                .to_owned()
        })
        .collect()
}

fn assert_certified_bundle_integrity(root: &Path) {
    assert!(root.is_dir());
    let manifest = json(root, "render_manifest.schema1.json");
    exact_keys(
        &manifest,
        &[
            "schema_version",
            "output_kind",
            "renderer_contract",
            "route",
            "final_output_status",
            "input_references",
            "requested",
            "render_order",
            "generated_files",
            "unavailable_outputs",
            "warnings",
            "legacy_input_notices",
            "optional_compatibility",
            "determinism",
        ],
    );
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["output_kind"], "phase_d_render_manifest");
    assert_eq!(
        manifest["renderer_contract"],
        "mhi_v1_phase_d_public_output_v1"
    );
    assert_eq!(manifest["route"], "electroanalysis report render");
    assert_eq!(manifest["final_output_status"], "published");
    let paths = generated_paths(&manifest);
    assert_eq!(
        paths.last().map(String::as_str),
        Some("render_manifest.schema1.json")
    );
    let render_order = manifest["render_order"].as_array().expect("render order");
    assert_eq!(render_order.len(), paths.len());
    for (ordinal, (step, path)) in render_order.iter().zip(&paths).enumerate() {
        assert_eq!(step["ordinal"], ordinal as u64);
        assert_eq!(step["relative_path"], *path);
    }
    let mut declared = paths;
    declared.sort();
    assert_eq!(relative_files(root), declared);
    if root.join("public_summary.schema1.json").is_file() {
        let summary = json(root, "public_summary.schema1.json");
        exact_keys(
            &summary,
            &[
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
            ],
        );
        assert_eq!(summary["schema_version"], 1);
        assert_eq!(summary["output_kind"], "phase_d_public_scientific_output");
    }
    if root.join("scientific_report.md").is_file() {
        let report = read(root, "scientific_report.md");
        assert!(report.starts_with("# Analysis identity and renderer boundary\n"));
        assert!(report.contains("This report projects serialized assessments."));
        assert!(report.contains("do not by themselves establish causal proof."));
    }
}

fn csv_records(root: &Path, relative: &str) -> Vec<Vec<String>> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(root.join(relative))
        .expect("CSV output")
        .records()
        .map(|row| {
            row.expect("CSV row")
                .iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

fn cli_output(arguments: &[String], environment: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rust_electroanalysis_cli"));
    command.args(arguments);
    for (name, value) in environment {
        command.env(name, value);
    }
    command.output().expect("certified CLI process")
}

fn full_cli_arguments(output: &Path) -> Vec<String> {
    let mut arguments = vec!["report".into(), "render".into()];
    for (flag, path) in [
        ("--mechanism", fixture("base/mechanism.json")),
        ("--health", fixture("base/health.json")),
        ("--lineage-catalog", fixture("base/lineage_catalog.json")),
        ("--eis", fixture("base/eis.json")),
        ("--transient", fixture("base/transient.json")),
        ("--calibration", fixture("base/calibration.json")),
        (
            "--calibration-observations",
            fixture("base/calibration_observations.json"),
        ),
        ("--signal", fixture("base/signal.json")),
        ("--estimation", fixture("base/estimation.json")),
        ("--model", fixture("base/model.json")),
    ] {
        arguments.push(flag.into());
        arguments.push(path.to_string_lossy().into_owned());
    }
    arguments.push("--output-dir".into());
    arguments.push(output.to_string_lossy().into_owned());
    arguments
}

fn compact_cli_arguments(output: &Path, overwrite: bool) -> Vec<String> {
    let mut arguments = vec![
        "report".into(),
        "render".into(),
        "--mechanism".into(),
        fixture("base/mechanism.json")
            .to_string_lossy()
            .into_owned(),
        "--health".into(),
        fixture("base/health.json").to_string_lossy().into_owned(),
        "--lineage-catalog".into(),
        fixture("base/lineage_catalog.json")
            .to_string_lossy()
            .into_owned(),
        "--output-dir".into(),
        output.to_string_lossy().into_owned(),
        "--figures".into(),
        "none".into(),
        "--tables".into(),
        "none".into(),
    ];
    if overwrite {
        arguments.push("--overwrite".into());
    }
    arguments
}

fn bundle_bytes(root: &Path) -> BTreeMap<String, Vec<u8>> {
    relative_files(root)
        .into_iter()
        .map(|relative| {
            let bytes = fs::read(root.join(&relative)).expect("bundle bytes");
            (relative, bytes)
        })
        .collect()
}

fn publication_siblings(root: &Path) -> Vec<PathBuf> {
    let Some(parent) = root.parent() else {
        return Vec::new();
    };
    let Some(name) = root.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    let mut siblings = fs::read_dir(parent)
        .expect("output parent")
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| {
                    value.starts_with(&format!(".{name}.phase-d-staging-"))
                        || value.starts_with(&format!(".{name}.phase-d-backup-"))
                })
        })
        .collect::<Vec<_>>();
    siblings.sort();
    siblings
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
fn phase_d_cli_requires_mechanism_and_health() {
    for (missing, present_flag, present_value) in [
        ("mechanism", "--health", "health.json"),
        ("health", "--mechanism", "mechanism.json"),
    ] {
        let output = temporary_root(&format!("missing-{missing}"));
        let arguments = vec![
            "electroanalysis".into(),
            "report".into(),
            "render".into(),
            present_flag.into(),
            present_value.into(),
            "--output-dir".into(),
            output.to_string_lossy().into_owned(),
        ];
        let error = parse_cli_args(&arguments).expect_err("required flag");
        match error {
            CliError::Parse(message) => {
                assert!(message.to_string().contains(&format!("--{missing}")))
            }
            other => panic!("expected parser error, got {other:?}"),
        }
        assert!(!output.exists());
    }
}

#[test]
fn phase_d_clap_rejects_unknown_format_before_runner() {
    let output = temporary_root("unknown-format");
    let arguments = vec![
        "electroanalysis".into(),
        "report".into(),
        "render".into(),
        "--mechanism".into(),
        "mechanism.json".into(),
        "--health".into(),
        "health.json".into(),
        "--output-dir".into(),
        output.to_string_lossy().into_owned(),
        "--format".into(),
        "yaml".into(),
    ];
    match parse_cli_args(&arguments).expect_err("closed format") {
        CliError::Parse(message) => {
            let message = message.to_string();
            assert!(message.contains("invalid value 'yaml'"));
            assert!(message.contains("all"));
            assert!(message.contains("json"));
            assert!(message.contains("markdown"));
        }
        other => panic!("expected parser error, got {other:?}"),
    }
    assert!(!output.exists());
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
    for calibration_only in [true, false] {
        let mut options = compact_options(if calibration_only {
            "calibration-only"
        } else {
            "observations-only"
        });
        if calibration_only {
            options.calibration = Some(fixture("base/calibration.json"));
        } else {
            options.calibration_observations = Some(fixture("base/calibration_observations.json"));
        }
        assert!(matches!(
            report::render(&options),
            Err(PublicReportError::InvalidCombination {
                detail: "--calibration and --calibration-observations must be supplied together"
            })
        ));
        assert!(!options.output_dir.exists());
    }
});

phase_d_test!(phase_d_cli_rejects_unknown_selection, {
    for (figures, tables, selector) in [
        (Some("unknown"), None, "figures"),
        (None, Some("unknown"), "tables"),
    ] {
        assert!(matches!(
            ReportSelection::parse(figures, tables),
            Err(PublicReportError::InvalidSelection { selector: actual, ref value })
                if actual == selector && value == "unknown"
        ));
    }
});

phase_d_test!(phase_d_cli_rejects_duplicate_selection, {
    for (figures, tables, selector, value) in [
        (Some("lineage,lineage"), None, "figures", "lineage"),
        (
            None,
            Some("artifact_lineage,artifact_lineage"),
            "tables",
            "artifact_lineage",
        ),
    ] {
        assert!(matches!(
            ReportSelection::parse(figures, tables),
            Err(PublicReportError::InvalidSelection { selector: actual, value: actual_value })
                if actual == selector && actual_value == value
        ));
    }
});

phase_d_test!(phase_d_cli_rejects_existing_output_without_overwrite, {
    let options = compact_options("collision");
    fs::create_dir(&options.output_dir).expect("output root");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::OutputCollision { path }) if path == options.output_dir
    ));
    assert!(relative_files(&options.output_dir).is_empty());
    cleanup(&options.output_dir);
});

phase_d_test!(phase_d_cli_overwrite_rejects_unmanaged_entry, {
    let mut options = compact_options("unmanaged");
    fs::create_dir(&options.output_dir).expect("output root");
    fs::write(options.output_dir.join("keep.txt"), "do not delete").expect("sentinel");
    options.overwrite = true;
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::UnmanagedOutputEntry { path })
            if path == options.output_dir.join("render_manifest.schema1.json")
    ));
    assert_eq!(
        fs::read_to_string(options.output_dir.join("keep.txt")).expect("sentinel retained"),
        "do not delete"
    );
    cleanup(&options.output_dir);

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;

        let mut symlink_options = compact_options("managed-symlink");
        let root = render(&symlink_options);
        let manifest_before = fs::read(root.join("render_manifest.schema1.json")).unwrap();
        let symlink_path = root.join("figures/unmanaged-link.svg");
        fs::create_dir(root.join("figures")).expect("figure directory");
        symlink(fixture("base/mechanism.json"), &symlink_path).expect("unmanaged symlink");
        symlink_options.overwrite = true;
        assert!(matches!(
            report::render(&symlink_options),
            Err(PublicReportError::UnmanagedOutputEntry { path }) if path == symlink_path
        ));
        assert_eq!(
            fs::read(root.join("render_manifest.schema1.json")).unwrap(),
            manifest_before
        );
        fs::remove_file(symlink_path).expect("remove test symlink");
        cleanup(&root);

        let root_link = temporary_root("output-root-symlink");
        symlink(fixture("base"), &root_link).expect("output-root symlink");
        let mut root_link_options = compact_options("unused-root-link-options");
        root_link_options.output_dir = root_link.clone();
        root_link_options.overwrite = true;
        assert!(matches!(
            report::render(&root_link_options),
            Err(PublicReportError::InvalidOutputDirectory { path }) if path == root_link
        ));
        fs::remove_file(root_link).expect("remove output-root symlink");
    }
});

phase_d_test!(phase_d_reads_only_canonical_artifacts, {
    let mut options = compact_options("wrong-kind");
    options.mechanism = fixture("failure/wrong_kind.json");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::Artifact {
            flag: "--mechanism",
            path,
            source: rust_electroanalysis_cli::domain::ArtifactError::IncompatibleKind {
                expected: rust_electroanalysis_cli::domain::ArtifactKind::MechanismAnalysis,
                ..
            },
        }) if path == fixture("failure/wrong_kind.json")
    ));
    assert!(!options.output_dir.exists());
});

phase_d_test!(phase_d_rejects_unsupported_optional_schema, {
    let mut options = compact_options("optional-schema");
    options.eis = Some(fixture("failure/eis_schema2.json"));
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::Artifact {
            flag: "--eis",
            path,
            source: rust_electroanalysis_cli::domain::ArtifactError::UnsupportedSchemaVersion {
                actual: 2,
                expected: rust_electroanalysis_cli::domain::ArtifactKind::EisFit,
                ..
            },
        }) if path == fixture("failure/eis_schema2.json")
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
        assert_eq!(
            keys,
            [
                "sha256:12b73e011b71dfe35bf5e6d88ba15ecf4767a7fc1e2c95820602e6c120dc5ddf",
                "sha256:325483a1050eb603dd7b15c9587cfae97fa41aaf29a393a71c6082725b028e44",
                "sha256:4717ab60c11af2a14fb665ff07427530861d2eb52773b288a733b9e814562964",
                "sha256:927c0d3e846978f80e964fb040bfcca3e15cfffaf79bd712e223b6cf6d71c4f3",
                "sha256:a9e888019fd01dee61c98390a27bd9c6ca80eafe6b6379b77ca41f6a42a8c5b0",
                "sha256:d9465a5deff1224c5190dae21a674c34e9eb293f88055973491616ea2ba02b5c",
            ]
        );
        let mut options = compact_options("catalog-order");
        explicit(&mut options, "none", "artifact_lineage");
        let root = render(&options);
        let rows = csv_records(&root, "tables/artifact_lineage.csv");
        assert_eq!(rows.len(), 7);
        assert_eq!(rows[1][0], "mechanism");
        assert_eq!(rows[1][1], "root");
        assert_eq!(rows[2][1], "direct_dependency");
        assert_eq!(rows[2][5], "TransformationInput");
        assert_eq!(rows[2][6], "calibration_observations");
        assert_eq!(rows[3][6], "eis_fit");
        assert_eq!(rows[4][6], "state_estimation");
        assert_eq!(rows[5][6], "transient_analysis");
        assert_eq!(rows[6][0], "health");
        assert_eq!(rows[6][1], "root");
        cleanup(&root);
    }
);

phase_d_test!(phase_d_catalog_reader_rejects_schema2, {
    let path = fixture("failure/catalog_schema2.json");
    assert!(matches!(
        read_artifact_lineage_catalog(&path),
        Err(LineageCatalogReadError::UnsupportedSchemaVersion { path: actual, actual: 2 })
            if actual == path
    ));
});

phase_d_test!(phase_d_catalog_reader_rejects_key_identity_mismatch, {
    let path = fixture("failure/catalog_key_identity_mismatch.json");
    assert!(matches!(
        read_artifact_lineage_catalog(&path),
        Err(LineageCatalogReadError::KeyIdentityMismatch {
            path: actual,
            key,
            identity,
        }) if actual == path
            && key == "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            && identity == "sha256:12b73e011b71dfe35bf5e6d88ba15ecf4767a7fc1e2c95820602e6c120dc5ddf"
    ));
});

phase_d_test!(phase_d_catalog_reader_rejects_duplicate_json_key, {
    let path = fixture("failure/catalog_duplicate_root_key.json");
    assert_eq!(
        fs::read(&path).expect("literal duplicate-key bytes"),
        b"{\"schema_version\":1,\"schema_version\":1,\"artifacts\":{}}\n"
    );
    assert!(matches!(
        read_artifact_lineage_catalog(&path),
        Err(LineageCatalogReadError::DuplicateField { path: actual, field })
            if actual == path && field == "schema_version"
    ));
});

phase_d_test!(phase_d_reporting_never_ad_hoc_parses_catalog, {
    let source = include_str!("../src/reporting/reader.rs");
    assert!(!source.contains("serde_json::from_"));
    assert_eq!(source.matches("read_artifact::<").count(), 3);
    assert_eq!(source.matches("read_optional::<").count(), 7);
    assert_eq!(source.matches("domain::read_artifact(path)").count(), 1);
    assert_eq!(
        source
            .matches("domain::read_artifact_lineage_catalog")
            .count(),
        1
    );
    assert_eq!(source.matches("scope_compatible(").count(), 2);
    assert!(!source.contains("fn scope_compatible"));
    let runner = include_str!("../src/runners/report.rs");
    assert!(runner.contains("pub(crate) fn render_public_report("));
    assert!(!runner.contains("pub fn render_public_report("));
    assert!(runner.contains("pub fn run(options: &ReportRenderOptions)"));
    let facade = include_str!("../src/reporting/mod.rs");
    assert!(facade.contains("pub(crate) use crate::runners::report::render_public_report;"));
});

phase_d_test!(phase_d_required_known_scope_mismatch_is_rejected, {
    let mut options = compact_options("sensor-mismatch");
    options.health = fixture("compat/health_sensor_mismatch.json");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::RequiredInputsIncompatible {
            left_flag: "--mechanism",
            right_flag: "--health",
            axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::SensorScope,
            left,
            right,
        }) if left == "Unspecified" && right == "Specific(\"sensor-mismatch\")"
    ));
    assert!(!options.output_dir.exists());
});

phase_d_test!(phase_d_required_experiment_mismatch_is_rejected, {
    let mut options = compact_options("experiment-mismatch");
    options.mechanism = fixture("compat/mechanism_experiment_mismatch.json");
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::RequiredInputsIncompatible {
            left_flag: "--mechanism",
            right_flag: "--health",
            axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::ExperimentScope,
            left,
            right,
        }) if left == "Single { experiment_id: ExperimentId(\"experiment-mismatch\") }"
            && right == "Single { experiment_id: ExperimentId(\"b-e2e-1\") }"
    ));
    assert!(!options.output_dir.exists());
});

phase_d_test!(
    phase_d_required_equal_unknown_scope_reuses_phase_c_admissibility,
    {
        let mut options = compact_options("unknown-scope");
        options.mechanism = fixture("compat/mechanism_unknown_scope.json");
        options.health = fixture("compat/health_unknown_scope.json");
        let root = render(&options);
        let summary = json(&root, "public_summary.schema1.json");
        assert_eq!(summary["compatibility"]["required_pair"], "compatible");
        let references = summary["input_references"].as_array().expect("references");
        for flag in ["mechanism", "health"] {
            let reference = references
                .iter()
                .find(|reference| reference["input_flag"] == flag)
                .expect("required reference");
            assert_eq!(reference["lineage"]["status"], "known");
            assert_eq!(
                reference["lineage"]["identity"]["sensor_scope"]["kind"],
                "unspecified"
            );
            assert_eq!(
                reference["lineage"]["identity"]["channel_scope"]["kind"],
                "unspecified"
            );
        }
        cleanup(&root);
    }
);

phase_d_test!(phase_d_required_legacy_unknown_is_explicit, {
    let mut options = compact_options("legacy-required");
    options.mechanism = fixture("legacy/mechanism_v1.json");
    options.health = fixture("legacy/health_v3.json");
    let root = render(&options);
    let summary = json(&root, "public_summary.schema1.json");
    assert_eq!(summary["compatibility"]["required_pair"], "legacy_unknown");
    assert_eq!(summary["mechanism"]["availability"], "unavailable");
    assert_eq!(summary["sensor_health"]["availability"], "unavailable");
    assert!(
        summary["mechanism"]["hypotheses"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        summary["sensor_health"]["dimensions"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let limitation_codes = summary["limitations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(
        limitation_codes
            .iter()
            .filter(|code| **code == "legacy_input")
            .count()
            >= 2
    );
    cleanup(&root);
});

phase_d_test!(
    phase_d_optional_known_mismatch_is_rejected_when_unselected,
    {
        let mut options = compact_options("optional-mismatch");
        options.eis = Some(fixture("compat/eis_sensor_mismatch.json"));
        assert!(matches!(
            report::render(&options),
            Err(PublicReportError::OptionalInputIncompatible {
                flag: "--eis",
                required_flag: "--mechanism",
                axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::SensorScope,
                actual,
                expected,
        }) if actual == "Specific(\"sensor-mismatch\")" && expected == "Unspecified"
        ));
        assert!(!options.output_dir.exists());

        let output = temporary_root("transient-zero-explicit-preflight");
        let mut arguments = compact_cli_arguments(&output, false);
        let figures_index = arguments
            .iter()
            .position(|argument| argument == "--figures")
            .unwrap()
            + 1;
        arguments[figures_index] = "transient_response".into();
        arguments.push("--transient".into());
        arguments.push(
            fixture("transient/zero_selected_fit.json")
                .to_string_lossy()
                .into_owned(),
        );
        let result = cli_output(
            &arguments,
            &[(
                "ELECTROANALYSIS_PHASE_D_TEST_FAIL_CLEANUP",
                Path::new("staging"),
            )],
        );
        assert!(!result.status.success());
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(stderr.contains("SelectedFitNotFound"));
        assert!(!stderr.contains("could not clean up"));
        assert!(!output.exists());
        assert!(publication_siblings(&output).is_empty());
    }
);

phase_d_test!(phase_d_optional_legacy_unknown_is_limited_not_inferred, {
    let mut options = compact_options("optional-legacy");
    options.model = Some(fixture("base/model.json"));
    let root = render(&options);
    let summary = json(&root, "public_summary.schema1.json");
    let model = summary["optional_sources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|source| source["kind"] == "model")
        .expect("model source");
    assert_eq!(model["availability"], "not_selected");
    assert_eq!(model["compatibility"], "legacy_unknown");
    assert_eq!(model["input"]["lineage"]["status"], "legacy_unknown");
    assert_eq!(
        model["input"]["acquisition_families"]["status"],
        "legacy_unknown"
    );
    assert_eq!(
        model["input"]["acquisition_families"]["values"],
        serde_json::json!([])
    );
    assert_eq!(model["detail"], serde_json::Value::Null);
    assert!(!read(&root, "public_summary.schema1.json").contains("independent"));
    cleanup(&root);
});

phase_d_test!(phase_d_schema4_health_projects_exactly_nine_dimensions, {
    let root = render(&compact_options("health-nine"));
    let value: serde_json::Value =
        serde_json::from_str(&read(&root, "public_summary.schema1.json")).expect("summary json");
    let dimensions = value["sensor_health"]["dimensions"]
        .as_array()
        .expect("dimensions");
    assert_eq!(dimensions.len(), 9);
    assert_eq!(
        dimensions
            .iter()
            .map(|row| row["dimension"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "signal_integrity",
            "calibration_health",
            "dynamic_response_health",
            "reference_stability",
            "environmental_robustness",
            "model_consistency",
            "observability",
            "uncertainty_health",
            "data_quality",
        ]
    );
    assert_eq!(dimensions[0]["status"], "critical");
    assert_eq!(dimensions[8]["status"], "data_quality_insufficient");
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
            .unwrap()
            .is_empty()
    );
    assert_eq!(value["sensor_health"]["availability"], "unavailable");
    assert_eq!(
        value["sensor_health"]["assessment_id"],
        "phase-c-legacy-schema3-fixture"
    );
    assert!(value["limitations"].as_array().unwrap().iter().any(|item| {
        item["code"] == "legacy_input"
            && item["input_flag"] == "health"
            && item["message"]
                == "Legacy schema-3 health assessment: Phase C nine-dimension assessment was not serialized."
    }));
    cleanup(&root);
});

phase_d_test!(
    phase_d_legacy_mechanism_marks_phase_b_assessment_unavailable,
    {
        let mut options = compact_options("legacy-mechanism");
        options.mechanism = fixture("legacy/mechanism_v1.json");
        let root = render(&options);
        let summary = json(&root, "public_summary.schema1.json");
        assert_eq!(summary["mechanism"]["availability"], "unavailable");
        assert_eq!(summary["mechanism"]["analysis_id"], "mechanism:a0:R0");
        assert!(
            summary["mechanism"]["hypotheses"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert!(summary["limitations"].as_array().unwrap().iter().any(|item| {
            item["code"] == "legacy_input"
                && item["input_flag"] == "mechanism"
                && item["message"] == "Legacy mechanism artifact; Phase B V1 hypothesis assessment unavailable."
        }));
        cleanup(&root);
    }
);

phase_d_test!(phase_d_public_summary_schema1_is_closed_and_ordered, {
    let root = render(&compact_options("summary-order"));
    let text = read(&root, "public_summary.schema1.json");
    let keys = [
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
    ];
    let summary = json(&root, "public_summary.schema1.json");
    exact_keys(&summary, &keys);
    let positions = keys
        .iter()
        .map(|key| {
            text.find(&format!("\n  \"{key}\":"))
                .expect("top-level declared key")
        })
        .collect::<Vec<_>>();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    exact_keys(
        &summary["mechanism"],
        &[
            "availability",
            "analysis_id",
            "hypotheses",
            "comparisons",
            "warning_messages",
            "lineage",
            "provenance",
        ],
    );
    exact_keys(
        &summary["sensor_health"],
        &[
            "availability",
            "assessment_id",
            "sensor_id",
            "experiment_id",
            "overall_status",
            "dimensions",
            "features",
            "baseline_comparisons",
            "warning_codes",
            "lineage",
            "provenance",
        ],
    );
    assert_eq!(summary["input_references"].as_array().unwrap().len(), 10);
    assert_eq!(summary["optional_sources"].as_array().unwrap().len(), 7);
    cleanup(&root);
});

phase_d_test!(phase_d_public_summary_field_authorities_are_typed_copies, {
    let root = render(&compact_options("typed-copy"));
    let summary = json(&root, "public_summary.schema1.json");
    let health_fixture: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture("base/health.json")).unwrap()).unwrap();
    let mechanism_fixture: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture("base/mechanism.json")).unwrap()).unwrap();
    assert_eq!(
        summary["sensor_health"]["assessment_id"],
        health_fixture["assessment_id"]
    );
    assert_eq!(
        summary["sensor_health"]["overall_status"],
        health_fixture["overall_status"]
    );
    assert_eq!(
        summary["mechanism"]["analysis_id"],
        mechanism_fixture["analysis_id"]
    );
    assert_eq!(
        summary["mechanism"]["hypotheses"][0]["hypothesis_id"],
        mechanism_fixture["hypothesis_assessments"][0]["definition"]["hypothesis_id"]
    );
    assert_eq!(
        summary["sensor_health"]["features"][0]["value"],
        health_fixture["features"]
            .as_array()
            .unwrap()
            .iter()
            .min_by_key(|feature| feature["name"].as_str().unwrap())
            .unwrap()["value"]
    );
    assert_eq!(
        summary["mechanism"]["provenance"]["software_version"],
        "phase-b-fixture-generator"
    );
    assert_eq!(
        summary["sensor_health"]["provenance"]["input_sha256"],
        "a0-test"
    );
    cleanup(&root);
});

phase_d_test!(phase_d_render_manifest_schema1_records_semantic_fields, {
    let root = render(&compact_options("manifest"));
    let manifest = json(&root, "render_manifest.schema1.json");
    assert_eq!(
        manifest["requested"]["formats"],
        serde_json::json!(["json", "markdown"])
    );
    assert_eq!(manifest["requested"]["figures"], serde_json::json!([]));
    assert_eq!(manifest["requested"]["tables"], serde_json::json!([]));
    assert_eq!(manifest["requested"]["figures_mode"], "explicit");
    assert_eq!(manifest["requested"]["tables_mode"], "explicit");
    assert_eq!(
        generated_paths(&manifest),
        [
            "public_summary.schema1.json",
            "scientific_report.md",
            "render_manifest.schema1.json",
        ]
    );
    assert!(
        manifest["unavailable_outputs"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(manifest["input_references"].as_array().unwrap().len(), 10);
    assert_eq!(
        manifest["determinism"]["json_object_order"],
        "declaration_order"
    );
    assert_eq!(manifest["determinism"]["array_order"], "contract_order");
    assert_eq!(
        manifest["determinism"]["numeric_format"],
        "rust_display_normalized_negative_zero_v1"
    );
    assert_eq!(manifest["determinism"]["clock"], serde_json::Value::Null);
    cleanup(&root);
});

phase_d_test!(phase_d_render_manifest_orders_paths_and_legacy_notices, {
    let mut options = compact_options("legacy-manifest");
    options.mechanism = fixture("legacy/mechanism_v1.json");
    options.health = fixture("legacy/health_v3.json");
    let root = render(&options);
    let manifest = json(&root, "render_manifest.schema1.json");
    assert_eq!(manifest["determinism"]["path_separator"], "/");
    assert_eq!(
        manifest["legacy_input_notices"],
        serde_json::json!([
            {
                "input_flag": "mechanism",
                "schema_version": 1,
                "notice": "legacy_mechanism_assessment_not_serialized"
            },
            {
                "input_flag": "health",
                "schema_version": 3,
                "notice": "legacy_phase_c_not_serialized"
            },
            {
                "input_flag": "mechanism",
                "schema_version": 1,
                "notice": "legacy_lineage_unknown"
            },
            {
                "input_flag": "health",
                "schema_version": 3,
                "notice": "legacy_lineage_unknown"
            }
        ])
    );
    for file in manifest["generated_files"].as_array().unwrap() {
        let path = file["relative_path"].as_str().unwrap();
        assert!(!path.contains('\\'));
        assert!(!path.starts_with('/'));
        assert!(!path.contains(".."));
    }
    cleanup(&root);
});

phase_d_test!(phase_d_markdown_sections_and_order_are_stable, {
    let root = render(&full_options("markdown-sections"));
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
        .map(|heading| report.find(&format!("# {heading}\n")).expect("section"))
        .collect::<Vec<_>>();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    for exact in [
        "Mechanism analysis: `mechanism-phase-b:b-e2e-1` (schema 4)",
        "Health assessment: `health:signal:a0-test:E1` (schema 4)",
        "## b-hypothesis — B E2E validated for domain",
        "| data_quality | Data quality insufficient (DQI) | poor_data_quality",
        "| observability | Indeterminate | no_evidence",
        "signal.descriptive.rms;signal.descriptive.robust_standard_deviation",
        "`tables/health_dimensions.csv`",
        "`eis_nyquist`: serialized-source SVG and PNG",
        "LegacyUnknown (source schema `5`, reason `FieldAbsentInLegacyArtifact`)",
        "Numeric format: `rust_display_normalized_negative_zero_v1`",
    ] {
        assert!(
            report.contains(exact),
            "missing exact Markdown evidence: {exact}"
        );
    }
    assert!(
        report
            .find("do not by themselves establish causal proof.")
            .unwrap()
            < positions[1]
    );
    cleanup(&root);
});

phase_d_test!(phase_d_mechanism_table_projects_serialized_gate_statuses, {
    let mut options = compact_options("mechanism-table");
    explicit(&mut options, "none", "mechanism_evidence");
    let root = render(&options);
    assert_eq!(
        csv_records(&root, "tables/mechanism_evidence.csv"),
        vec![
            vec![
                "hypothesis_id",
                "display_name",
                "evidence_level",
                "reason_codes",
                "validation_status",
                "temporal_statuses",
                "timescale_statuses",
                "amplitude_statuses",
                "repeatability_statuses",
                "identifiability_statuses",
                "contradiction_requirement_ids",
                "component_ids",
                "history_ids",
                "legacy_status"
            ],
            vec![
                "b-hypothesis",
                "B E2E validated for domain",
                "validated_for_domain",
                "validation_satisfied;timescale_satisfied;identifiability_satisfied",
                "satisfied",
                "[]",
                "satisfied",
                "[]",
                "[]",
                "satisfied",
                "[]",
                "b-eis-tau;b-validation-calibration;b-validation-estimation;tau_fast_s",
                "[]",
                "current"
            ],
        ]
        .into_iter()
        .map(|row| row.into_iter().map(str::to_owned).collect())
        .collect::<Vec<Vec<String>>>()
    );
    cleanup(&root);
});

phase_d_test!(phase_d_health_table_preserves_dqi_reason_codes, {
    let mut options = compact_options("dqi-table");
    explicit(&mut options, "none", "health_dimensions");
    let root = render(&options);
    let rows = csv_records(&root, "tables/health_dimensions.csv");
    assert_eq!(rows.len(), 10);
    assert_eq!(
        rows[0],
        [
            "dimension",
            "display_label",
            "status",
            "evidence_state",
            "reason_codes",
            "interpretation_category",
            "causal_status",
            "source_evidence_ids",
            "excluded_evidence_ids",
            "source_artifact_ids",
            "legacy_status"
        ]
    );
    assert_eq!(
        rows[9],
        [
            "data_quality",
            "Data quality",
            "data_quality_insufficient",
            "poor_data_quality",
            "quality_gate_failed",
            "observed_behavior",
            "indeterminate",
            "signal.sampling.duplicate_timestamps;signal.sampling.finite_sample_count;signal.sampling.interpolation_gap_exceeded;signal.sampling.interval_cv;signal.sampling.missing_fraction;signal.sampling.non_monotonic_timestamps",
            "[]",
            "[]",
            "current"
        ]
    );
    cleanup(&root);
});

phase_d_test!(phase_d_health_table_preserves_indeterminate_reason_codes, {
    let mut options = compact_options("indeterminate-table");
    explicit(&mut options, "none", "health_dimensions");
    let root = render(&options);
    let rows = csv_records(&root, "tables/health_dimensions.csv");
    assert_eq!(
        rows[7],
        [
            "observability",
            "Observability",
            "indeterminate",
            "no_evidence",
            "optional_source_absent",
            "model_inconsistency",
            "indeterminate",
            "[]",
            "[]",
            "[]",
            "current"
        ]
    );
    assert_eq!(
        rows.iter()
            .skip(1)
            .filter(|row| row[2] == "indeterminate")
            .count(),
        7
    );
    assert_eq!(
        rows.iter()
            .skip(1)
            .filter(|row| row[3] == "no_evidence")
            .count(),
        7
    );
    cleanup(&root);
});

phase_d_test!(phase_d_evidence_provenance_csv_is_deterministic, {
    let mut left = compact_options("evidence-left");
    explicit(&mut left, "none", "evidence_provenance");
    let mut right = compact_options("evidence-right");
    explicit(&mut right, "none", "evidence_provenance");
    let left_root = render(&left);
    let right_root = render(&right);
    let left_rows = csv_records(&left_root, "tables/evidence_provenance.csv");
    let right_rows = csv_records(&right_root, "tables/evidence_provenance.csv");
    assert_eq!(left_rows, right_rows);
    assert_eq!(left_rows.len(), 11);
    assert_eq!(
        left_rows[0],
        [
            "assessment_scope",
            "evidence_id",
            "target",
            "source_class",
            "direction",
            "availability",
            "validity",
            "quantity_value",
            "quantity_unit",
            "uncertainty",
            "source_artifact_kind",
            "source_artifact_id_or_fingerprint",
            "source_field_path",
            "experiment_scope",
            "acquisition_families",
            "temporal_support"
        ]
    );
    assert_eq!(left_rows[1][1], "signal.descriptive.rms");
    assert_eq!(left_rows[1][7], "0.21472615802499273");
    assert_eq!(left_rows[1][8], "V");
    assert_eq!(left_rows[10][1], "signal.spikes.flagged_fraction");
    assert!(
        left_rows
            .windows(2)
            .skip(1)
            .all(|pair| pair[0][1] < pair[1][1])
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
        let rows = csv_records(&root, "tables/artifact_lineage.csv");
        assert_eq!(rows.len(), 7);
        assert_eq!(
            rows[0],
            [
                "root_input_flag",
                "row_kind",
                "root_artifact_kind",
                "root_artifact_id",
                "lineage_state",
                "direct_dependency_role",
                "direct_dependency_kind",
                "direct_dependency_id",
                "catalog_supplied",
                "root_catalog_entry_present"
            ]
        );
        assert_eq!(
            rows.iter()
                .skip(1)
                .map(|row| (row[0].as_str(), row[1].as_str(), row[6].as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("mechanism", "root", "NA"),
                ("mechanism", "direct_dependency", "calibration_observations"),
                ("mechanism", "direct_dependency", "eis_fit"),
                ("mechanism", "direct_dependency", "state_estimation"),
                ("mechanism", "direct_dependency", "transient_analysis"),
                ("health", "root", "NA")
            ]
        );
        assert!(
            rows.iter()
                .all(|row| !row.iter().any(|cell| cell == "catalog_node"))
        );
        cleanup(&root);
    }
);

phase_d_test!(phase_d_timescale_table_uses_only_serialized_comparisons, {
    let mut options = compact_options("timescale-table");
    options.mechanism = fixture("mechanism/timescale_cmp01.json");
    explicit(&mut options, "none", "timescale_comparison");
    let root = render(&options);
    let rows = csv_records(&root, "tables/timescale_comparison.csv");
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0],
        [
            "comparison_id",
            "record_id",
            "eis_timescale_id",
            "eis_value_s",
            "eis_standard_error_s",
            "transient_timescale_id",
            "transient_value_s",
            "transient_standard_error_s",
            "ratio",
            "log10_distance",
            "symmetric_relative_difference",
            "confidence_interval_overlap",
            "compatibility_probability",
            "evidence_level",
            "supporting_evidence",
            "contradictory_evidence",
            "alternative_explanations",
            "warnings"
        ]
    );
    assert_eq!(
        rows[1],
        [
            "cmp-01",
            "rec-01",
            "eis-tau",
            "10",
            "1",
            "transient-tau",
            "11",
            "1",
            "1.1",
            "0.041",
            "0.09523809523809523",
            "true",
            "0.9",
            "moderate",
            "[]",
            "[]",
            "[]",
            "[]"
        ]
    );
    cleanup(&root);
});

phase_d_test!(
    phase_d_current_baseline_csv_uses_unique_feature_unit_authority,
    {
        let mut options = compact_options("baseline-table");
        options.health = fixture("health/comparable_with_warnings.json");
        explicit(&mut options, "none", "current_vs_baseline");
        let root = render(&options);
        let rows = csv_records(&root, "tables/current_vs_baseline.csv");
        let row = rows
            .iter()
            .find(|row| row[1] == "signal.rms_noise")
            .unwrap();
        assert_eq!(
            row,
            &[
                "available",
                "signal.rms_noise",
                "V",
                "0.21472615802499273",
                "0.058",
                "comparable_with_warnings",
                "0.15672615802499273",
                "2.702175138361943",
                "NA",
                "NA",
                "NA",
                "NA",
                "0",
                "temperature differs within configured tolerance",
                "baseline_comparable_with_warnings"
            ]
        );
        cleanup(&root);
    }
);

phase_d_test!(phase_d_current_baseline_csv_marks_missing_unit_authority, {
    let mut options = compact_options("baseline-unit");
    options.health = fixture("health/missing_unit.json");
    explicit(&mut options, "none", "current_vs_baseline");
    let root = render(&options);
    let rows = csv_records(&root, "tables/current_vs_baseline.csv");
    let row = rows
        .iter()
        .find(|row| row[1] == "signal.rms_noise")
        .unwrap();
    assert_eq!(
        row,
        &[
            "unit_authority_unavailable",
            "signal.rms_noise",
            "NA",
            "NA",
            "NA",
            "unknown",
            "NA",
            "NA",
            "NA",
            "NA",
            "NA",
            "NA",
            "NA",
            "baseline unavailable",
            "[]"
        ]
    );
    cleanup(&root);
});

phase_d_test!(phase_d_model_consistency_csv_never_recomputes_residual, {
    let mut options = compact_options("model-table");
    options.model = Some(fixture("model/missing_values.json"));
    explicit(&mut options, "none", "model_consistency");
    let root = render(&options);
    let rows = csv_records(&root, "tables/model_consistency.csv");
    assert_eq!(
        rows[0],
        [
            "availability",
            "time_s",
            "observed_voltage_v",
            "predicted_voltage_v",
            "unexplained_residual_v",
            "uncertainty_status",
            "validity_status",
            "equilibrium_status"
        ]
    );
    assert_eq!(
        rows[1],
        [
            "available",
            "0",
            "NA",
            "0",
            "NA",
            "complete",
            "valid",
            "indeterminate"
        ]
    );
    assert_eq!(
        rows[2],
        [
            "available",
            "1",
            "0.002",
            "0",
            "0.002",
            "complete",
            "valid",
            "indeterminate"
        ]
    );
    assert_eq!(
        rows[3],
        [
            "available",
            "2",
            "0.002",
            "0",
            "0.002",
            "complete",
            "valid",
            "indeterminate"
        ]
    );
    cleanup(&root);
});

phase_d_test!(phase_d_figure_mechanism_uses_stored_log_distance_only, {
    let mut options = compact_options("mechanism-figure");
    options.mechanism = fixture("mechanism/timescale_cmp01.json");
    explicit(&mut options, "mechanism_timescale", "none");
    let root = render(&options);
    let svg = read(&root, "figures/mechanism_timescale.svg");
    assert_eq!(svg.matches("data-category=\"cmp-01\"").count(), 1);
    assert!(svg.contains("data-series=\"moderate\" data-category=\"cmp-01\" data-y=\"0.041\""));
    assert!(svg.contains("y: Stored log10 distance [dimensionless]"));
    assert!(svg.contains("performs no log10 calculation"));
    assert!(!svg.contains("0.041392"));
    cleanup(&root);
});

phase_d_test!(phase_d_figure_health_shows_all_nine_statuses, {
    let mut options = compact_options("health-figure");
    explicit(&mut options, "sensor_health_dimension_status", "none");
    let root = render(&options);
    let svg = read(&root, "figures/sensor_health_dimension_status.svg");
    assert_eq!(svg.matches("data-dimension=").count(), 9);
    for exact in [
        "data-dimension=\"signal_integrity\"",
        "data-status=\"critical\"",
        "data-evidence-state=\"adequate_evidence\"",
        "data-dimension=\"observability\"",
        "data-status=\"indeterminate\"",
        "data-evidence-state=\"no_evidence\"",
        "data-dimension=\"data_quality\"",
        "data-status=\"data_quality_insufficient\"",
        "data-evidence-state=\"poor_data_quality\"",
    ] {
        assert!(svg.contains(exact), "{exact}");
    }
    assert!(svg.contains("data-categorical-grid=\"health-dimensions\""));
    assert!(!svg.contains("data-x="));
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
        assert!(svg.contains("Current versus baseline [V]"));
        assert!(svg.contains("data-series=\"current\" data-category=\"signal.rms_noise\" data-y=\"0.21472615802499273\""));
        assert!(svg.contains(
            "data-series=\"baseline\" data-category=\"signal.rms_noise\" data-y=\"0.058\""
        ));
        assert!(svg.contains("ComparableWithWarnings pairs are rendered without conversion"));
        assert!(svg.contains("temperature differs within configured tolerance"));
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
        assert!(svg.contains("x: Re(Z) [Ohm]"));
        assert!(svg.contains("y: Im(Z) [Ohm]"));
        for exact in [
            "data-series=\"observed\" data-x=\"1\" data-y=\"-2\"",
            "data-series=\"observed\" data-x=\"2\" data-y=\"-1\"",
            "data-series=\"fitted\" data-x=\"1.5\" data-y=\"-1.5\"",
            "data-series=\"fitted\" data-x=\"2.5\" data-y=\"-0.5\"",
        ] {
            assert!(svg.contains(exact), "{exact}");
        }
        assert!(svg.contains("Imaginary impedance is plotted with its serialized sign; Phase D performs no Nyquist sign transform."));
        assert!(!svg.contains("data-y=\"1.5\""));
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
        assert_eq!(svg.matches("data-panel=").count(), 2);
        assert!(svg.contains("data-panel=\"Magnitude\""));
        assert!(svg.contains("data-panel=\"Phase\""));
        for exact in [
            "data-series=\"observed magnitude\" data-x=\"1\" data-y=\"2.23606797749979\"",
            "data-series=\"observed magnitude\" data-x=\"10\" data-y=\"2.23606797749979\"",
            "data-series=\"fitted magnitude\" data-x=\"10\" data-y=\"2.5495097567963922\"",
            "data-series=\"observed phase\" data-x=\"1\" data-y=\"-63.43494882292201\"",
            "data-series=\"fitted phase\" data-x=\"10\" data-y=\"-11.309932474020215\"",
        ] {
            assert!(svg.contains(exact), "{exact}");
        }
        assert!(svg.contains("x: Frequency [Hz] (log display axis)"));
        assert!(svg.contains("y: Magnitude [Ohm]"));
        assert!(svg.contains("y: Phase [deg]"));
        assert!(svg.contains("source artifact sha256:312fee67b46260013fb21405ff0448917cd55ed140aee63f57114cba11383c90"));
        assert!(svg.contains("Observed magnitude uses serialized derived_magnitude_ohm"));
        assert!(svg.contains("Phase D performs no sqrt, atan, or atan2 calculation."));
        cleanup(&root);
    }
);

phase_d_test!(phase_d_figure_transient_renders_one_unique_selected_fit, {
    let mut options = compact_options("transient-unique");
    options.transient = Some(fixture("base/transient.json"));
    explicit(&mut options, "transient_response", "none");
    let root = render(&options);
    let svg = read(&root, "figures/transient_response.svg");
    assert_eq!(svg.matches("data-series=\"observed\"").count(), 301);
    assert_eq!(svg.matches("data-series=\"fitted\"").count(), 2);
    assert_eq!(svg.matches("data-series=\"residual\"").count(), 2);
    assert!(svg.contains("Events are separate series."));
    assert!(svg.contains(
        "source artifact sha256:d9465a5deff1224c5190dae21a674c34e9eb293f88055973491616ea2ba02b5c"
    ));
    for exact in [
        "data-series=\"observed\" data-x=\"1\" data-y=\"0.29200444146293236\"",
        "data-series=\"fitted\" data-x=\"0\" data-y=\"0.30000000000000004\"",
        "data-series=\"fitted\" data-x=\"10\" data-y=\"0.29200444146293236\"",
        "data-series=\"residual\" data-x=\"10\" data-y=\"0\"",
        "event 0: predicted_v has 301 serialized values but fitted_time_local has 2 coordinates; only serialized coordinate/value pairs are plotted",
        "event 0: residuals_v has 301 serialized values but fitted_time_local has 2 coordinates; only serialized coordinate/value pairs are plotted",
    ] {
        assert!(svg.contains(exact), "{exact}");
    }
    cleanup(&root);
});

phase_d_test!(
    phase_d_figure_transient_zero_match_default_is_manifest_unavailable,
    {
        let mut options = compact_options("transient-zero-default");
        options.transient = Some(fixture("transient/zero_selected_fit.json"));
        options.selection = ReportSelection::parse(None, Some("none")).expect("default selection");
        let root = render(&options);
        let manifest = json(&root, "render_manifest.schema1.json");
        assert!(manifest["unavailable_outputs"].as_array().unwrap().contains(&serde_json::json!({
            "output_kind": "figure", "output_id": "transient_response", "reason": "selected_fit_not_found"
        })));
        assert!(!root.join("figures/transient_response.svg").exists());
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
            Err(PublicReportError::RequestedOutputUnavailable { output_id, reason: rust_electroanalysis_cli::reporting::AvailabilityReason::SelectedFitNotFound })
                if output_id == "transient_response"
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
        let manifest = json(&root, "render_manifest.schema1.json");
        assert!(manifest["unavailable_outputs"].as_array().unwrap().contains(&serde_json::json!({
            "output_kind": "figure", "output_id": "transient_response", "reason": "selected_fit_ambiguous"
        })));
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
    assert_eq!(svg.matches("data-series=\"observed\"").count(), 3);
    assert_eq!(svg.matches("data-series=\"predicted\"").count(), 3);
    for exact in [
        "data-series=\"observed\" data-x=\"-3\" data-y=\"0.1\"",
        "data-series=\"predicted\" data-x=\"-3\" data-y=\"0.09996724278094837\"",
        "data-series=\"observed\" data-x=\"-1\" data-y=\"0.22000000000000003\"",
        "ValidationPredictionPoint does not serialize a residual; Phase D does not recompute one.",
    ] {
        assert!(svg.contains(exact), "{exact}");
    }
    assert!(!svg.contains("data-series=\"residual\""));
    assert!(!svg.to_ascii_lowercase().contains("theoretical curve"));
    cleanup(&root);
});

phase_d_test!(phase_d_figure_signal_marks_missing_samples, {
    let mut options = compact_options("signal-figure");
    options.signal = Some(fixture("base/signal.json"));
    explicit(&mut options, "signal_diagnostics", "none");
    let root = render(&options);
    let svg = read(&root, "figures/signal_diagnostics.svg");
    assert_eq!(svg.matches("data-panel=").count(), 3);
    assert_eq!(svg.matches("data-series=\"time signal\"").count(), 7);
    assert!(svg.contains("data-series=\"time signal\" data-x=\"1\" data-y=\"0.11\""));
    assert!(!svg.contains("data-series=\"time signal\" data-x=\"2\""));
    assert!(svg.contains("signal: NA at serialized x=2; no plotting coordinate assigned"));
    assert!(svg.contains("data-panel=\"Power spectral density\""));
    assert!(svg.contains("PSD unavailable in the serialized artifact."));
    assert!(svg.contains("data-panel=\"Allan deviation\""));
    assert!(svg.contains("Allan deviation unavailable in the serialized artifact."));
    cleanup(&root);
});

phase_d_test!(
    phase_d_figure_estimation_shows_serialized_uncertainty_only,
    {
        let mut options = compact_options("estimation-figure");
        options.estimation = Some(fixture("base/estimation.json"));
        options.selection = ReportSelection::parse(None, Some("none")).expect("default selection");
        let root = render(&options);
        let manifest = json(&root, "render_manifest.schema1.json");
        assert!(manifest["unavailable_outputs"].as_array().unwrap().contains(&serde_json::json!({
            "output_kind": "figure", "output_id": "estimation_observed_predicted", "reason": "serialized_series_unavailable"
        })));
        assert!(
            !root
                .join("figures/estimation_observed_predicted.svg")
                .exists()
        );
        cleanup(&root);
    }
);

phase_d_test!(phase_d_figure_model_never_maps_missing_to_zero, {
    let mut options = compact_options("model-figure");
    options.model = Some(fixture("model/missing_values.json"));
    explicit(&mut options, "model_observed_predicted", "none");
    let root = render(&options);
    let svg = read(&root, "figures/model_observed_predicted.svg");
    assert_eq!(svg.matches("data-series=\"observed\"").count(), 2);
    assert_eq!(svg.matches("data-series=\"predicted\"").count(), 3);
    assert_eq!(svg.matches("data-series=\"residual\"").count(), 2);
    assert!(!svg.contains("data-series=\"observed\" data-x=\"0\""));
    assert!(!svg.contains("data-series=\"residual\" data-x=\"0\""));
    assert!(svg.contains("data-series=\"predicted\" data-x=\"0\" data-y=\"0\""));
    assert!(svg.contains("observed: NA at serialized x=0; no plotting coordinate assigned"));
    assert!(svg.contains("residual: NA at serialized x=0; no plotting coordinate assigned"));
    cleanup(&root);
});

phase_d_test!(phase_d_figure_lineage_marks_legacy_unknown, {
    let mut options = compact_options("legacy-lineage");
    options.mechanism = fixture("legacy/mechanism_v1.json");
    explicit(&mut options, "lineage", "none");
    let root = render(&options);
    let svg = read(&root, "figures/lineage.svg");
    assert!(svg.contains("LegacyUnknown / schema NA / FieldAbsentInLegacyArtifact"));
    assert!(svg.contains("data-lineage-graph=\"root-direct-only\""));
    assert_eq!(svg.matches("<line ").count(), 0);
    assert!(!svg.contains("catalog_node"));
    cleanup(&root);
});

phase_d_test!(phase_d_selected_figure_files_are_valid_svg_and_png, {
    let root = render(&full_options("image-validity"));
    let manifest = json(&root, "render_manifest.schema1.json");
    let unavailable = manifest["unavailable_outputs"].as_array().unwrap();
    let rendered = [
        ("sensor_health_dimension_status", (1600, 1000)),
        ("eis_nyquist", (1600, 1000)),
        ("eis_bode", (1600, 1400)),
        ("transient_response", (1600, 1400)),
        ("calibration_performance", (1600, 1400)),
        ("signal_diagnostics", (1600, 1400)),
        ("model_observed_predicted", (1600, 1400)),
        ("lineage", (1600, 1000)),
    ];
    assert_eq!(unavailable.len(), 3);
    for (id, dimensions) in rendered {
        let svg = read(&root, &format!("figures/{id}.svg"));
        assert!(
            svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""),
            "{id}"
        );
        assert!(svg.ends_with("</svg>"), "{id}");
        assert!(
            svg.contains(&format!(
                "phase_d_figure={id};threshold_lines=0;missing_values_have_no_plot_coordinates=true"
            )),
            "{id}"
        );
        assert_eq!(
            image::image_dimensions(root.join(format!("figures/{id}.png")))
                .expect("PNG dimensions"),
            dimensions,
            "{id}"
        );
    }
    cleanup(&root);
});

phase_d_test!(
    phase_d_figure_metadata_has_labels_units_series_and_dqi_visibility,
    {
        let mut options = compact_options("figure-metadata");
        options.eis = Some(fixture("eis/nyquist_bode.json"));
        explicit(
            &mut options,
            "sensor_health_dimension_status,eis_nyquist",
            "none",
        );
        let root = render(&options);
        let svg = read(&root, "figures/eis_nyquist.svg");
        assert!(svg.contains("phase_d_figure=eis_nyquist;threshold_lines=0;missing_values_have_no_plot_coordinates=true"));
        assert!(svg.contains("x: Re(Z) [Ohm]"));
        assert!(svg.contains("y: Im(Z) [Ohm]"));
        assert_eq!(svg.matches("data-series=\"observed\"").count(), 2);
        assert_eq!(svg.matches("data-series=\"fitted\"").count(), 2);
        assert!(svg.contains(
            "source sha256:312fee67b46260013fb21405ff0448917cd55ed140aee63f57114cba11383c90"
        ));
        assert!(
            svg.contains("producer warning: parameter covariance was not available for this fit")
        );
        let health = read(&root, "figures/sensor_health_dimension_status.svg");
        assert!(health.contains("Data quality insufficient (DQI)"));
        assert!(health.contains("source artifact sha256:4717ab60c11af2a14fb665ff07427530861d2eb52773b288a733b9e814562964"));
        assert_eq!(health.matches("data-status=\"indeterminate\"").count(), 7);
        assert!(health.contains("data-dimension=\"data_quality\""));
        assert!(health.contains("data-status=\"data_quality_insufficient\""));
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
        assert_eq!(
            relative_files(&root),
            [
                "figures/eis_nyquist.png",
                "figures/eis_nyquist.svg",
                "public_summary.schema1.json",
                "render_manifest.schema1.json"
            ]
        );
        let manifest = json(&root, "render_manifest.schema1.json");
        assert_eq!(
            manifest["requested"]["formats"],
            serde_json::json!(["json"])
        );
        assert_eq!(
            generated_paths(&manifest),
            [
                "public_summary.schema1.json",
                "figures/eis_nyquist.svg",
                "figures/eis_nyquist.png",
                "render_manifest.schema1.json"
            ]
        );
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
        assert_eq!(
            relative_files(&root),
            [
                "figures/eis_nyquist.png",
                "figures/eis_nyquist.svg",
                "render_manifest.schema1.json",
                "scientific_report.md"
            ]
        );
        let manifest = json(&root, "render_manifest.schema1.json");
        assert_eq!(
            manifest["requested"]["formats"],
            serde_json::json!(["markdown"])
        );
        assert_eq!(
            generated_paths(&manifest),
            [
                "scientific_report.md",
                "figures/eis_nyquist.svg",
                "figures/eis_nyquist.png",
                "render_manifest.schema1.json"
            ]
        );
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
        let manifest = json(&default_root, "render_manifest.schema1.json");
        assert_eq!(manifest["requested"]["figures_mode"], "default");
        assert_eq!(
            manifest["requested"]["figures"],
            serde_json::json!([
                "mechanism_timescale",
                "sensor_health_dimension_status",
                "current_vs_baseline",
                "lineage"
            ])
        );
        assert_eq!(
            manifest["unavailable_outputs"],
            serde_json::json!([
                {"output_kind":"figure", "output_id":"mechanism_timescale", "reason":"serialized_series_unavailable"},
                {"output_kind":"figure", "output_id":"current_vs_baseline", "reason":"comparison_unknown"}
            ])
        );
        assert!(
            default_root
                .join("figures/sensor_health_dimension_status.svg")
                .is_file()
        );
        assert!(default_root.join("figures/lineage.svg").is_file());
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
    assert_eq!(format_public_f64(0.041).expect("finite"), "0.041");
    assert_eq!(
        format_public_f64(100000000000000000000.0).expect("finite"),
        "100000000000000000000"
    );
    assert_eq!(
        format_public_f64(-63.43494882292201).expect("finite"),
        "-63.43494882292201"
    );
});

phase_d_test!(
    phase_d_csv_markdown_and_figure_annotations_share_float_format,
    {
        let mut options = compact_options("cross-format-number");
        options.mechanism = fixture("mechanism/timescale_cmp01.json");
        explicit(&mut options, "mechanism_timescale", "timescale_comparison");
        let root = render(&options);
        let csv = csv_records(&root, "tables/timescale_comparison.csv");
        assert_eq!(csv[1][9], "0.041");
        let svg = read(&root, "figures/mechanism_timescale.svg");
        assert!(svg.contains("data-y=\"0.041\""));
        let summary = json(&root, "public_summary.schema1.json");
        assert_eq!(
            summary["mechanism"]["comparisons"][0]["log10_distance"],
            0.041
        );
        assert!(read(&root, "scientific_report.md").contains("| 1.1 | 0.041 | moderate |"));
        cleanup(&root);
    }
);

phase_d_test!(phase_d_nonfinite_projection_fails_before_serialization, {
    assert!(format_public_f64(f64::NAN).is_err());
    assert!(format_public_f64(f64::INFINITY).is_err());
    assert!(format_public_f64(f64::NEG_INFINITY).is_err());
    let output = temporary_root("nonfinite-projection");
    let result = cli_output(
        &compact_cli_arguments(&output, false),
        &[(
            "ELECTROANALYSIS_PHASE_D_TEST_NONFINITE_PROJECTION",
            Path::new("1"),
        )],
    );
    assert!(!result.status.success());
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.contains("staging validation failed for public report projection"));
    assert!(stderr.contains("non-finite number in public projection"));
    assert!(!output.exists());
    assert!(publication_siblings(&output).is_empty());
});

phase_d_test!(phase_d_staging_write_failure_publishes_no_final_bundle, {
    let output = temporary_root("staging-write-failure");
    let mut arguments = compact_cli_arguments(&output, false);
    let tables_index = arguments.iter().position(|arg| arg == "--tables").unwrap() + 1;
    arguments[tables_index] = "mechanism_evidence".into();
    let result = cli_output(
        &arguments,
        &[(
            "ELECTROANALYSIS_PHASE_D_TEST_FAIL_WRITE",
            Path::new("tables/mechanism_evidence.csv"),
        )],
    );
    assert!(!result.status.success());
    let stderr = String::from_utf8(result.stderr).unwrap();
    assert!(stderr.contains("could not write report output"));
    assert!(stderr.contains("injected Phase-D staged writer failure"));
    assert!(!output.exists());
    assert!(publication_siblings(&output).is_empty());

    let cleanup_output = temporary_root("staging-cleanup-failure");
    let mut cleanup_arguments = compact_cli_arguments(&cleanup_output, false);
    let tables_index = cleanup_arguments
        .iter()
        .position(|arg| arg == "--tables")
        .unwrap()
        + 1;
    cleanup_arguments[tables_index] = "mechanism_evidence".into();
    let cleanup_result = cli_output(
        &cleanup_arguments,
        &[
            (
                "ELECTROANALYSIS_PHASE_D_TEST_FAIL_WRITE",
                Path::new("tables/mechanism_evidence.csv"),
            ),
            (
                "ELECTROANALYSIS_PHASE_D_TEST_FAIL_CLEANUP",
                Path::new("staging"),
            ),
        ],
    );
    assert!(!cleanup_result.status.success());
    let cleanup_stderr = String::from_utf8(cleanup_result.stderr).unwrap();
    assert!(cleanup_stderr.contains("could not clean up"));
    assert!(cleanup_stderr.contains("phase-d-staging"));
    assert!(!cleanup_output.exists());
    let retained = publication_siblings(&cleanup_output);
    assert_eq!(retained.len(), 1);
    assert!(
        retained[0]
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("phase-d-staging")
    );
    cleanup(&cleanup_output);
});

phase_d_test!(
    phase_d_publication_failure_restores_previous_complete_bundle,
    {
        let options = compact_options("managed-overwrite");
        let root = render(&options);
        let before = bundle_bytes(&root);
        let result = cli_output(
            &compact_cli_arguments(&root, true),
            &[(
                "ELECTROANALYSIS_PHASE_D_TEST_FAIL_PUBLISH_RENAME",
                Path::new("1"),
            )],
        );
        assert!(!result.status.success());
        assert!(
            String::from_utf8(result.stderr)
                .unwrap()
                .contains("PublishRename")
        );
        assert_eq!(bundle_bytes(&root), before);
        assert!(publication_siblings(&root).is_empty());
        cleanup(&root);

        let restore_options = compact_options("restore-failure");
        let restore_root = render(&restore_options);
        let restore_before = bundle_bytes(&restore_root);
        let restore_result = cli_output(
            &compact_cli_arguments(&restore_root, true),
            &[
                (
                    "ELECTROANALYSIS_PHASE_D_TEST_FAIL_PUBLISH_RENAME",
                    Path::new("1"),
                ),
                (
                    "ELECTROANALYSIS_PHASE_D_TEST_FAIL_RESTORE_RENAME",
                    Path::new("1"),
                ),
            ],
        );
        assert!(!restore_result.status.success());
        assert!(
            String::from_utf8(restore_result.stderr)
                .unwrap()
                .contains("RestoreRename")
        );
        assert!(!restore_root.exists());
        let restore_siblings = publication_siblings(&restore_root);
        assert_eq!(restore_siblings.len(), 1);
        assert!(
            restore_siblings[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains("phase-d-backup")
        );
        assert_eq!(bundle_bytes(&restore_siblings[0]), restore_before);
        fs::rename(&restore_siblings[0], &restore_root).expect("restore recovery bundle");
        cleanup(&restore_root);

        let mut collision_options = compact_options("backup-collision");
        let collision_root = render(&collision_options);
        let collision_path = collision_root.parent().unwrap().join(format!(
            ".{}.phase-d-backup-{}-0",
            collision_root.file_name().unwrap().to_string_lossy(),
            std::process::id()
        ));
        fs::create_dir(&collision_path).expect("reserved backup collision");
        fs::write(collision_path.join("sentinel"), "retain").expect("collision sentinel");
        collision_options.overwrite = true;
        report::render(&collision_options).expect("backup collision advances attempt");
        assert_certified_bundle_integrity(&collision_root);
        assert_eq!(
            fs::read_to_string(collision_path.join("sentinel")).unwrap(),
            "retain"
        );
        assert_eq!(publication_siblings(&collision_root), [collision_path]);
        cleanup(&collision_root);

        let cleanup_options = compact_options("backup-cleanup-failure");
        let cleanup_root = render(&cleanup_options);
        let cleanup_before = bundle_bytes(&cleanup_root);
        let cleanup_result = cli_output(
            &compact_cli_arguments(&cleanup_root, true),
            &[(
                "ELECTROANALYSIS_PHASE_D_TEST_FAIL_CLEANUP",
                Path::new("backup"),
            )],
        );
        assert!(!cleanup_result.status.success());
        assert!(
            String::from_utf8(cleanup_result.stderr)
                .unwrap()
                .contains("could not clean up")
        );
        assert_certified_bundle_integrity(&cleanup_root);
        let cleanup_siblings = publication_siblings(&cleanup_root);
        assert_eq!(cleanup_siblings.len(), 1);
        assert_eq!(bundle_bytes(&cleanup_siblings[0]), cleanup_before);
        cleanup(&cleanup_root);
    }
);

phase_d_test!(phase_d_rendering_does_not_mutate_health_assessment, {
    let source = fixture("base/health.json");
    let before = fs::read(&source).expect("source bytes");
    let before_value: rust_electroanalysis_cli::results::SensorHealthAssessment =
        rust_electroanalysis_cli::domain::read_artifact(&source).expect("health before");
    let root = render(&compact_options("immutable-health"));
    assert_eq!(fs::read(source).expect("source bytes"), before);
    let after_value: rust_electroanalysis_cli::results::SensorHealthAssessment =
        rust_electroanalysis_cli::domain::read_artifact(&fixture("base/health.json"))
            .expect("health after");
    assert_eq!(after_value, before_value);
    cleanup(&root);
});

phase_d_test!(phase_d_rendering_does_not_mutate_mechanism_assessment, {
    let source = fixture("base/mechanism.json");
    let before = fs::read(&source).expect("source bytes");
    let before_value: rust_electroanalysis_cli::results::MechanismAnalysisReport =
        rust_electroanalysis_cli::domain::read_artifact(&source).expect("mechanism before");
    let root = render(&compact_options("immutable-mechanism"));
    assert_eq!(fs::read(source).expect("source bytes"), before);
    let after_value: rust_electroanalysis_cli::results::MechanismAnalysisReport =
        rust_electroanalysis_cli::domain::read_artifact(&fixture("base/mechanism.json"))
            .expect("mechanism after");
    assert_eq!(after_value, before_value);
    cleanup(&root);
});

phase_d_test!(phase_d_repeated_render_is_deterministic, {
    let left = render(&full_options("deterministic-left"));
    let right = render(&full_options("deterministic-right"));
    let left_bytes = bundle_bytes(&left);
    let right_bytes = bundle_bytes(&right);
    assert_eq!(
        left_bytes.keys().collect::<Vec<_>>(),
        right_bytes.keys().collect::<Vec<_>>()
    );
    assert_eq!(left_bytes, right_bytes);
    cleanup(&left);
    cleanup(&right);
});

phase_d_test!(
    phase_d_large_history_does_not_duplicate_artifact_series_unboundedly,
    {
        use rust_electroanalysis_cli::domain::read_artifact;
        use rust_electroanalysis_cli::results::{MechanismAnalysisReport, SensorHealthAssessment};
        let mechanism_path = fixture("scale/mechanism_large_history.json");
        let health_path = fixture("scale/health_large_evidence.json");
        let mechanism_bytes = fs::read(&mechanism_path).expect("N-F29 bytes");
        let health_bytes = fs::read(&health_path).expect("N-F30 bytes");
        let mechanism: MechanismAnalysisReport = read_artifact(&mechanism_path).expect("N-F29");
        let health: SensorHealthAssessment = read_artifact(&health_path).expect("N-F30");
        assert_eq!(mechanism.hypothesis_history.len(), 1000);
        assert_eq!(
            health
                .phase_c
                .as_ref()
                .expect("phase c")
                .evidence_bundle
                .records
                .len(),
            10_000
        );
        assert!(matches!(
            mechanism.lineage,
            rust_electroanalysis_cli::domain::ArtifactLineageState::Known { .. }
        ));
        assert!(matches!(
            health.lineage,
            rust_electroanalysis_cli::domain::ArtifactLineageState::Known { .. }
        ));

        let output = temporary_root("large-certified-cli");
        let audit = temporary_root("large-traversal-audit.json");
        let mut arguments = full_cli_arguments(&output);
        let mechanism_index = arguments
            .iter()
            .position(|arg| arg == "--mechanism")
            .unwrap()
            + 1;
        let health_index = arguments.iter().position(|arg| arg == "--health").unwrap() + 1;
        arguments[mechanism_index] = mechanism_path.to_string_lossy().into_owned();
        arguments[health_index] = health_path.to_string_lossy().into_owned();
        arguments.extend(["--format".into(), "json".into()]);
        let result = cli_output(
            &arguments,
            &[("ELECTROANALYSIS_PHASE_D_TEST_TRAVERSAL_AUDIT", &audit)],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            String::from_utf8(result.stdout)
                .unwrap()
                .contains("(25 written, 3 unavailable)")
        );
        assert_certified_bundle_integrity(&output);
        assert_eq!(
            generated_paths(&json(&output, "render_manifest.schema1.json")),
            [
                "public_summary.schema1.json",
                "tables/mechanism_evidence.csv",
                "tables/health_dimensions.csv",
                "tables/evidence_provenance.csv",
                "tables/artifact_lineage.csv",
                "tables/timescale_comparison.csv",
                "tables/model_consistency.csv",
                "tables/current_vs_baseline.csv",
                "figures/sensor_health_dimension_status.svg",
                "figures/sensor_health_dimension_status.png",
                "figures/eis_nyquist.svg",
                "figures/eis_nyquist.png",
                "figures/eis_bode.svg",
                "figures/eis_bode.png",
                "figures/transient_response.svg",
                "figures/transient_response.png",
                "figures/calibration_performance.svg",
                "figures/calibration_performance.png",
                "figures/signal_diagnostics.svg",
                "figures/signal_diagnostics.png",
                "figures/model_observed_predicted.svg",
                "figures/model_observed_predicted.png",
                "figures/lineage.svg",
                "figures/lineage.png",
                "render_manifest.schema1.json",
            ]
        );
        let manifest = json(&output, "render_manifest.schema1.json");
        assert_eq!(
            manifest["unavailable_outputs"],
            serde_json::json!([
                {"output_kind":"figure", "output_id":"mechanism_timescale", "reason":"serialized_series_unavailable"},
                {"output_kind":"figure", "output_id":"current_vs_baseline", "reason":"comparison_unknown"},
                {"output_kind":"figure", "output_id":"estimation_observed_predicted", "reason":"serialized_series_unavailable"}
            ])
        );
        assert_eq!(
            json(
                audit.parent().unwrap(),
                audit.file_name().unwrap().to_str().unwrap()
            ),
            serde_json::json!({
                "mechanism_history_projection_traversals": 1,
                "health_evidence_projection_traversals": 1,
                "mechanism_history_count": 1000,
                "health_evidence_count": 10000
            })
        );
        let summary = json(&output, "public_summary.schema1.json");
        assert_eq!(summary["input_references"].as_array().unwrap().len(), 10);
        assert_eq!(fs::read(&mechanism_path).unwrap(), mechanism_bytes);
        assert_eq!(fs::read(&health_path).unwrap(), health_bytes);
        let mechanism_after: MechanismAnalysisReport = read_artifact(&mechanism_path).unwrap();
        let health_after: SensorHealthAssessment = read_artifact(&health_path).unwrap();
        assert_eq!(mechanism_after.hypothesis_history.len(), 1000);
        assert_eq!(
            health_after
                .phase_c
                .expect("phase c")
                .evidence_bundle
                .records
                .len(),
            10_000
        );
        cleanup(&output);
        let _ = fs::remove_file(audit);
    }
);

phase_d_test!(
    phase_d_golden_expectations_are_hand_derived_from_fixture_literals,
    {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/phase_d");
        let mut files = Vec::new();
        fixture_files(&fixtures, &mut files);
        files.sort();
        assert_eq!(files.len(), 36);
        assert!(files.iter().all(|path| {
            !path
                .file_name()
                .expect("name")
                .to_string_lossy()
                .contains("golden")
        }));
        let production_source = [
            include_str!("../src/reporting/document.rs"),
            include_str!("../src/reporting/figures.rs"),
            include_str!("../src/reporting/tables.rs"),
            include_str!("../src/runners/report.rs"),
        ]
        .join("\n");
        for forbidden in [
            "insta::",
            "assert_snapshot",
            "golden.json",
            "generate_fixture",
        ] {
            assert!(
                !production_source.contains(forbidden),
                "forbidden circular oracle: {forbidden}"
            );
        }
        let comparison: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture("mechanism/timescale_cmp01.json")).unwrap())
                .unwrap();
        assert_eq!(comparison["comparisons"][0]["ratio"], 1.1);
        assert_eq!(comparison["comparisons"][0]["log10_distance"], 0.041);
        assert_eq!(comparison["comparisons"][0]["evidence_level"], "moderate");
        let eis: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture("eis/nyquist_bode.json")).unwrap()).unwrap();
        assert_eq!(eis["source"]["z_imag_ohm"], serde_json::json!([-2.0, -1.0]));
        assert_eq!(eis["fitted"]["z_imag_ohm"], serde_json::json!([-1.5, -0.5]));
        assert_eq!(
            eis["source"]["derived_phase_deg"],
            serde_json::json!([-63.43494882292201, -26.56505117707799])
        );
    }
);

phase_d_test!(phase_d_public_report_error_is_publicly_reachable, {
    let error = PublicReportError::InvalidCombination { detail: "test" };
    assert_eq!(error.to_string(), "invalid report option combination: test");
    let runner = RunnerError::from(error);
    assert_eq!(
        runner.to_string(),
        "invalid report option combination: test"
    );
    assert!(matches!(
        runner,
        RunnerError::PublicReport(PublicReportError::InvalidCombination { detail: "test" })
    ));
});

#[test]
fn phase_d_catalog_reader_rejects_syntactically_malformed_json() {
    let path = fixture("failure/catalog_malformed.json");
    assert_eq!(fs::read(&path).expect("literal bytes"), b"{not-json}\n");
    let error = read_artifact_lineage_catalog(&path).expect_err("malformed catalog must fail");
    match error {
        LineageCatalogReadError::Json { path: actual, .. } => assert_eq!(actual, path),
        other => panic!("expected JSON error, got {other:?}"),
    }
    let mut options = compact_options("catalog-malformed-route");
    options.lineage_catalog = Some(path.clone());
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::LineageCatalog { path: actual, source: LineageCatalogReadError::Json { path: nested, .. } })
            if actual == path && nested == path
    ));
    assert!(!options.output_dir.exists());
}

#[test]
fn phase_d_catalog_reader_rejects_structurally_invalid_catalog() {
    let path = fixture("failure/catalog_invalid_structure.json");
    assert_eq!(
        fs::read(&path).expect("literal bytes"),
        b"{\"schema_version\":1,\"artifacts\":{},\"unexpected\":true}\n"
    );
    let error =
        read_artifact_lineage_catalog(&path).expect_err("closed-schema violation must fail");
    match error {
        LineageCatalogReadError::UnknownField {
            path: actual,
            field,
        } => {
            assert_eq!(actual, path);
            assert_eq!(field, "unexpected");
        }
        other => panic!("expected UnknownField, got {other:?}"),
    }
    let mut options = compact_options("catalog-structure-route");
    options.lineage_catalog = Some(path.clone());
    assert!(matches!(
        report::render(&options),
        Err(PublicReportError::LineageCatalog { path: actual, source: LineageCatalogReadError::UnknownField { path: nested, field } })
            if actual == path && nested == path && field == "unexpected"
    ));
    assert!(!options.output_dir.exists());
}

phase_d_test!(
    phase_d_different_known_acquisition_families_are_projected_not_rejected,
    {
        let mut options = compact_options("different-families");
        options.mechanism = fixture("compat/mechanism_family.json");
        options.health = fixture("compat/health_family.json");
        let root = render(&options);
        let summary = json(&root, "public_summary.schema1.json");
        assert_eq!(summary["compatibility"]["required_pair"], "compatible");
        let references = summary["input_references"].as_array().unwrap();
        let mechanism = references
            .iter()
            .find(|item| item["input_flag"] == "mechanism")
            .unwrap();
        let health = references
            .iter()
            .find(|item| item["input_flag"] == "health")
            .unwrap();
        assert_eq!(
            mechanism["acquisition_families"],
            serde_json::json!({"status":"known", "values":["family-mechanism"]})
        );
        assert_eq!(
            health["acquisition_families"],
            serde_json::json!({"status":"known", "values":["family-health"]})
        );
        let manifest = json(&root, "render_manifest.schema1.json");
        assert_eq!(
            manifest["optional_compatibility"].as_array().unwrap().len(),
            7
        );
        assert!(
            manifest["optional_compatibility"]
                .as_array()
                .unwrap()
                .iter()
                .all(|item| item["status"] == "not_provided"
                    && item["against_flag"] == "mechanism")
        );
        assert!(!read(&root, "public_summary.schema1.json").contains("same_source"));
        assert!(!read(&root, "scientific_report.md").contains("independent"));
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_comparable_with_warnings_is_rendered_and_disclosed,
    {
        use rust_electroanalysis_cli::{domain::read_artifact, results::SensorHealthAssessment};
        let source = fixture("health/comparable_with_warnings.json");
        let health: SensorHealthAssessment =
            read_artifact(&source).expect("N-F23 canonical reader");
        assert_eq!(health.schema_version, 4);
        assert_eq!(
            health.phase_c.as_ref().unwrap().dimension_assessments.len(),
            9
        );
        let comparison = &health.baseline_comparison[0];
        assert_eq!(comparison.feature, "signal.rms_noise");
        assert_eq!(comparison.current_value, Some(0.21472615802499273));
        assert_eq!(comparison.baseline_value, Some(0.058));
        assert_eq!(
            format!("{:?}", comparison.comparability),
            "ComparableWithWarnings"
        );
        assert_eq!(comparison.absolute_difference, Some(0.15672615802499273));
        assert_eq!(comparison.relative_difference, Some(2.702175138361943));
        assert_eq!(
            comparison.override_reason.as_deref(),
            Some("temperature differs within configured tolerance")
        );
        let mut options = compact_options("comparable-warning");
        options.health = source.clone();
        explicit(&mut options, "current_vs_baseline", "current_vs_baseline");
        let root = render(&options);
        let svg = read(&root, "figures/current_vs_baseline.svg");
        assert!(svg.contains("data-series=\"current\" data-category=\"signal.rms_noise\" data-y=\"0.21472615802499273\""));
        assert!(svg.contains(
            "data-series=\"baseline\" data-category=\"signal.rms_noise\" data-y=\"0.058\""
        ));
        let rows = csv_records(&root, "tables/current_vs_baseline.csv");
        assert_eq!(
            rows.iter()
                .find(|row| row[1] == "signal.rms_noise")
                .unwrap(),
            &[
                "available",
                "signal.rms_noise",
                "V",
                "0.21472615802499273",
                "0.058",
                "comparable_with_warnings",
                "0.15672615802499273",
                "2.702175138361943",
                "NA",
                "NA",
                "NA",
                "NA",
                "0",
                "temperature differs within configured tolerance",
                "baseline_comparable_with_warnings"
            ]
        );
        let manifest = json(&root, "render_manifest.schema1.json");
        assert!(manifest["warnings"].as_array().unwrap().contains(&serde_json::json!({"code":"baseline_comparable_with_warnings", "message":"temperature differs within configured tolerance", "input_flag":"health", "output_id":"current_vs_baseline"})));
        assert!(read(&root, "scientific_report.md").contains(
            "| signal.rms_noise | V | 0.21472615802499273 | 0.058 | comparable_with_warnings | 0.15672615802499273 | 2.702175138361943 | temperature differs within configured tolerance |"
        ));
        let malformed = temporary_root("pascal-case-health.json");
        let bytes = fs::read_to_string(&source)
            .unwrap()
            .replace("\"comparable_with_warnings\"", "\"ComparableWithWarnings\"");
        fs::write(&malformed, bytes).unwrap();
        assert!(read_artifact::<SensorHealthAssessment>(&malformed).is_err());
        let _ = fs::remove_file(malformed);
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
            if relative == "public_summary.schema1.json" {
                exact_keys(
                    catalog,
                    &[
                        "input_kind",
                        "supplied_path_basename",
                        "schema_version",
                        "availability",
                        "validation",
                    ],
                );
            } else {
                exact_keys(
                    catalog,
                    &[
                        "input_kind",
                        "supplied_path_basename",
                        "schema_version",
                        "availability",
                        "validation",
                        "compatibility",
                    ],
                );
                assert_eq!(catalog["compatibility"], "not_applicable");
            }
            assert_eq!(catalog["input_kind"], "lineage_catalog");
            assert_eq!(catalog["supplied_path_basename"], "lineage_catalog.json");
            assert_eq!(catalog["schema_version"], 1);
            assert_eq!(catalog["availability"], "available");
            assert_eq!(catalog["validation"], "validated");
            for forbidden in [
                "input_flag",
                "artifact_id",
                "artifact_kind",
                "lineage",
                "acquisition_families",
            ] {
                assert!(catalog.get(forbidden).is_none(), "{relative}: {forbidden}");
            }
        }
        let before = bundle_bytes(&root);
        let manifest_path = root.join("render_manifest.schema1.json");
        let mut manifest = json(&root, "render_manifest.schema1.json");
        let catalog = manifest["input_references"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|item| item["input_kind"] == "lineage_catalog")
            .unwrap();
        catalog.as_object_mut().unwrap().insert(
            "artifact_kind".into(),
            serde_json::json!("artifact_lineage_catalog"),
        );
        fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let mut overwrite = compact_options("unused");
        overwrite.output_dir = root.clone();
        overwrite.overwrite = true;
        assert!(
            matches!(report::render(&overwrite), Err(PublicReportError::UnmanagedOutputEntry { path }) if path == manifest_path)
        );
        assert!(root.is_dir());
        let mut restored = before;
        restored.remove("render_manifest.schema1.json");
        let mut current = bundle_bytes(&root);
        current.remove("render_manifest.schema1.json");
        assert_eq!(current, restored);
        cleanup(&root);
    }
);

phase_d_test!(
    phase_d_fixture_ledger_materializes_exact_literal_files_and_canonical_readers_accept_them,
    {
        use rust_electroanalysis_cli::{
            domain::{ArtifactLineageState, read_artifact},
            results::{
                CalibrationAnalysisReport, CalibrationObservationSet, EisFitArtifact,
                MechanismAnalysisReport, ModelAnalysisReport, SensorHealthAssessment,
                SignalAnalysisReport, StateEstimationReport, TransientAnalysisReport,
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
        let mut valid_reader_count = 0_usize;
        macro_rules! accept {
            ($type:ty, $relative:literal, $schema:literal, $known:literal, $provenance:literal) => {{
                let artifact: $type = read_artifact(&fixture($relative)).expect($relative);
                valid_reader_count += 1;
                assert_eq!(artifact.schema_version, $schema, $relative);
                match (&artifact.lineage, $known) {
                    (
                        ArtifactLineageState::Known {
                            identity,
                            direct_dependencies,
                        },
                        true,
                    ) => {
                        identity.validate().expect("valid known identity");
                        assert_eq!(identity.schema_version, artifact.schema_version, $relative);
                        assert_eq!(
                            identity.artifact_id.0,
                            format!("sha256:{}", identity.semantic_sha256),
                            $relative
                        );
                        assert!(!identity.producer_version.is_empty(), $relative);
                        let ids = direct_dependencies
                            .iter()
                            .map(|dependency| dependency.artifact_id.0.as_str())
                            .collect::<BTreeSet<_>>();
                        assert_eq!(ids.len(), direct_dependencies.len(), $relative);
                    }
                    (ArtifactLineageState::LegacyUnknown { reason, .. }, false) => {
                        assert_eq!(
                            format!("{reason:?}"),
                            "FieldAbsentInLegacyArtifact",
                            $relative
                        );
                    }
                    (actual, expected) => {
                        panic!("{}: lineage known={expected}, got {actual:?}", $relative)
                    }
                }
                let wire = serde_json::to_value(&artifact).expect("typed artifact wire");
                if $provenance {
                    exact_keys(
                        &wire["provenance"],
                        &[
                            "software_version",
                            "input_path",
                            "input_sha256",
                            "configuration_path",
                            "configuration_sha256",
                            "generation_timestamp",
                            "git_commit",
                        ],
                    );
                    assert!(
                        !wire["provenance"]["software_version"]
                            .as_str()
                            .unwrap()
                            .is_empty(),
                        $relative
                    );
                    assert!(
                        !wire["provenance"]["input_path"]
                            .as_str()
                            .unwrap()
                            .is_empty(),
                        $relative
                    );
                    assert!(
                        !wire["provenance"]["input_sha256"]
                            .as_str()
                            .unwrap()
                            .is_empty(),
                        $relative
                    );
                } else {
                    assert!(wire.get("provenance").is_none(), $relative);
                }
                artifact
            }};
        }

        let base_mechanism = accept!(
            MechanismAnalysisReport,
            "base/mechanism.json",
            4,
            true,
            true
        );
        let base_health = accept!(SensorHealthAssessment, "base/health.json", 4, true, true);
        let _base_eis = accept!(EisFitArtifact, "base/eis.json", 3, true, true);
        let _base_transient = accept!(
            TransientAnalysisReport,
            "base/transient.json",
            3,
            true,
            true
        );
        let _base_observations = accept!(
            CalibrationObservationSet,
            "base/calibration_observations.json",
            3,
            true,
            true
        );
        let _base_estimation =
            accept!(StateEstimationReport, "base/estimation.json", 4, true, true);
        let _base_calibration = accept!(
            CalibrationAnalysisReport,
            "base/calibration.json",
            3,
            true,
            true
        );
        let _base_signal = accept!(SignalAnalysisReport, "base/signal.json", 3, true, true);
        let _base_model = accept!(ModelAnalysisReport, "base/model.json", 5, false, false);
        let catalog =
            read_artifact_lineage_catalog(&fixture("base/lineage_catalog.json")).expect("N-F10");
        valid_reader_count += 1;
        assert_eq!(catalog.schema_version, 1);
        assert_eq!(catalog.artifacts.len(), 6);
        let _legacy_health = accept!(
            SensorHealthAssessment,
            "legacy/health_v3.json",
            3,
            false,
            true
        );
        let _legacy_mechanism = accept!(
            MechanismAnalysisReport,
            "legacy/mechanism_v1.json",
            1,
            false,
            true
        );
        let _health_mismatch = accept!(
            SensorHealthAssessment,
            "compat/health_sensor_mismatch.json",
            4,
            true,
            true
        );
        let _mechanism_mismatch = accept!(
            MechanismAnalysisReport,
            "compat/mechanism_experiment_mismatch.json",
            4,
            true,
            true
        );
        let _health_unknown = accept!(
            SensorHealthAssessment,
            "compat/health_unknown_scope.json",
            4,
            true,
            true
        );
        let _mechanism_unknown = accept!(
            MechanismAnalysisReport,
            "compat/mechanism_unknown_scope.json",
            4,
            true,
            true
        );
        let _eis_mismatch = accept!(
            EisFitArtifact,
            "compat/eis_sensor_mismatch.json",
            3,
            true,
            true
        );
        let _health_family = accept!(
            SensorHealthAssessment,
            "compat/health_family.json",
            4,
            true,
            true
        );
        let _mechanism_family = accept!(
            MechanismAnalysisReport,
            "compat/mechanism_family.json",
            4,
            true,
            true
        );
        let _missing_unit = accept!(
            SensorHealthAssessment,
            "health/missing_unit.json",
            4,
            true,
            true
        );
        let comparable = accept!(
            SensorHealthAssessment,
            "health/comparable_with_warnings.json",
            4,
            true,
            true
        );
        let bode = accept!(EisFitArtifact, "eis/nyquist_bode.json", 3, true, true);
        let zero = accept!(
            TransientAnalysisReport,
            "transient/zero_selected_fit.json",
            3,
            true,
            true
        );
        let duplicate = accept!(
            TransientAnalysisReport,
            "transient/duplicate_selected_fit.json",
            3,
            true,
            true
        );
        let missing_model = accept!(
            ModelAnalysisReport,
            "model/missing_values.json",
            5,
            false,
            false
        );
        let large_mechanism = accept!(
            MechanismAnalysisReport,
            "scale/mechanism_large_history.json",
            4,
            true,
            true
        );
        let large_health = accept!(
            SensorHealthAssessment,
            "scale/health_large_evidence.json",
            4,
            true,
            true
        );
        let stored_comparison = accept!(
            MechanismAnalysisReport,
            "mechanism/timescale_cmp01.json",
            4,
            true,
            true
        );
        assert_eq!(valid_reader_count, 28);

        assert_eq!(base_mechanism.analysis_id, "mechanism-phase-b:b-e2e-1");
        assert_eq!(base_health.assessment_id, "health:signal:a0-test:E1");
        assert_eq!(
            comparable
                .phase_c
                .as_ref()
                .unwrap()
                .dimension_assessments
                .len(),
            9
        );
        assert_eq!(
            comparable.baseline_comparison[0].current_value,
            Some(0.21472615802499273)
        );
        assert_eq!(
            comparable.baseline_comparison[0].baseline_value,
            Some(0.058)
        );
        assert_eq!(bode.source.z_imag_ohm, [-2.0, -1.0]);
        assert_eq!(bode.fitted.z_imag_ohm, [-1.5, -0.5]);
        assert_eq!(
            zero.events
                .iter()
                .flat_map(|event| event.candidate_fits.iter().filter(move |fit| {
                    Some(fit.model) == event.selected_model
                        && format!("{:?}", fit.status) == "Converged"
                }))
                .count(),
            0
        );
        assert!(
            duplicate
                .events
                .iter()
                .flat_map(|event| event.candidate_fits.iter().filter(move |fit| {
                    Some(fit.model) == event.selected_model
                        && format!("{:?}", fit.status) == "Converged"
                }))
                .count()
                > 1
        );
        assert_eq!(missing_model.points[0].observed_voltage_v, None);
        assert_eq!(missing_model.points[0].unexplained_residual_v, None);
        assert_eq!(stored_comparison.comparisons[0].log10_distance, Some(0.041));
        assert_eq!(large_mechanism.hypothesis_history.len(), 1000);
        assert_eq!(
            large_mechanism.hypothesis_history[0].history_id,
            "history-0000"
        );
        assert_eq!(
            large_mechanism.hypothesis_history[999].history_id,
            "history-0999"
        );
        assert_eq!(
            large_health
                .phase_c
                .as_ref()
                .expect("phase c")
                .evidence_bundle
                .records
                .len(),
            10_000
        );
        assert_eq!(
            large_health
                .phase_c
                .as_ref()
                .unwrap()
                .dimension_assessments
                .len(),
            9
        );
        assert_eq!(large_health.provenance.software_version, "a0-test");
        assert_eq!(
            large_mechanism
                .provenance
                .as_ref()
                .unwrap()
                .software_version,
            "phase-b-fixture-generator"
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

        let mut wrong_kind = compact_options("ledger-wrong-kind");
        wrong_kind.mechanism = fixture("failure/wrong_kind.json");
        assert!(matches!(
            report::render(&wrong_kind),
            Err(PublicReportError::Artifact {
                flag: "--mechanism",
                source: rust_electroanalysis_cli::domain::ArtifactError::IncompatibleKind { .. },
                ..
            })
        ));
        assert!(!wrong_kind.output_dir.exists());
        let mut old_eis = compact_options("ledger-old-eis");
        old_eis.eis = Some(fixture("failure/eis_schema2.json"));
        assert!(matches!(
            report::render(&old_eis),
            Err(PublicReportError::Artifact {
                flag: "--eis",
                source: rust_electroanalysis_cli::domain::ArtifactError::UnsupportedSchemaVersion {
                    actual: 2,
                    ..
                },
                ..
            })
        ));
        assert!(!old_eis.output_dir.exists());

        let schema1_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/a0_artifact_contracts/schema1");
        let mut old_signal = compact_options("ledger-old-signal");
        old_signal.signal = Some(schema1_root.join("signal_analysis.schema1.json"));
        assert!(matches!(
            report::render(&old_signal),
            Err(PublicReportError::Artifact {
                flag: "--signal",
                source: rust_electroanalysis_cli::domain::ArtifactError::UnsupportedSchemaVersion {
                    actual: 1,
                    ..
                },
                ..
            })
        ));
        assert!(!old_signal.output_dir.exists());
        let mut old_calibration = compact_options("ledger-old-calibration");
        old_calibration.calibration = Some(schema1_root.join("calibration_analysis.schema1.json"));
        old_calibration.calibration_observations =
            Some(schema1_root.join("calibration_observations.schema1.json"));
        assert!(matches!(
            report::render(&old_calibration),
            Err(PublicReportError::Artifact {
                flag: "--calibration",
                source: rust_electroanalysis_cli::domain::ArtifactError::UnsupportedSchemaVersion {
                    actual: 1,
                    ..
                },
                ..
            })
        ));
        assert!(!old_calibration.output_dir.exists());

        for (label, mechanism, health) in [
            ("ledger-base", "base/mechanism.json", "base/health.json"),
            (
                "ledger-legacy",
                "legacy/mechanism_v1.json",
                "legacy/health_v3.json",
            ),
            (
                "ledger-unknown",
                "compat/mechanism_unknown_scope.json",
                "compat/health_unknown_scope.json",
            ),
            (
                "ledger-families",
                "compat/mechanism_family.json",
                "compat/health_family.json",
            ),
            (
                "ledger-large",
                "scale/mechanism_large_history.json",
                "scale/health_large_evidence.json",
            ),
        ] {
            let mut options = compact_options(label);
            options.mechanism = fixture(mechanism);
            options.health = fixture(health);
            options.lineage_catalog = None;
            let published = render(&options);
            cleanup(&published);
        }
        let mut sensor_mismatch = compact_options("ledger-sensor-mismatch");
        sensor_mismatch.health = fixture("compat/health_sensor_mismatch.json");
        assert!(matches!(
            report::render(&sensor_mismatch),
            Err(PublicReportError::RequiredInputsIncompatible {
                axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::SensorScope,
                ..
            })
        ));
        let mut experiment_mismatch = compact_options("ledger-experiment-mismatch");
        experiment_mismatch.mechanism = fixture("compat/mechanism_experiment_mismatch.json");
        assert!(matches!(
            report::render(&experiment_mismatch),
            Err(PublicReportError::RequiredInputsIncompatible {
                axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::ExperimentScope,
                ..
            })
        ));
        let mut optional_mismatch = compact_options("ledger-optional-mismatch");
        optional_mismatch.eis = Some(fixture("compat/eis_sensor_mismatch.json"));
        assert!(matches!(
            report::render(&optional_mismatch),
            Err(PublicReportError::OptionalInputIncompatible {
                flag: "--eis",
                axis: rust_electroanalysis_cli::reporting::CompatibilityAxis::SensorScope,
                ..
            })
        ));
    }
);
