use rust_electroanalysis_cli::{
    cli::{CommandSpec, parse_cli_args},
    model::{InputValue, ModelInput},
    results::ModelAnalysisReport,
    runners::model,
};
use std::{
    collections::BTreeMap,
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn workspace() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "ism_model_phase06_{}_{}",
        std::process::id(),
        nonce
    ));
    fs::create_dir_all(&path).expect("workspace");
    path
}
#[test]
fn cli_parses_model_commands_and_preserves_estimate_command() {
    let parsed = parse_cli_args(&[
        "electroanalysis".into(),
        "model".into(),
        "simulate".into(),
        "--steps".into(),
        "3".into(),
    ])
    .expect("model CLI");
    assert!(matches!(
        parsed.command,
        Some(CommandSpec::ModelSimulate { steps: 3, .. })
    ));
    let estimate = parse_cli_args(&[
        "electroanalysis".into(),
        "estimate".into(),
        "simulate".into(),
    ])
    .expect("legacy command");
    assert!(matches!(
        estimate.command,
        Some(CommandSpec::EstimateSimulate { .. })
    ));
}
#[test]
fn validate_simulate_decompose_and_report_generate_finite_artifacts() {
    let root = workspace();
    let out = root.join("out");
    model::validate(&root, None, Some(&out)).expect("validate");
    assert!(out.join("model_definition_resolved.json").exists());
    model::simulate(&root, None, Some(&out), 3, 1.0).expect("simulate");
    let json = fs::read_to_string(out.join("model_analysis.json")).expect("analysis");
    assert!(!json.contains("NaN") && !json.contains("Infinity"));
    let report: ModelAnalysisReport = serde_json::from_str(&json).expect("schema compatibility");
    assert_eq!(report.points.len(), 3);
    let mut values = BTreeMap::new();
    values.insert(
        "primary_concentration".into(),
        InputValue {
            value: 1e-3,
            unit: "mol/L".into(),
        },
    );
    values.insert(
        "temperature".into(),
        InputValue {
            value: 298.15,
            unit: "K".into(),
        },
    );
    values.insert(
        "driving_step_v".into(),
        InputValue {
            value: 0.01,
            unit: "V".into(),
        },
    );
    let input = root.join("inputs.json");
    fs::write(
        &input,
        serde_json::to_string(&vec![ModelInput {
            time_s: 0.0,
            values,
        }])
        .expect("serialize"),
    )
    .expect("write");
    model::decompose(
        &root,
        None,
        Some(&input),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(&out),
    )
    .expect("decompose");
    for file in [
        "model_states.csv",
        "model_contributions.csv",
        "model_equilibrium.csv",
        "model_validity.csv",
        "model_evidence.json",
        "model_report.txt",
    ] {
        assert!(out.join(file).exists(), "{file}");
    }
    model::report(
        &root,
        &out.join("model_analysis.json"),
        Some(&out.join("recreated.txt")),
    )
    .expect("report");
    fs::remove_dir_all(root).ok();
}
#[test]
fn invalid_model_configuration_is_rejected() {
    let root = workspace();
    let invalid = root.join("invalid.toml");
    fs::write(&invalid, "schema_version = 99\n[model]\nschema_version = 1\nmodel_id = 'x'\ndescription = 'x'\nvalidity_domain = 'x'\nstates=[]\nparameters=[]\ninputs=[]\ncomponents=[]\n").expect("write");
    assert!(model::validate(&root, Some(&invalid), None).is_err());
    fs::remove_dir_all(root).ok();
}
