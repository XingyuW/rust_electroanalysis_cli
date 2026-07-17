use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn test_workspace(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rust_electroanalysis_phase0_{name}_{}_{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir_all(root.join("config")).expect("create config directory");
    fs::create_dir_all(root.join("data")).expect("create data directory");
    fs::create_dir_all(root.join("output")).expect("create output directory");

    fs::write(
        root.join("config/plotting.toml"),
        r#"
schema_version = 1

[shared]
input_path = "../data"
output_path = "../output"
input_is_directory = true
output_prefix = ""
"#,
    )
    .expect("write plotting config");

    fs::write(
        root.join("config/analysis.toml"),
        r#"
schema_version = 1
max_ranked_results = 1

[evolution]
population_size = 8
generation_limit = 1
num_individuals_per_parents = 2
selection_ratio = 0.7
mutation_rate = 0.2
reinsertion_ratio = 0.75

"#,
    )
    .expect("write analysis config");

    fs::write(
        root.join("config/parsing.toml"),
        r#"
schema_version = 1
fallback_model = "R0-p(CPE1,R1)"
"#,
    )
    .expect("write parsing config");

    fs::write(root.join("data/sample.csv"), eis_fixture()).expect("write EIS fixture");
    root
}

fn eis_fixture() -> &'static str {
    r#"Mar. 12, 2026   15:48:13
A.C. Impedance
File: sample.csv
Data Source:  Experiment
Instrument Model:  CHI760F
Header:
Note:

Freq/Hz, Z'/ohm, Z"/ohm, Z/ohm, Phase/deg

1000, 10.0, -1.0, 10.05, -5.7106
300, 12.0, -5.0, 13.0, -22.6199
100, 18.0, -10.0, 20.5913, -29.0546
30, 25.0, -9.0, 26.5707, -19.7989
10, 30.0, -5.0, 30.4138, -9.4623
"#
}

fn run_binary(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rust_electroanalysis_cli"))
        .args(args)
        .current_dir(root)
        .output()
        .expect("run electroanalysis binary")
}

#[test]
fn structured_and_legacy_plot_commands_still_render_eis_outputs() {
    let root = test_workspace("plot");
    let legacy = run_binary(&root, &["--plot", "eis"]);
    assert!(
        legacy.status.success(),
        "legacy plot failed: {}",
        String::from_utf8_lossy(&legacy.stderr)
    );

    let structured = run_binary(&root, &["plot", "eis"]);
    assert!(
        structured.status.success(),
        "structured plot failed: {}",
        String::from_utf8_lossy(&structured.stderr)
    );

    let output_names = fs::read_dir(root.join("output"))
        .expect("read plot output")
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(output_names.iter().any(|name| name.ends_with(".svg")));
    assert!(output_names.iter().any(|name| name.ends_with(".png")));
    assert!(output_names.iter().any(|name| name.contains("fit_report")));

    fs::remove_dir_all(root).ok();
}

#[test]
fn structured_and_legacy_search_commands_still_export_reports() {
    let root = test_workspace("search");
    let structured = run_binary(
        &root,
        &[
            "eis",
            "search",
            "data/sample.csv",
            "--search-output",
            "output/structured.txt",
            "--search-top",
            "1",
        ],
    );
    assert!(
        structured.status.success(),
        "structured search failed: {}",
        String::from_utf8_lossy(&structured.stderr)
    );
    assert!(root.join("output/structured.txt").is_file());
    assert!(root.join("output/structured.csv").is_file());

    let legacy = run_binary(
        &root,
        &[
            "--search-eis",
            "data/sample.csv",
            "--search-output",
            "output/legacy.txt",
            "--search-top",
            "1",
        ],
    );
    assert!(
        legacy.status.success(),
        "legacy search failed: {}",
        String::from_utf8_lossy(&legacy.stderr)
    );
    assert!(root.join("output/legacy.txt").is_file());
    assert!(root.join("output/legacy.csv").is_file());

    fs::remove_dir_all(root).ok();
}

#[test]
fn structured_fit_command_writes_named_report() {
    let root = test_workspace("fit");
    let output = run_binary(
        &root,
        &[
            "eis",
            "fit",
            "data/sample.csv",
            "--output",
            "output/fit.txt",
        ],
    );
    assert!(
        output.status.success(),
        "structured fit failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = fs::read_to_string(root.join("output/fit.txt")).expect("read fit report");
    assert!(report.contains("Parameters:"));
    assert!(report.contains("Fitted real impedance: available"));

    fs::remove_dir_all(root).ok();
}

#[test]
fn invalid_command_combination_fails_clearly() {
    let root = test_workspace("invalid");
    let output = run_binary(&root, &["plot", "eis", "--search-eis", "data/sample.csv"]);
    assert!(!output.status.success());
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostics.contains("unexpected argument '--search-eis'"));

    fs::remove_dir_all(root).ok();
}
