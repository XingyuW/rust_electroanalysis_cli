use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rust_electroanalysis_cli::cli::{CommandSpec, parse_cli_args};
use rust_electroanalysis_cli::domain::{
    AnalysisProvenance, ElectrochemicalExperiment, ExperimentEvent, ExperimentEventKind,
    MeasurementChannel, MultiChannelMeasurement, SensorMetadata,
};
use rust_electroanalysis_cli::potentiometry::transient::diagnostics::compute_statistics;
use rust_electroanalysis_cli::potentiometry::transient::models::TransientModelKind;
use rust_electroanalysis_cli::potentiometry::{TransientAnalysisOptions, analyze_experiment};
use rust_electroanalysis_cli::results::transient::FitStatus;
use rust_electroanalysis_cli::transient_config::{
    DuplicateTimestampPolicy, ResolvedTransientConfig, SelectionCriterion,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn provenance() -> AnalysisProvenance {
    AnalysisProvenance {
        software_version: "phase2-test".to_string(),
        input_path: PathBuf::from("synthetic.csv"),
        input_sha256: "synthetic".to_string(),
        configuration_path: Some(PathBuf::from("experiment.toml")),
        configuration_sha256: Some("metadata".to_string()),
        generation_timestamp: 1,
        git_commit: None,
    }
}

fn event(timestamp: f64) -> ExperimentEvent {
    ExperimentEvent {
        timestamp,
        kind: ExperimentEventKind::ConcentrationStep,
        value: Some(0.01),
        unit: Some("mol/L".to_string()),
        analyte: Some("K+".to_string()),
        annotation: None,
        metadata: None,
    }
}

fn experiment<F>(post_response: F) -> ElectrochemicalExperiment
where
    F: Fn(f64) -> f64,
{
    let mut time = Vec::new();
    let mut values = Vec::new();
    for index in -20..=120 {
        let local_time = index as f64;
        time.push(local_time);
        values.push(if local_time < 0.0 {
            Some(0.30)
        } else {
            Some(post_response(local_time))
        });
    }
    let measurement = MultiChannelMeasurement::new(
        time,
        vec![MeasurementChannel::from_values(
            "E1",
            "V",
            values.into_iter().map(Option::unwrap).collect(),
        )],
    )
    .expect("synthetic measurement");
    experiment_from_measurement(measurement, vec![event(0.0)])
}

fn experiment_from_measurement(
    measurement: MultiChannelMeasurement,
    events: Vec<ExperimentEvent>,
) -> ElectrochemicalExperiment {
    ElectrochemicalExperiment::new(
        "synthetic-experiment",
        SensorMetadata::default(),
        None,
        measurement,
        Vec::new(),
        events,
        "buffer",
        provenance(),
    )
    .expect("synthetic experiment")
}

fn config(model: TransientModelKind) -> ResolvedTransientConfig {
    let mut config = ResolvedTransientConfig::default();
    config.segmentation.minimum_points = 20;
    config.segmentation.minimum_duration_s = 20.0;
    config.segmentation.post_event_s = 120.0;
    config.segmentation.pre_event_s = 20.0;
    config.models.enabled = vec![model];
    config.uncertainty.bootstrap_iterations = 0;
    config.plotting.enabled = false;
    config.validation.maximum_tau_to_window_ratio = 100.0;
    config
}

fn analyze<F>(
    model: TransientModelKind,
    response: F,
) -> rust_electroanalysis_cli::results::transient::TransientFitResult
where
    F: Fn(f64) -> f64,
{
    let experiment = experiment(response);
    let options = TransientAnalysisOptions {
        event_kind: ExperimentEventKind::ConcentrationStep,
        event_index: None,
        config: config(model),
    };
    let report = analyze_experiment(&experiment, "E1/V", &options).expect("transient analysis");
    report.events[0].candidate_fits[0].clone()
}

// Synthetic-data tolerances are intentionally tied to the test condition:
// noise-free recovery uses sub-percent parameter error, while the moderate
// Gaussian-noise case permits ±3 s on a 12 s time constant. The double-model
// checks permit ±0.2 s for the 2 s fast component and ±2 s for the 35 s slow
// component; stretched-beta recovery permits ±0.05. These tolerances test
// numerical stability without requiring exact optimizer iteration paths.
#[test]
fn recovers_noise_free_single_exponential() {
    let fit = analyze(TransientModelKind::Single, |time| {
        0.20 + 0.10 * (-time / 12.0).exp()
    });
    assert_eq!(fit.status, FitStatus::Converged);
    let tau = fit.derived_features.tau_fast_s.expect("tau");
    assert!((tau - 12.0).abs() < 0.05, "tau={tau}");
    assert!((fit.derived_features.fitted_equilibrium_potential_v.unwrap() - 0.20).abs() < 1e-6);
    assert!(fit.statistics.rmse_v.unwrap() < 1e-7);
}

#[test]
fn recovers_double_exponential_with_ordered_timescales() {
    let fit = analyze(TransientModelKind::Double, |time| {
        0.20 + 0.07 * (-time / 2.0).exp() + 0.03 * (-time / 35.0).exp()
    });
    assert_eq!(fit.status, FitStatus::Converged);
    let fast = fit.derived_features.tau_fast_s.unwrap();
    let slow = fit.derived_features.tau_slow_s.unwrap();
    assert!(fast < slow);
    assert!((fast - 2.0).abs() < 0.2, "fast={fast}");
    assert!((slow - 35.0).abs() < 2.0, "slow={slow}");
}

#[test]
fn recovers_double_exponential_drift_and_stretched_response() {
    let drift = analyze(TransientModelKind::DoubleDrift, |time| {
        0.20 + 0.07 * (-time / 2.0).exp() + 0.03 * (-time / 35.0).exp() + 1e-5 * time
    });
    assert_eq!(drift.status, FitStatus::Converged);
    assert!((drift.derived_features.drift_rate_v_per_s.unwrap() - 1e-5).abs() < 2e-6);

    let stretched = analyze(TransientModelKind::Stretched, |time| {
        0.20 + 0.10 * (-(time / 20.0).powf(0.7)).exp()
    });
    assert_eq!(stretched.status, FitStatus::Converged);
    assert!((stretched.derived_features.tau_fast_s.unwrap() - 20.0).abs() < 1.0);
    assert!((stretched.derived_features.stretched_beta.unwrap() - 0.7).abs() < 0.05);
}

#[test]
fn model_comparison_reports_aic_and_bic_without_raw_rss_selection() {
    let experiment = experiment(|time| {
        0.20 + 0.07 * (-time / 2.0).exp() + 0.03 * (-time / 35.0).exp() + 5e-4 * time
    });
    let mut config = config(TransientModelKind::Double);
    config.models.enabled = vec![TransientModelKind::Double, TransientModelKind::DoubleDrift];
    config.selection.criterion = SelectionCriterion::Bic;
    let report = analyze_experiment(
        &experiment,
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config,
        },
    )
    .expect("model comparison");
    assert_eq!(report.events[0].candidate_fits.len(), 2);
    for fit in &report.events[0].candidate_fits {
        assert!(fit.statistics.aic.is_some());
        assert!(fit.statistics.bic.is_some());
        assert!(fit.statistics.rss.is_some());
    }
    assert!(report.events[0].selected_model.is_some());
}

#[test]
fn default_model_set_processes_all_candidates() {
    let mut config = config(TransientModelKind::Single);
    config.models.enabled = TransientModelKind::ALL.to_vec();
    let report = analyze_experiment(
        &experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp()),
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config,
        },
    )
    .expect("all models");
    assert_eq!(report.events[0].candidate_fits.len(), 4);
}

#[test]
fn single_exponential_tolerates_noise_irregular_sampling_and_missing_values() {
    let mut rng = StdRng::seed_from_u64(123);
    let mut time = (-20..0).map(f64::from).collect::<Vec<_>>();
    let mut values = vec![Some(0.30); 20];
    for index in 0..100 {
        let local_time = index as f64 + if index % 3 == 0 { 0.25 } else { 0.0 };
        time.push(local_time);
        let u1 = rng.r#gen::<f64>().max(1e-12);
        let u2 = rng.r#gen::<f64>();
        let noise = 0.001 * (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        let value = 0.20 + 0.10 * (-local_time / 12.0).exp() + noise;
        values.push(if index % 17 == 0 { None } else { Some(value) });
    }
    let measurement =
        MultiChannelMeasurement::new(time, vec![MeasurementChannel::new("E1", "V", values)])
            .expect("irregular synthetic measurement");
    let mut config = config(TransientModelKind::Single);
    config.segmentation.maximum_missing_fraction = 0.20;
    let report = analyze_experiment(
        &experiment_from_measurement(measurement, vec![event(0.0)]),
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config,
        },
    )
    .expect("noisy transient analysis");
    let result = &report.events[0].candidate_fits[0];
    assert_eq!(result.status, FitStatus::Converged);
    assert!(report.events[0].segment.irregular_sampling);
    assert!(report.events[0].segment.missing_observations > 0);
    assert!((result.derived_features.tau_fast_s.unwrap() - 12.0).abs() < 3.0);
}

#[test]
fn bootstrap_is_reproducible_and_counts_iterations() {
    let mut config = config(TransientModelKind::Single);
    config.uncertainty.bootstrap_iterations = 20;
    config.uncertainty.seed = 77;
    let experiment = experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp());
    let options = TransientAnalysisOptions {
        event_kind: ExperimentEventKind::ConcentrationStep,
        event_index: None,
        config: config.clone(),
    };
    let first = analyze_experiment(&experiment, "E1/V", &options).expect("first bootstrap");
    let second = analyze_experiment(&experiment, "E1/V", &options).expect("second bootstrap");
    assert_eq!(
        first.events[0].candidate_fits[0].confidence_intervals,
        second.events[0].candidate_fits[0].confidence_intervals
    );
    assert_eq!(
        first.events[0].candidate_fits[0].confidence_intervals[0].successful_iterations
            + first.events[0].candidate_fits[0].confidence_intervals[0].failed_iterations,
        20
    );
}

#[test]
fn reports_aicc_unavailable_for_small_sample() {
    let (statistics, warnings) = compute_statistics(
        &[1.0, 2.0, 3.0],
        &[1.0, 2.0, 3.0],
        3,
        SelectionCriterion::Aic,
    )
    .expect("statistics");
    assert!(statistics.aicc.is_none());
    assert!(!warnings.is_empty());
}

#[test]
fn missing_channel_and_no_events_are_explicit_errors() {
    let experiment = experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp());
    let options = TransientAnalysisOptions {
        event_kind: ExperimentEventKind::ConcentrationStep,
        event_index: None,
        config: config(TransientModelKind::Single),
    };
    let missing =
        analyze_experiment(&experiment, "missing", &options).expect_err("missing channel");
    assert!(missing.to_string().contains("does not exist"));

    let mut no_events = experiment.clone();
    no_events.events.clear();
    let no_events = analyze_experiment(&no_events, "E1/V", &options).expect_err("no events");
    assert!(no_events.to_string().contains("no eligible"));
}

#[test]
fn duplicate_policy_is_explicit_and_non_monotonic_rows_are_paired() {
    let mut time: Vec<f64> = vec![
        -2.0, -1.0, 0.0, 1.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0,
        14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0,
    ];
    let mut values = time
        .iter()
        .map(|time| {
            Some(if *time < 0.0 {
                0.3
            } else {
                0.2 + 0.1 * (-time / 12.0).exp()
            })
        })
        .collect::<Vec<_>>();
    time.swap(3, 4);
    values.swap(3, 4);
    let measurement = MultiChannelMeasurement {
        time,
        channels: vec![MeasurementChannel::new("E1", "V", values)],
    };
    let mut custom = config(TransientModelKind::Single);
    custom.segmentation.duplicate_timestamp_policy = DuplicateTimestampPolicy::Average;
    let experiment = ElectrochemicalExperiment::new(
        "duplicate",
        SensorMetadata::default(),
        None,
        measurement,
        Vec::new(),
        vec![event(0.0)],
        "buffer",
        provenance(),
    )
    .expect("measurement with duplicate timestamps is structurally valid");
    let report = analyze_experiment(
        &experiment,
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config: custom,
        },
    )
    .expect("duplicate averaging should continue");
    assert_eq!(report.events[0].segment.duplicate_timestamps, 1);
    assert!(report.events[0].segment.irregular_sampling);
}

#[test]
fn event_index_and_event_window_failures_are_recorded_per_event() {
    let mut events = vec![event(0.0), event(120.0), event(200.0)];
    events[1].value = Some(0.02);
    events[2].value = Some(0.03);
    let experiment = experiment_from_measurement(
        experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp()).measurement_data,
        events,
    );
    let mut config = config(TransientModelKind::Single);
    config.segmentation.post_event_s = 20.0;
    config.segmentation.minimum_points = 5;
    let report = analyze_experiment(
        &experiment,
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config: config.clone(),
        },
    )
    .expect("per-event failures should not abort the report");
    assert_eq!(report.events.len(), 3);
    assert!(report.events[1].failure.is_some());
    assert!(report.events[2].failure.is_some());

    let selected = analyze_experiment(
        &experiment,
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: Some(0),
            config,
        },
    )
    .expect("event index");
    assert_eq!(selected.events.len(), 1);
    assert_eq!(selected.events[0].event_index, 0);
}

#[test]
fn concentration_before_skips_compatible_events_without_values() {
    let mut missing_value = event(10.0);
    missing_value.value = None;
    let mut current = event(20.0);
    current.value = Some(0.03);
    let experiment = experiment_from_measurement(
        experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp()).measurement_data,
        vec![event(0.0), missing_value, current],
    );
    let report = analyze_experiment(
        &experiment,
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: Some(2),
            config: config(TransientModelKind::Single),
        },
    )
    .expect("eligible event");
    assert_eq!(report.events.len(), 1);
    assert_eq!(
        report.events[0]
            .concentration_before
            .as_ref()
            .map(|value| value.value),
        Some(0.01)
    );
}

#[test]
fn constant_signal_and_event_at_first_timestamp_are_reported_without_nonfinite_values() {
    let constant = analyze(TransientModelKind::Single, |_| 0.20);
    assert!(
        constant
            .predicted_v
            .iter()
            .chain(constant.residuals_v.iter())
            .all(|value| value.is_finite())
    );

    let mut first_event = event(-20.0);
    first_event.value = Some(0.02);
    let report = analyze_experiment(
        &experiment_from_measurement(
            experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp()).measurement_data,
            vec![first_event],
        ),
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config: config(TransientModelKind::Single),
        },
    )
    .expect("event at first timestamp should be retained as a result");
    assert_eq!(report.events.len(), 1);
    assert!(report.events[0].failure.is_none());
}

#[test]
fn too_short_segments_and_duplicate_error_policy_fail_per_event() {
    let mut short_config = config(TransientModelKind::Single);
    short_config.segmentation.minimum_duration_s = 500.0;
    let short = analyze_experiment(
        &experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp()),
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config: short_config,
        },
    )
    .expect("segment failure should remain per event");
    assert!(
        short.events[0]
            .failure
            .as_ref()
            .is_some_and(|failure| failure.message.contains("too short"))
    );

    let mut time = vec![-2.0, -1.0, 0.0, 1.0, 1.0];
    let mut values = time
        .iter()
        .map(|time| Some(if *time < 0.0 { 0.3 } else { 0.2 }))
        .collect::<Vec<_>>();
    for index in 2..=4 {
        time.push(index as f64);
        values.push(Some(0.2));
    }
    let duplicate_measurement =
        MultiChannelMeasurement::new(time, vec![MeasurementChannel::new("E1", "V", values)])
            .expect("duplicate timestamps are structurally valid");
    let duplicate = analyze_experiment(
        &experiment_from_measurement(duplicate_measurement, vec![event(0.0)]),
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config: config(TransientModelKind::Single),
        },
    )
    .expect("duplicate policy should be a per-event failure");
    assert!(
        duplicate.events[0]
            .failure
            .as_ref()
            .is_some_and(|failure| failure.message.contains("duplicate"))
    );
}

#[test]
fn high_missing_fraction_is_rejected_without_mutating_measurement() {
    let mut measurement = experiment(|time| 0.20 + 0.10 * (-time / 12.0).exp()).measurement_data;
    for value in measurement.channels[0]
        .values
        .iter_mut()
        .skip(20)
        .step_by(2)
    {
        *value = None;
    }
    let before = measurement.clone();
    let mut config = config(TransientModelKind::Single);
    config.segmentation.maximum_missing_fraction = 0.20;
    let report = analyze_experiment(
        &experiment_from_measurement(measurement.clone(), vec![event(0.0)]),
        "E1/V",
        &TransientAnalysisOptions {
            event_kind: ExperimentEventKind::ConcentrationStep,
            event_index: None,
            config,
        },
    )
    .expect("rejection should remain per-event");
    assert!(report.events[0].failure.is_some());
    assert_eq!(measurement, before);
}

#[test]
fn transient_cli_creates_machine_and_human_outputs() {
    let args = [
        "electroanalysis",
        "transient",
        "fit",
        "--input",
        "data/sensor.csv",
        "--metadata",
        "data/experiment.toml",
        "--channel",
        "E1/V",
        "--event-kind",
        "concentration-step",
        "--model",
        "single",
        "--selection",
        "bic",
        "--bootstrap",
        "0",
        "--seed",
        "11",
    ];
    let parsed = parse_cli_args(
        &args
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>(),
    )
    .expect("transient CLI should parse");
    assert!(matches!(
        parsed.command,
        Some(CommandSpec::TransientFit { .. })
    ));

    let root = fixture_workspace();
    let output = Command::new(env!("CARGO_BIN_EXE_rust_electroanalysis_cli"))
        .args([
            "transient",
            "fit",
            "--input",
            "data/sensor.csv",
            "--metadata",
            "data/experiment.toml",
            "--channel",
            "E1/V",
            "--config",
            "config/transient.toml",
            "--output",
            "output/transient",
            "--model",
            "single",
            "--selection",
            "bic",
            "--bootstrap",
            "0",
            "--seed",
            "11",
        ])
        .current_dir(&root)
        .output()
        .expect("run transient CLI");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for filename in [
        "transient_results.json",
        "transient_features.csv",
        "transient_model_comparison.csv",
        "transient_report.txt",
    ] {
        assert!(
            root.join("output/transient").join(filename).is_file(),
            "missing {filename}"
        );
    }
    let json =
        fs::read_to_string(root.join("output/transient/transient_results.json")).expect("JSON");
    assert!(!json.contains("NaN"));
    assert!(!json.contains("Infinity"));
    assert!(json.contains("\"criterion\": \"bic\""));
    fs::remove_dir_all(root).ok();
}

fn fixture_workspace() -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("phase2_cli_{}_{}", std::process::id(), suffix));
    fs::create_dir_all(root.join("config")).expect("config dir");
    fs::create_dir_all(root.join("data")).expect("data dir");
    let mut data = String::from("time/sec,E1/V\n");
    for time in 0..=100 {
        let potential = if time < 20 {
            0.30
        } else {
            0.20 + 0.10 * (-((time - 20) as f64) / 12.0).exp()
        };
        data.push_str(&format!("{time},{potential}\n"));
    }
    fs::write(root.join("data/sensor.csv"), data).expect("data");
    fs::write(
        root.join("data/experiment.toml"),
        "experiment_id = 'cli-test'\nsample_matrix = 'buffer'\n\n[sensor]\nsensor_id = 's1'\n\n[[events]]\ntimestamp = 20.0\nkind = 'concentration_step'\nvalue = 0.01\nunit = 'mol/L'\nanalyte = 'K+'\n",
    )
    .expect("metadata");
    fs::write(
        root.join("config/transient.toml"),
        "schema_version = 1\n[segmentation]\npre_event_s = 10.0\npost_event_s = 80.0\nminimum_points = 20\nminimum_duration_s = 10.0\n[models]\nenabled = ['single']\n[uncertainty]\nbootstrap_iterations = 0\n[plotting]\nenabled = false\n",
    )
    .expect("transient config");
    root
}
