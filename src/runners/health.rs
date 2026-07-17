use crate::{
    health::{self, baseline::Context},
    health_config::LoadedHealthConfig,
    results::{HealthDomain, HealthWarning, SensorHealthAssessment, SensorHealthBaseline},
    runners::RunnerError,
};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Deserialize)]
pub struct HealthManifest {
    pub schema_version: u32,
    pub records: Vec<HealthManifestRecord>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct HealthManifestRecord {
    pub record_id: String,
    pub signal_results: PathBuf,
    #[serde(default)]
    pub transient_results: Option<PathBuf>,
    #[serde(default)]
    pub calibration_results: Option<PathBuf>,
    #[serde(default)]
    pub eis_fit: Option<PathBuf>,
    #[serde(default)]
    pub mechanism_results: Option<PathBuf>,
    #[serde(default)]
    pub metadata: Option<PathBuf>,
    #[serde(default)]
    pub independent_value: Option<f64>,
}
pub fn baseline(
    workspace: &Path,
    manifest: &Path,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedHealthConfig::load(workspace, config_path)?;
    let path = resolve(workspace, manifest);
    let man: HealthManifest = read_toml(&path)?;
    if man.schema_version != 1 {
        return Err(RunnerError::Message(
            "unsupported health manifest schema".into(),
        ));
    }
    let base = path.parent().unwrap_or(workspace);
    let mut records = Vec::new();
    let mut provenance = None;
    for r in &man.records {
        let signal_path = resolve(base, &r.signal_results);
        let signal: crate::results::SignalAnalysisReport = read_json(&signal_path)?;
        provenance.get_or_insert(signal.provenance.clone());
        let mut fs = health::features::from_signal(&signal);
        if let Some(p) = &r.transient_results {
            let t: crate::results::TransientAnalysisReport = read_json(&resolve(base, p))?;
            fs.extend(health::features::from_transient(&t));
        }
        if let Some(p) = &r.calibration_results {
            let c: crate::results::CalibrationAnalysisReport = read_json(&resolve(base, p))?;
            fs.extend(health::features::from_calibration(&c));
        }
        if let Some(p) = &r.eis_fit {
            let e: crate::results::EisFitArtifact = read_json(&resolve(base, p))?;
            fs.extend(health::features::from_eis(&e));
        }
        if let Some(p) = &r.mechanism_results {
            let m: crate::results::MechanismAnalysisReport = read_json(&resolve(base, p))?;
            fs.extend(health::features::from_mechanism(&m));
        }
        let context = r
            .metadata
            .as_ref()
            .map(|p| load_context(&resolve(base, p)))
            .transpose()?
            .unwrap_or_default();
        records.push((r.record_id.clone(), fs, context));
    }
    let provenance = provenance
        .ok_or_else(|| RunnerError::Message("health baseline manifest is empty".into()))?;
    let b = health::baseline::build_with_contexts(
        "health-baseline",
        &records,
        provenance,
        loaded.config.baseline.minimum_required_records,
    );
    let dest = output_file(workspace, output, &loaded.config.export.baseline_filename);
    write_json(&dest, &b)?;
    println!("Health baseline written to {}", dest.display());
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub fn assess(
    workspace: &Path,
    signal_path: &Path,
    transient: Option<&Path>,
    calibration: Option<&Path>,
    eis: Option<&Path>,
    mechanism: Option<&Path>,
    baseline_path: Option<&Path>,
    metadata: Option<&Path>,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedHealthConfig::load(workspace, config_path)?;
    let signal_path = resolve(workspace, signal_path);
    let signal: crate::results::SignalAnalysisReport = read_json(&signal_path)?;
    let mut features = health::features::from_signal(&signal);
    let mut missing = Vec::new();
    if let Some(p) = transient {
        let r: crate::results::TransientAnalysisReport = read_json(&resolve(workspace, p))?;
        features.extend(health::features::from_transient(&r));
    } else {
        missing.push(HealthDomain::DynamicResponse);
    }
    if let Some(p) = calibration {
        let r: crate::results::CalibrationAnalysisReport = read_json(&resolve(workspace, p))?;
        features.extend(health::features::from_calibration(&r));
    } else {
        missing.push(HealthDomain::Calibration);
    }
    if let Some(p) = eis {
        let r: crate::results::EisFitArtifact = read_json(&resolve(workspace, p))?;
        features.extend(health::features::from_eis(&r));
    } else {
        missing.push(HealthDomain::Impedance);
    }
    if let Some(p) = mechanism {
        let r: crate::results::MechanismAnalysisReport = read_json(&resolve(workspace, p))?;
        features.extend(health::features::from_mechanism(&r));
    } else {
        missing.push(HealthDomain::MechanismEvidence);
    }
    let mut warnings = signal
        .warnings
        .iter()
        .map(|_| HealthWarning::AssessmentBasedOnWarningBearingFits)
        .collect::<Vec<_>>();
    let base = baseline_path.map(|p| resolve(workspace, p));
    let baseline: Option<SensorHealthBaseline> = if let Some(p) = base.as_deref() {
        Some(read_json(p)?)
    } else {
        warnings.push(HealthWarning::MissingBaseline);
        None
    };
    let current_context = metadata
        .map(|p| load_context(&resolve(workspace, p)))
        .transpose()?
        .unwrap_or_default();
    let base_context = baseline
        .as_ref()
        .map(|b| Context {
            sensor_id: None,
            sensor_type: b.sensor_type.clone(),
            sensor_design: b.sensor_design.clone(),
            analyte: b.analyte.clone(),
            sample_matrix: b.sample_matrix.clone(),
            temperature_k: b.temperature_domain_k.map(|x| (x.0 + x.1) / 2.0),
            temperature_values_k: b
                .temperature_domain_k
                .map(|x| vec![x.0, x.1])
                .unwrap_or_default(),
            experiment_id: None,
            metadata_source: None,
        })
        .unwrap_or_default();
    let mut comparisons = Vec::new();
    for f in &features {
        let (cmp, reason) = if let Some(b) = &baseline {
            let bdist = b.feature_distributions.iter().find(|x| x.feature == f.name);
            let (c, r) = health::normalization::comparable(
                &current_context,
                &base_context,
                &loaded.config.comparability,
            );
            (
                c,
                r.or_else(|| {
                    bdist
                        .is_none()
                        .then_some("feature absent from baseline".into())
                }),
            )
        } else {
            (
                crate::results::FeatureComparability::Unknown,
                Some("baseline unavailable".into()),
            )
        };
        let dist = baseline
            .as_ref()
            .and_then(|b| b.feature_distributions.iter().find(|x| x.feature == f.name));
        let mut c = health::normalization::compare_with_config(
            f,
            dist,
            cmp,
            &loaded.config.normalization,
            None,
        );
        c.override_reason = reason;
        if matches!(cmp, crate::results::FeatureComparability::NotComparable) {
            warnings.push(HealthWarning::FeatureNoncomparable);
        }
        comparisons.push(c);
    }
    let (evaluations, findings) = health::rules::evaluate(
        &loaded.config.rules,
        &features,
        &comparisons,
        loaded
            .config
            .assessment
            .minimum_domains_for_mechanistic_finding,
    );
    let domains = features.iter().map(|f| f.domain).collect::<BTreeSet<_>>();
    if domains.len() < loaded.config.assessment.minimum_domains_for_assessment {
        warnings.push(HealthWarning::InsufficientEvidenceDomains);
    }
    let assessment = health::assessment::assemble(
        &format!("health:{}", signal.analysis_id),
        signal.sensor_id.clone(),
        signal.experiment_id.clone(),
        features,
        comparisons,
        evaluations,
        findings,
        missing,
        loaded.config,
        signal.provenance,
        warnings,
    );
    export_assessment(workspace, output, &assessment)
}
pub fn trend(
    workspace: &Path,
    manifest: &Path,
    baseline_path: Option<&Path>,
    config_path: Option<&Path>,
    output: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedHealthConfig::load(workspace, config_path)?;
    let path = resolve(workspace, manifest);
    let man: HealthTrendManifest = read_toml(&path)?;
    if man.schema_version != 1 {
        return Err(RunnerError::Message(
            "unsupported health trend manifest schema".into(),
        ));
    }
    let base = path.parent().unwrap_or(workspace);
    let baseline = baseline_path
        .map(|p| read_json::<SensorHealthBaseline>(&resolve(workspace, p)))
        .transpose()?;
    let mut all =
        std::collections::BTreeMap::<String, Vec<(String, Option<f64>, Option<f64>)>>::new();
    let mut provenance = None;
    for r in man.records {
        let a: SensorHealthAssessment = read_json(&resolve(base, &r.assessment))?;
        provenance.get_or_insert(a.provenance.clone());
        for f in a.features {
            all.entry(f.name.clone()).or_default().push((
                r.record_id.clone(),
                f.value,
                r.independent_value,
            ));
        }
    }
    let mut trends = Vec::new();
    for (name, points) in all {
        let b = baseline.as_ref().and_then(|x| {
            x.feature_distributions
                .iter()
                .find(|f| f.feature == name)
                .and_then(|f| f.mean)
        });
        trends.push(health::trend::calculate(&name, points, b));
    }
    let p =
        provenance.ok_or_else(|| RunnerError::Message("health trend manifest is empty".into()))?;
    let report = health::trend::report("health-trend", trends, p);
    let dir = output_dir(workspace, output, "health_trend");
    fs::create_dir_all(&dir)?;
    write_json(&dir.join(&loaded.config.export.trends_filename), &report)?;
    let mut w = csv::Writer::from_path(dir.join("health_trends.csv"))?;
    w.write_record([
        "feature",
        "record_id",
        "independent_value",
        "value",
        "absolute_change",
        "relative_change",
        "log_change",
    ])?;
    for t in &report.trends {
        for p in &t.points {
            w.write_record([
                t.feature.clone(),
                p.record_id.clone(),
                fmt(p.independent_value),
                fmt(p.value),
                fmt(p.absolute_change),
                fmt(p.relative_change),
                fmt(p.log_change),
            ])?;
        }
    }
    println!("Health trend written to {}", dir.display());
    Ok(())
}
pub fn report(workspace: &Path, results: &Path, output: Option<&Path>) -> Result<(), RunnerError> {
    let r: SensorHealthAssessment = read_json(&resolve(workspace, results))?;
    let dest = output_file(workspace, output, "health_report.txt");
    fs::write(&dest, human_report(&r))?;
    println!("Health report written to {}", dest.display());
    Ok(())
}
fn export_assessment(
    workspace: &Path,
    output: Option<&Path>,
    r: &SensorHealthAssessment,
) -> Result<(), RunnerError> {
    let dir = output_dir(workspace, output, "health");
    fs::create_dir_all(&dir)?;
    let c = &r.configuration.export;
    write_json(&dir.join(&c.assessment_filename), r)?;
    let mut f = csv::Writer::from_path(dir.join(&c.features_filename))?;
    f.write_record(["feature", "domain", "value", "unit", "source"])?;
    for x in &r.features {
        f.write_record([
            x.name.clone(),
            format!("{:?}", x.domain),
            fmt(x.value),
            x.unit.clone(),
            x.source.clone(),
        ])?;
    }
    let mut w = csv::Writer::from_path(dir.join(&c.findings_filename))?;
    w.write_record([
        "finding",
        "severity",
        "confidence",
        "triggered_rules",
        "supporting_domains",
        "alternatives",
    ])?;
    for x in &r.findings {
        w.write_record([
            format!("{:?}", x.finding),
            format!("{:?}", x.severity),
            format!("{:?}", x.confidence),
            x.triggered_rules.join(";"),
            x.supporting_evidence
                .iter()
                .map(|e| format!("{:?}", e.domain))
                .collect::<Vec<_>>()
                .join(";"),
            x.alternative_explanations.join(";"),
        ])?;
    }
    fs::write(dir.join(&c.report_filename), human_report(r))?;
    if r.configuration.plotting.enabled {
        crate::plottings::health_plot::plot_health_assessment(r, &dir)?;
    }
    println!("Health assessment written to {}", dir.display());
    Ok(())
}
fn human_report(r: &SensorHealthAssessment) -> String {
    let mut s = format!(
        "Sensor health assessment\n=========================\nStatus: {:?}\nFeatures: {}\nFindings: {}\nMissing domains: {:?}\nWarnings: {:?}\n",
        r.overall_status,
        r.features.len(),
        r.findings.len(),
        r.missing_domains,
        r.warnings
    );
    for f in &r.findings {
        s.push_str(&format!("Finding {:?} severity {:?}, confidence {:?}; supporting domains: {:?}; alternatives: {:?}\n",f.finding,f.severity,f.confidence,f.supporting_evidence.iter().map(|e|e.domain).collect::<Vec<_>>(),f.alternative_explanations));
    }
    s
}
#[derive(Debug, Clone, Deserialize)]
struct HealthTrendManifest {
    pub schema_version: u32,
    pub records: Vec<HealthTrendRecord>,
}
#[derive(Debug, Clone, Deserialize)]
struct HealthTrendRecord {
    pub record_id: String,
    pub assessment: PathBuf,
    #[serde(default)]
    pub independent_value: Option<f64>,
}
fn load_context(p: &Path) -> Result<Context, RunnerError> {
    let d = crate::domain::load_experiment_metadata(p)?;
    let temperature_values_k = d
        .environmental_data
        .iter()
        .filter(|series| series.name.to_ascii_lowercase().contains("temp"))
        .flat_map(|series| {
            let celsius =
                series.unit.eq_ignore_ascii_case("c") || series.unit.eq_ignore_ascii_case("°c");
            series
                .values
                .iter()
                .flatten()
                .copied()
                .filter(|value| value.is_finite())
                .map(move |value| if celsius { value + 273.15 } else { value })
        })
        .collect::<Vec<_>>();
    let temperature_k = if temperature_values_k.is_empty() {
        None
    } else {
        Some(temperature_values_k.iter().sum::<f64>() / temperature_values_k.len() as f64)
    };
    Ok(Context {
        sensor_id: d.sensor.sensor_id,
        sensor_type: d.sensor.sensor_type.clone(),
        sensor_design: d.sensor.model.or(d.sensor.sensor_type),
        analyte: d.sensor.analyte,
        sample_matrix: Some(d.sample_matrix),
        temperature_k,
        temperature_values_k,
        experiment_id: Some(d.experiment_id),
        metadata_source: Some(p.display().to_string()),
    })
}
fn read_json<T: DeserializeOwned>(p: &Path) -> Result<T, RunnerError> {
    Ok(serde_json::from_str(&fs::read_to_string(p)?)?)
}
fn read_toml<T: DeserializeOwned>(p: &Path) -> Result<T, RunnerError> {
    Ok(toml::from_str(&fs::read_to_string(p)?)?)
}
fn write_json<T: serde::Serialize>(p: &Path, v: &T) -> Result<(), RunnerError> {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(p, serde_json::to_string_pretty(v)?)?;
    Ok(())
}
fn resolve(w: &Path, p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        w.join(p)
    }
}
fn output_dir(w: &Path, p: Option<&Path>, d: &str) -> PathBuf {
    p.map(|x| resolve(w, x))
        .unwrap_or_else(|| w.join("output").join(d))
}
fn output_file(w: &Path, p: Option<&Path>, d: &str) -> PathBuf {
    let p = p
        .map(|x| resolve(w, x))
        .unwrap_or_else(|| w.join("output").join("health"));
    if p.extension().is_some() {
        p
    } else {
        p.join(d)
    }
}
fn fmt(v: Option<f64>) -> String {
    v.filter(|x| x.is_finite())
        .map(|x| format!("{x:.12e}"))
        .unwrap_or_default()
}
