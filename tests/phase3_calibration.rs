use rust_electroanalysis_cli::calibration_config::ResolvedCalibrationConfig;
use rust_electroanalysis_cli::cli::{CommandSpec, parse_cli_args};
use rust_electroanalysis_cli::domain::AnalysisProvenance;
use rust_electroanalysis_cli::potentiometry::calibration::{
    activity::evaluate_activity,
    fit_calibration,
    fitting::fit_model,
    nernst::{activity_from_potential, theoretical_slope_v_per_decade},
    nicolsky_eisenman::{InterferentModelInput, evaluate_potential},
    prediction::{predict_activity_from_potential, predict_potential_from_activity},
    stored_model_from_report,
};
use rust_electroanalysis_cli::potentiometry::units::{Quantity, QuantityUnit};
use rust_electroanalysis_cli::results::calibration::{
    ActivityModelKind, CalibrationBranch, CalibrationModelKind, CalibrationObservation,
    CalibrationObservationSet, CalibrationPotentialSource, CalibrationWarningKind,
    CrossValidationMode,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn provenance() -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "phase3-test".to_string(),
        input_path: PathBuf::from("synthetic.csv"),
        input_sha256: "synthetic".to_string(),
        configuration_path: None,
        configuration_sha256: None,
        generation_timestamp: 1,
        git_commit: None,
    }
}

fn observation(activity: f64, potential_v: f64) -> CalibrationObservation {
    CalibrationObservation {
        observation_id: format!("obs-{activity}"),
        experiment_id: "synthetic".to_string(),
        event_index: None,
        timestamp: None,
        analyte: "Na+".to_string(),
        ion_charge: 1,
        concentration: Some(Quantity::new(activity, QuantityUnit::MolPerL).unwrap()),
        molar_concentration_mol_l: Some(activity),
        activity: Some(activity),
        activity_coefficient: Some(1.0),
        potential_v,
        potential_standard_error_v: Some(0.001),
        temperature_k: Some(298.15),
        ionic_strength_mol_l: None,
        conductivity: None,
        interferent_activities: BTreeMap::new(),
        branch: CalibrationBranch::Ascending,
        source: CalibrationPotentialSource::ExplicitObservation,
        source_fit_status: None,
        source_warnings: Vec::new(),
        steady_state: None,
        environmental_alignment: Vec::new(),
        metadata: BTreeMap::new(),
    }
}

fn nernst_observations() -> Vec<CalibrationObservation> {
    let e0 = 0.20;
    let slope = theoretical_slope_v_per_decade(298.15, 1).unwrap();
    [1e-5, 2e-4, 1e-3, 2e-2, 0.2]
        .into_iter()
        .map(|activity| observation(activity, e0 + slope * activity.log10()))
        .collect()
}

fn config() -> ResolvedCalibrationConfig {
    let mut config = ResolvedCalibrationConfig::default();
    config.plotting.enabled = false;
    config.uncertainty.bootstrap_iterations = 24;
    config.validation.mode = CrossValidationMode::LeaveOneOut;
    config
}

#[test]
fn recovers_noise_free_nernst_intercept_and_slope() {
    let observations = nernst_observations();
    let result = fit_model(&observations, &config(), CalibrationModelKind::Nernst).unwrap();
    assert_eq!(
        result.status,
        rust_electroanalysis_cli::results::calibration::CalibrationFitStatus::Converged
    );
    assert!((result.parameters[0].value - 0.20).abs() < 1e-10);
    assert!(
        (result.fitted_slope_v_per_decade.unwrap()
            - theoretical_slope_v_per_decade(298.15, 1).unwrap())
        .abs()
            < 1e-10
    );
    assert!(result.statistics.aicc.is_some());
}

#[test]
fn uses_signed_charge_and_temperature_for_theoretical_slope() {
    let monovalent = theoretical_slope_v_per_decade(298.15, 1).unwrap();
    let divalent = theoretical_slope_v_per_decade(298.15, 2).unwrap();
    let warmer = theoretical_slope_v_per_decade(310.15, 1).unwrap();
    assert!((divalent.abs() - monovalent.abs() / 2.0).abs() < 1e-12);
    assert!(warmer > monovalent);
}

#[test]
fn activity_models_keep_concentration_and_activity_distinct() {
    let concentration = Quantity::new(18.0, QuantityUnit::MgPerL).unwrap();
    let config = rust_electroanalysis_cli::calibration_config::ActivityConfig {
        model: ActivityModelKind::Davies,
        ..Default::default()
    };
    let result = evaluate_activity(
        Some(&concentration),
        Some(18.0),
        None,
        None,
        1,
        Some(0.10),
        None,
        &config,
    )
    .unwrap();
    assert!(result.activity < 0.001);
    assert!(result.activity_coefficient.unwrap() < 1.0);
    assert!(result.warnings.is_empty());

    let molal = Quantity::new(0.01, QuantityUnit::MolPerKg).unwrap();
    let ideal = evaluate_activity(
        Some(&molal),
        None,
        None,
        None,
        1,
        None,
        None,
        &ResolvedCalibrationConfig::default().activity,
    )
    .unwrap();
    assert!((ideal.activity - 0.01).abs() < 1e-12);
}

#[test]
fn report_bootstrap_is_reproducible_and_prediction_warns_on_extrapolation() {
    let observation_set = CalibrationObservationSet {
        schema_version: 1,
        observations: nernst_observations(),
        provenance: provenance(),
        warnings: Vec::new(),
    };
    let report = fit_calibration(&observation_set, &config()).unwrap();
    let model = stored_model_from_report(&report).unwrap();
    let prediction =
        predict_potential_from_activity(&model, 1e-3, Some(298.15), &BTreeMap::new()).unwrap();
    assert!(
        (prediction.potential_v.unwrap()
            - (0.20 + theoretical_slope_v_per_decade(298.15, 1).unwrap() * -3.0))
            .abs()
            < 1e-7
    );
    let extrapolated =
        predict_activity_from_potential(&model, 1.0, Some(298.15), &BTreeMap::new()).unwrap();
    assert!(extrapolated.extrapolated);
    assert!(
        extrapolated
            .warnings
            .iter()
            .any(|warning| warning.kind == CalibrationWarningKind::PredictionExtrapolation)
    );
    let intervals = report.candidate_models[0].confidence_intervals.clone();
    assert!(!intervals.is_empty());
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("NaN"));
    assert!(!json.contains("Infinity"));
    assert!(!json.contains("-Infinity"));
}

#[test]
fn fit_calibration_handles_small_observation_sets_without_panicking() {
    let e0 = 0.20;
    let slope = theoretical_slope_v_per_decade(298.15, 1).unwrap();
    let observation_set = CalibrationObservationSet {
        schema_version: 1,
        observations: [1e-4, 1e-3, 1e-2]
            .into_iter()
            .map(|activity| observation(activity, e0 + slope * activity.log10()))
            .collect(),
        provenance: provenance(),
        warnings: Vec::new(),
    };
    let mut cfg = config();
    cfg.validation.mode = CrossValidationMode::None;

    let run = std::panic::catch_unwind(|| fit_calibration(&observation_set, &cfg));
    assert!(run.is_ok(), "fit_calibration panicked on a small dataset");
    assert!(run.unwrap().is_ok(), "fit_calibration returned an error");
}

#[test]
fn calibration_cli_parses_all_workflow_boundaries() {
    let args = |values: &[&str]| {
        let mut command = vec!["electroanalysis"];
        command.extend(values);
        parse_cli_args(
            &command
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .command
        .unwrap()
    };
    assert!(matches!(
        args(&[
            "calibration",
            "extract",
            "--input",
            "data.csv",
            "--metadata",
            "experiment.toml",
            "--channel",
            "E1/V"
        ]),
        CommandSpec::CalibrationExtract { .. }
    ));
    assert!(matches!(
        args(&["calibration", "fit", "--observations", "observations.json"]),
        CommandSpec::CalibrationFit { .. }
    ));
    assert!(matches!(
        args(&[
            "calibration",
            "validate",
            "--model",
            "model.json",
            "--observations",
            "validation.json"
        ]),
        CommandSpec::CalibrationValidate { .. }
    ));
    assert!(matches!(
        args(&[
            "calibration",
            "predict",
            "--model",
            "model.json",
            "--potential",
            "0.184",
            "--temperature",
            "25"
        ]),
        CommandSpec::CalibrationPredict {
            potential: Some(_),
            ..
        }
    ));
    let error = parse_cli_args(&[
        "electroanalysis".to_string(),
        "calibration".to_string(),
        "predict".to_string(),
        "--model".to_string(),
        "model.json".to_string(),
    ])
    .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires --potential or --input")
    );
}

#[test]
fn inverse_nernst_activity_is_finite_and_positive() {
    let slope = theoretical_slope_v_per_decade(298.15, 1).unwrap();
    let activity = activity_from_potential(0.20 + slope, 0.20, slope).unwrap();
    assert!((activity - 10.0).abs() < 1e-12);
}

#[test]
fn nicolsky_eisenman_fits_positive_selectivity_and_inverts_activity() {
    let mut config = config();
    config.uncertainty.bootstrap_iterations = 0;
    config.validation.mode = CrossValidationMode::None;
    config.models.enabled = vec![CalibrationModelKind::NicolskyEisenman];
    config.nicolsky_eisenman.enabled = true;
    config.nicolsky_eisenman.fit_selectivity_coefficients = true;
    config.nicolsky_eisenman.interferents = vec![
        rust_electroanalysis_cli::calibration_config::InterferentConfig {
            name: "K+".to_string(),
            charge: 1,
            selectivity_coefficient: Some(0.02),
            source: "user_supplied".to_string(),
        },
    ];
    let interferent_activity = 0.01;
    let observations = [1e-5, 1e-4, 1e-3, 1e-2, 1e-1]
        .into_iter()
        .map(|activity| {
            let potential = evaluate_potential(
                0.2,
                activity,
                1,
                298.15,
                &[InterferentModelInput {
                    name: "K+".to_string(),
                    charge: 1,
                    activity: interferent_activity,
                    selectivity_coefficient: 0.02,
                }],
            )
            .unwrap();
            let mut row = observation(activity, potential);
            row.interferent_activities
                .insert("K+".to_string(), interferent_activity);
            row
        })
        .collect::<Vec<_>>();
    let result = fit_model(
        &observations,
        &config,
        CalibrationModelKind::NicolskyEisenman,
    )
    .expect("Nicolsky-Eisenman fit");
    assert_eq!(
        result.status,
        rust_electroanalysis_cli::results::calibration::CalibrationFitStatus::Converged
    );
    let coefficient = result.selectivity_coefficients[0].value;
    assert!(coefficient > 0.0);
    assert!((coefficient - 0.02).abs() < 0.01, "Kpot={coefficient}");
}

#[test]
fn empirical_conductivity_correction_reports_empirical_label() {
    let mut config = config();
    config.uncertainty.bootstrap_iterations = 0;
    config.validation.mode = CrossValidationMode::None;
    config.models.enabled = vec![CalibrationModelKind::ConductivityEmpirical];
    config.activity.model = ActivityModelKind::ConductivityEmpirical;
    config.activity.conductivity_empirical.enabled = true;
    config.activity.conductivity_empirical.fit_b1 = true;
    let slope = theoretical_slope_v_per_decade(298.15, 1).unwrap();
    let conductivities = [0.5, 1.0, 1.8, 3.5, 8.0];
    let observations = [1e-5, 1e-4, 1e-3, 1e-2, 1e-1]
        .into_iter()
        .enumerate()
        .map(|(index, activity)| {
            let conductivity = conductivities[index];
            let mut row = observation(
                activity,
                0.2 + slope * (activity.log10() + 0.02 * conductivity),
            );
            row.conductivity =
                Some(Quantity::new(conductivity, QuantityUnit::SiemensPerM).unwrap());
            row
        })
        .collect::<Vec<_>>();
    let result = fit_model(
        &observations,
        &config,
        CalibrationModelKind::ConductivityEmpirical,
    )
    .expect("empirical conductivity fit");
    assert!(result.equation.contains("empirical"));
    assert!((result.parameters[2].value - slope * 0.02).abs() < 1e-8);
}
