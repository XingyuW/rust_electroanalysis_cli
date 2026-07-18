use crate::runners::RunnerError;
use crate::{
    domain::{AnalysisProvenance, ExperimentEvent},
    results::{EisFitArtifact, ResidualAnalysisResult},
    signal::{self},
    signal_config::LoadedSignalConfig,
};
use serde::de::DeserializeOwned;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn characterize(
    workspace: &Path,
    input: &Path,
    metadata: Option<&Path>,
    channel: &str,
    sheet: Option<&str>,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let input = resolve(workspace, input);
    let loaded = LoadedSignalConfig::load(workspace, config_path)?;
    for w in &loaded.warnings {
        eprintln!("Warning: {w}");
    }
    let parsed =
        crate::data_file::measurement_parser::parse_measurement_file_with_sheet(&input, sheet)?;
    let provenance = AnalysisProvenance::from_paths(&input, loaded.source_path.as_deref())
        .map_err(crate::domain::DataParsingError::from)?;
    let (events, experiment_id, sensor_id) = if let Some(m) = metadata {
        let m = resolve(workspace, m);
        let doc = crate::domain::load_experiment_metadata(&m)?;
        (doc.events, Some(doc.experiment_id), doc.sensor.sensor_id)
    } else {
        (Vec::<ExperimentEvent>::new(), None, None)
    };
    let mut report = signal::analyze_measurement(
        &parsed.measurement,
        channel,
        Some(&events),
        &loaded.config,
        Some(provenance),
    )?;
    report.experiment_id = experiment_id;
    report.sensor_id = report.sensor_id.or(sensor_id);
    export_signal(workspace, output, &report)
}
pub fn compare(
    workspace: &Path,
    manifest: &Path,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedSignalConfig::load(workspace, config_path)?;
    let path = resolve(workspace, manifest);
    let man = crate::signal::comparison::load_manifest(&path).map_err(RunnerError::Message)?;
    if man.schema_version != 1 {
        return Err(RunnerError::Message(
            "unsupported signal comparison schema".into(),
        ));
    }
    let base = path.parent().unwrap_or(workspace);
    let (records, provenance) = crate::signal::comparison::compare(base, &man, &loaded.config)
        .map_err(RunnerError::Message)?;
    let dir = output_dir(workspace, output, "signal_comparison");
    fs::create_dir_all(&dir)?;
    write_json(&dir.join("signal_comparison_results.json"), &records)?;
    let mut w = csv::Writer::from_path(dir.join("signal_comparison.csv"))?;
    w.write_record([
        "record_id",
        "category",
        "channel",
        "count",
        "mean",
        "standard_deviation",
        "robust_standard_deviation",
        "drift_slope_v_per_s",
        "spike_fraction",
    ])?;
    for r in &records {
        w.write_record([
            r.record_id.clone(),
            r.category.clone(),
            r.channel.clone(),
            r.count.to_string(),
            fmt(r.mean),
            fmt(r.standard_deviation),
            fmt(r.robust_standard_deviation),
            fmt(r.drift_slope_v_per_s),
            fmt(r.spike_fraction),
        ])?;
    }
    write_json(&dir.join("signal_comparison_provenance.json"), &provenance)?;
    println!("Signal comparison written to {}", dir.display());
    Ok(())
}
pub fn residuals(
    workspace: &Path,
    transient: Option<&Path>,
    calibration: Option<&Path>,
    eis: Option<&Path>,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedSignalConfig::load(workspace, config_path)?;
    let mut results = Vec::new();
    if let Some(path) = transient {
        let p = resolve(workspace, path);
        let r: crate::results::TransientAnalysisReport = read_json(&p)?;
        for e in &r.events {
            if let Some(model) = e.selected_model
                && let Some(fit) = e
                    .candidate_fits
                    .iter()
                    .find(|f| f.model == model && f.is_successful())
            {
                let time = &e.segment.fitted_time_local;
                let s = crate::signal::residuals::time_summary(
                    time,
                    &fit.residuals_v,
                    loaded.config.psd.enabled.then_some(&loaded.config.psd),
                    &loaded.config.spikes,
                );
                results.push(ResidualAnalysisResult::TimeDomain {
                    source: format!("transient:{}", e.event_index),
                    summary: s,
                });
            }
        }
    }
    if let Some(path) = calibration {
        let p = resolve(workspace, path);
        let r: crate::results::CalibrationAnalysisReport = read_json(&p)?;
        if let Some(model) = r
            .selected_model
            .and_then(|k| r.candidate_models.iter().find(|m| m.model_kind == k))
        {
            let time = (0..model.residuals_v.len())
                .map(|i| i as f64)
                .collect::<Vec<_>>();
            let s = crate::signal::residuals::time_summary(
                &time,
                &model.residuals_v,
                Some(&loaded.config.psd),
                &loaded.config.spikes,
            );
            results.push(ResidualAnalysisResult::TimeDomain {
                source: "calibration".into(),
                summary: s,
            });
        }
    }
    if let Some(path) = eis {
        let p = resolve(workspace, path);
        let r: EisFitArtifact = read_json(&p)?;
        results.push(ResidualAnalysisResult::Eis {
            source: "eis".into(),
            summary: crate::signal::residuals::eis_summary(&r),
        });
    }
    let dir = output_dir(workspace, output, "residual_analysis");
    fs::create_dir_all(&dir)?;
    write_json(&dir.join("residual_analysis_results.json"), &results)?;
    fs::write(
        dir.join("residual_analysis_report.txt"),
        format!(
            "Residual analysis\n=================\nArtifacts analyzed: {}\n",
            results.len()
        ),
    )?;
    println!("Residual analysis written to {}", dir.display());
    Ok(())
}
fn export_signal(
    workspace: &Path,
    output: Option<&Path>,
    report: &crate::results::SignalAnalysisReport,
) -> Result<(), RunnerError> {
    let dir = output_dir(workspace, output, "signal");
    fs::create_dir_all(&dir)?;
    let c = &report.configuration.export;
    write_json(&dir.join(&c.results_filename), report)?;
    let mut w = csv::Writer::from_path(dir.join(&c.summary_filename))?;
    w.write_record(["feature", "value", "unit"])?;
    for (name, value, unit) in [
        ("mean", report.descriptive.mean, report.unit.clone()),
        ("median", report.descriptive.median, report.unit.clone()),
        (
            "standard_deviation",
            report.descriptive.standard_deviation,
            report.unit.clone(),
        ),
        ("rms", report.descriptive.rms, report.unit.clone()),
        (
            "robust_standard_deviation",
            report.descriptive.robust_standard_deviation,
            report.unit.clone(),
        ),
        (
            "missing_fraction",
            report.sampling.missing_fraction,
            "fraction".into(),
        ),
        (
            "spike_fraction",
            report.spikes.flagged_fraction,
            "fraction".into(),
        ),
    ] {
        w.write_record([name.to_string(), fmt(value), unit])?;
    }
    let mut p = csv::Writer::from_path(dir.join(&c.psd_filename))?;
    p.write_record(["frequency_hz", "psd", "asd"])?;
    if let Some(psd) = &report.psd {
        for i in 0..psd.frequency_hz.len() {
            p.write_record([
                fmt(Some(psd.frequency_hz[i])),
                fmt(psd.psd.get(i).copied()),
                fmt(psd.amplitude_spectral_density.get(i).copied()),
            ])?;
        }
    }
    let mut a = csv::Writer::from_path(dir.join(&c.allan_filename))?;
    a.write_record([
        "averaging_time_s",
        "allan_deviation",
        "effective_differences",
    ])?;
    if let Some(allan) = &report.allan {
        for x in &allan.points {
            a.write_record([
                fmt(Some(x.averaging_time_s)),
                fmt(x.deviation),
                x.effective_differences.to_string(),
            ])?;
        }
    }
    let mut d = csv::Writer::from_path(dir.join(&c.drift_filename))?;
    d.write_record([
        "model",
        "slope_v_per_s",
        "slope_mv_per_h",
        "slope_mv_per_day",
    ])?;
    for x in &report.drift {
        d.write_record([
            format!("{:?}", x.model),
            fmt(x.slope_v_per_s),
            fmt(x.slope_mv_per_h),
            fmt(x.slope_mv_per_day),
        ])?;
    }
    let mut s = csv::Writer::from_path(dir.join(&c.spikes_filename))?;
    s.write_record([
        "index",
        "timestamp_s",
        "value",
        "normalized_deviation",
        "sustained_step",
    ])?;
    for x in &report.spikes.flagged {
        s.write_record([
            x.index.to_string(),
            x.timestamp_s.to_string(),
            x.value.to_string(),
            fmt(x.normalized_deviation),
            x.sustained_step.to_string(),
        ])?;
    }
    let mut co = csv::Writer::from_path(dir.join(&c.correlations_filename))?;
    co.write_record([
        "channel_a",
        "channel_b",
        "observations",
        "pearson",
        "spearman",
        "lag_of_max_absolute_correlation_s",
    ])?;
    for x in &report.correlations {
        co.write_record([
            x.channel_a.clone(),
            x.channel_b.clone(),
            x.observations.to_string(),
            fmt(x.pearson),
            fmt(x.spearman),
            fmt(x.lag_of_max_absolute_correlation_s),
        ])?;
    }
    fs::write(dir.join(&c.report_filename), human_report(report))?;
    if report.configuration.plotting.enabled {
        crate::plottings::signal_plot::plot_signal_report(report, &dir)?;
    }
    println!("Signal analysis written to {}", dir.display());
    Ok(())
}
fn human_report(r: &crate::results::SignalAnalysisReport) -> String {
    format!(
        "Signal analysis\n================\nChannel: {} [{}]\nObservations: {}\nMean: {}\nRMS: {}\nRobust standard deviation: {}\nDrift models: {}\nSpike fraction: {}\nWarnings: {:?}\n",
        r.channel,
        r.unit,
        r.sampling.sample_count,
        fmt(r.descriptive.mean),
        fmt(r.descriptive.rms),
        fmt(r.descriptive.robust_standard_deviation),
        r.drift.len(),
        fmt(r.spikes.flagged_fraction),
        r.warnings
    )
}
fn read_json<T: DeserializeOwned>(p: &Path) -> Result<T, RunnerError> {
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}
fn write_json<T: serde::Serialize>(p: &Path, v: &T) -> Result<(), RunnerError> {
    fs::write(p, serde_json::to_string_pretty(v)?)?;
    Ok(())
}
fn resolve(workspace: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        workspace.join(p)
    }
}
fn output_dir(workspace: &Path, p: Option<&Path>, default: &str) -> PathBuf {
    p.map(|x| resolve(workspace, x))
        .unwrap_or_else(|| workspace.join("output").join(default))
}
fn fmt(v: Option<f64>) -> String {
    v.filter(|x| x.is_finite())
        .map(|x| format!("{x:.12e}"))
        .unwrap_or_default()
}
