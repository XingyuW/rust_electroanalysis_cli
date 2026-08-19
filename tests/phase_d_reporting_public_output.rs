use rust_electroanalysis_cli::cli::{CliError, parse_cli_args};
use rust_electroanalysis_cli::domain::{LineageCatalogReadError, read_artifact_lineage_catalog};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn fixture_path(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("rust-electroanalysis-phase-d-{name}-{nonce}.json"))
}

#[test]
fn phase_d_catalog_reader_rejects_syntactically_malformed_json() {
    let path = fixture_path("malformed");
    fs::write(&path, b"{not-json}\n").expect("fixture");
    let error = read_artifact_lineage_catalog(&path).expect_err("malformed catalog must fail");
    assert!(matches!(error, LineageCatalogReadError::Json { .. }));
    fs::remove_file(path).expect("cleanup");
}

#[test]
fn phase_d_catalog_reader_rejects_structurally_invalid_catalog() {
    let path = fixture_path("unknown-field");
    fs::write(
        &path,
        b"{\"schema_version\":1,\"artifacts\":{},\"unexpected\":true}\n",
    )
    .expect("fixture");
    let error =
        read_artifact_lineage_catalog(&path).expect_err("closed-schema violation must fail");
    match error {
        LineageCatalogReadError::UnknownField { field, .. } => assert_eq!(field, "unexpected"),
        other => panic!("expected UnknownField, got {other:?}"),
    }
    fs::remove_file(path).expect("cleanup");
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
