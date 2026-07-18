//! Orchestration and exports for Phase 4 mechanism-evidence workflows.

use crate::mechanism::{
    calculate_trend, compare_timescales, extract_eis_timescales, extract_transient_timescales,
    load_manifest, resolve_path, summarize_record,
};
use crate::mechanism_config::LoadedMechanismConfig;
use crate::results::{
    EisFitArtifact, MechanismAnalysisReport, MechanismRecordInput, MechanismWarning,
};
use crate::runners::RunnerError;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn compare(
    workspace: &Path,
    eis_path: &Path,
    transient_path: &Path,
    calibration_path: Option<&Path>,
    metadata_path: Option<&Path>,
    config_path: Option<&Path>,
    output_path: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedMechanismConfig::load(workspace, config_path)?;
    for warning in &loaded.warnings {
        eprintln!("Warning: {warning}");
    }
    let eis_path = resolve_path(workspace, &eis_path.to_string_lossy());
    let transient_path = resolve_path(workspace, &transient_path.to_string_lossy());
    let eis: EisFitArtifact = read_json(&eis_path)?;
    let transient: crate::results::TransientAnalysisReport = read_json(&transient_path)?;
    let metadata = metadata_path
        .and_then(|path| load_context(&resolve_path(workspace, &path.to_string_lossy())));
    let record = MechanismRecordInput {
        record_id: eis.fit_id.clone(),
        experiment_id: metadata
            .as_ref()
            .and_then(|m| m.get("experiment_id").cloned())
            .or_else(|| eis.experiment_id.clone())
            .or_else(|| Some(transient.experiment_id.clone())),
        sensor_id: metadata
            .as_ref()
            .and_then(|m| m.get("sensor_id").cloned())
            .or_else(|| eis.sensor_id.clone()),
        eis_fit: eis_path.to_string_lossy().to_string(),
        transient_results: transient_path.to_string_lossy().to_string(),
        calibration_results: calibration_path.map(|p| p.to_string_lossy().to_string()),
        metadata: metadata_path.map(|p| p.to_string_lossy().to_string()),
        condition: metadata.as_ref().and_then(|m| m.get("condition").cloned()),
        sensor_age_days: metadata
            .as_ref()
            .and_then(|m| m.get("sensor_age_days").and_then(|v| v.parse().ok())),
    };
    let mut warnings = Vec::new();
    if eis.experiment_id.is_some()
        && eis.experiment_id.as_deref() != Some(transient.experiment_id.as_str())
    {
        warnings.push(MechanismWarning {
            kind: "record_mismatch".to_string(),
            message: format!(
                "EIS experiment ID {:?} differs from transient experiment ID {}",
                eis.experiment_id, transient.experiment_id
            ),
        });
    }
    let eis_timescales = extract_eis_timescales(
        &eis,
        loaded.config.confidence_level,
        loaded.config.frequency_boundary_margin,
    );
    let transient_timescales = extract_transient_timescales(
        &transient,
        loaded.config.allow_warning_fits,
        loaded.config.confidence_level,
    );
    if loaded.config.require_experiment_id && record.experiment_id.is_none() {
        warnings.push(MechanismWarning {
            kind: "missing_experiment_id".to_string(),
            message: "configured matching requires an experiment ID; no comparison was performed"
                .to_string(),
        });
    }
    if calibration_path.is_none() {
        warnings.push(MechanismWarning {
            kind: "calibration_context_unavailable".to_string(),
            message: "no calibration artifact was supplied; calibration context is absent"
                .to_string(),
        });
    }
    let comparisons = if warnings
        .iter()
        .any(|w| w.kind == "record_mismatch" || w.kind == "missing_experiment_id")
    {
        Vec::new()
    } else {
        eis_timescales
            .iter()
            .filter(|t| t.value_s > 0.0)
            .flat_map(|e| {
                transient_timescales
                    .iter()
                    .filter(|t| t.value_s > 0.0)
                    .map(|t| compare_timescales(&record.record_id, e, t, &loaded.config))
            })
            .collect()
    };
    let summary = summarize_record(
        &record,
        record.experiment_id.clone(),
        record.sensor_id.clone(),
        calibration_path.is_some(),
        warnings.clone(),
    );
    let hypotheses = assess_hypotheses(
        &loaded.config.hypotheses,
        &eis_timescales,
        &transient_timescales,
        &comparisons,
    );
    let report = MechanismAnalysisReport {
        schema_version: 1,
        analysis_id: format!("mechanism:{}", record.record_id),
        records: vec![summary],
        eis_timescales,
        transient_timescales,
        comparisons,
        hypotheses,
        trends: Vec::new(),
        configuration: loaded.config,
        provenance: Some(eis.provenance.clone()),
        warnings,
        transient_configuration: Some(transient.configuration.clone()),
    };
    export_report(workspace, output_path, &report)
}

pub fn trend(
    workspace: &Path,
    manifest_path: &Path,
    config_path: Option<&Path>,
    output_path: Option<&Path>,
) -> Result<(), RunnerError> {
    let loaded = LoadedMechanismConfig::load(workspace, config_path)?;
    let manifest_path = resolve_path(workspace, &manifest_path.to_string_lossy());
    let manifest =
        load_manifest(&manifest_path).map_err(|e| RunnerError::Message(e.to_string()))?;
    let base = manifest_path.parent().unwrap_or(workspace);
    let mut report=MechanismAnalysisReport { schema_version:1, analysis_id:"mechanism-trend".to_string(), records:Vec::new(), eis_timescales:Vec::new(), transient_timescales:Vec::new(), comparisons:Vec::new(), hypotheses:manifest.hypotheses.clone().into_iter().map(|h| crate::results::HypothesisAssessment { hypothesis_id:h.hypothesis_id, transient_timescale:h.transient_timescale, eis_role:h.eis_role, description:h.description, assessment:"insufficient evidence".to_string(), supporting_observations:Vec::new(), contradictory_observations:Vec::new(), missing_evidence:vec!["replicate-level hypothesis evaluation is not available from a single manifest record".to_string()], assumptions:Vec::new(), alternative_explanations:Vec::new() }).collect(), trends:Vec::new(), configuration:loaded.config.clone(), provenance:None, warnings:Vec::new(), transient_configuration:None };
    for record in manifest.records {
        let eis_path = resolve_path(base, &record.eis_fit);
        let transient_path = resolve_path(base, &record.transient_results);
        let eis: EisFitArtifact = read_json(&eis_path)?;
        let transient: crate::results::TransientAnalysisReport = read_json(&transient_path)?;
        let et = extract_eis_timescales(
            &eis,
            loaded.config.confidence_level,
            loaded.config.frequency_boundary_margin,
        );
        let tt = extract_transient_timescales(
            &transient,
            loaded.config.allow_warning_fits,
            loaded.config.confidence_level,
        );
        let record_id = record.record_id.clone();
        let summary = summarize_record(
            &record,
            record.experiment_id.clone().or(eis.experiment_id.clone()),
            record.sensor_id.clone().or(eis.sensor_id.clone()),
            record.calibration_results.is_some(),
            Vec::new(),
        );
        report.records.push(summary);
        report.eis_timescales.extend(et.into_iter().map(|mut t| {
            t.timescale_id = format!("{record_id}:{}", t.timescale_id);
            t
        }));
        report
            .transient_timescales
            .extend(tt.into_iter().map(|mut t| {
                t.timescale_id = format!("{record_id}:{}", t.timescale_id);
                t
            }));
    }
    for record in &report.records {
        let eis = report
            .eis_timescales
            .iter()
            .filter(|t| {
                t.timescale_id
                    .starts_with(&format!("{}:", record.record_id))
            })
            .collect::<Vec<_>>();
        let trans = report
            .transient_timescales
            .iter()
            .filter(|t| {
                t.timescale_id
                    .starts_with(&format!("{}:", record.record_id))
            })
            .collect::<Vec<_>>();
        for e in &eis {
            for t in &trans {
                report.comparisons.push(compare_timescales(
                    &record.record_id,
                    e,
                    t,
                    &report.configuration,
                ));
            }
        }
    }
    let values = report
        .records
        .iter()
        .filter_map(|r| {
            let value = report
                .eis_timescales
                .iter()
                .find(|t| t.timescale_id.starts_with(&format!("{}:", r.record_id)))
                .map(|t| t.value_s);
            value.map(|v| (r.record_id.clone(), v))
        })
        .collect::<Vec<_>>();
    report.trends.push(calculate_trend(
        "EIS characteristic timescale",
        &report.records,
        &values,
        &report.configuration.trend_independent_variable,
        report.configuration.trend_minimum_records,
    ));
    export_report(workspace, output_path, &report)
}

pub fn report(
    workspace: &Path,
    results_path: &Path,
    output_path: Option<&Path>,
) -> Result<(), RunnerError> {
    let results_path = resolve_path(workspace, &results_path.to_string_lossy());
    let report: MechanismAnalysisReport = read_json(&results_path)?;
    let destination = output_path
        .map(|p| resolve_path(workspace, &p.to_string_lossy()))
        .unwrap_or_else(|| results_path.with_file_name("mechanism_report.txt"));
    let destination = if destination.extension().is_some() {
        destination
    } else {
        destination.join("mechanism_report.txt")
    };
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&destination, human_report(&report))?;
    println!("Mechanism report written to {}", destination.display());
    Ok(())
}

fn export_report(
    workspace: &Path,
    output_path: Option<&Path>,
    report: &MechanismAnalysisReport,
) -> Result<(), RunnerError> {
    let dir = output_path
        .map(|p| resolve_path(workspace, &p.to_string_lossy()))
        .unwrap_or_else(|| workspace.join("output/mechanism"));
    fs::create_dir_all(&dir)?;
    fs::write(
        dir.join("mechanism_results.json"),
        serde_json::to_string_pretty(report).map_err(|e| RunnerError::Message(e.to_string()))?,
    )?;
    write_timescales(&dir.join("characteristic_timescales.csv"), report)?;
    write_comparisons(&dir.join("timescale_comparisons.csv"), report)?;
    write_trends(&dir.join("mechanism_trends.csv"), report)?;
    fs::write(dir.join("mechanism_report.txt"), human_report(report))?;
    crate::plottings::plot_mechanism_report(report, &dir)?;
    println!("Mechanism outputs written to {}", dir.display());
    Ok(())
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, RunnerError> {
    serde_json::from_str(&fs::read_to_string(path)?)
        .map_err(|e| RunnerError::Message(format!("{}: {e}", path.display())))
}
fn load_context(path: &Path) -> Option<BTreeMap<String, String>> {
    let text = fs::read_to_string(path).ok()?;
    let value: textual_toml::Value = toml::from_str(&text).ok()?;
    Some(value.into_map())
}
mod textual_toml {
    #[derive(Debug)]
    pub enum Value {
        Table(std::collections::BTreeMap<String, Value>),
        String(String),
        Float(f64),
        Integer(i64),
        Bool(bool),
    }
    impl<'de> serde::Deserialize<'de> for Value {
        fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            let v = toml::Value::deserialize(d).map_err(serde::de::Error::custom)?;
            Ok(match v {
                toml::Value::Table(t) => {
                    Value::Table(t.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
                }
                toml::Value::String(s) => Value::String(s),
                toml::Value::Float(f) => Value::Float(f),
                toml::Value::Integer(i) => Value::Integer(i),
                toml::Value::Boolean(b) => Value::Bool(b),
                _ => Value::String(String::new()),
            })
        }
    }
    impl From<toml::Value> for Value {
        fn from(v: toml::Value) -> Self {
            match v {
                toml::Value::String(s) => Value::String(s),
                toml::Value::Float(f) => Value::Float(f),
                toml::Value::Integer(i) => Value::Integer(i),
                toml::Value::Boolean(b) => Value::Bool(b),
                toml::Value::Table(t) => {
                    Value::Table(t.into_iter().map(|(k, v)| (k, Self::from(v))).collect())
                }
                _ => Value::String(String::new()),
            }
        }
    }
    impl Value {
        pub fn into_map(self) -> std::collections::BTreeMap<String, String> {
            let Value::Table(t) = self else {
                return Default::default();
            };
            t.into_iter()
                .filter_map(|(k, v)| match v {
                    Value::String(s) => Some((k, s)),
                    Value::Float(v) => Some((k, v.to_string())),
                    Value::Integer(v) => Some((k, v.to_string())),
                    Value::Bool(v) => Some((k, v.to_string())),
                    Value::Table(_) => None,
                })
                .collect()
        }
    }
}
fn write_timescales(path: &Path, report: &MechanismAnalysisReport) -> Result<(), RunnerError> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "record_id",
        "source",
        "timescale_id",
        "label",
        "value_s",
        "standard_error_s",
        "validity",
        "warnings",
    ])?;
    for t in report
        .eis_timescales
        .iter()
        .chain(&report.transient_timescales)
    {
        let record = t.timescale_id.split(':').next().unwrap_or("");
        w.write_record(vec![
            record.to_string(),
            format!("{:?}", t.source),
            t.timescale_id.clone(),
            t.label.clone(),
            t.value_s.to_string(),
            t.standard_error_s
                .map(|v| v.to_string())
                .unwrap_or_default(),
            format!("{:?}", t.validity),
            t.warnings.len().to_string(),
        ])?;
    }
    w.flush()?;
    Ok(())
}
fn write_comparisons(path: &Path, report: &MechanismAnalysisReport) -> Result<(), RunnerError> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "record_id",
        "eis_timescale_id",
        "transient_timescale_id",
        "ratio",
        "log10_distance",
        "relative_difference",
        "interval_overlap",
        "compatibility_probability",
        "evidence_level",
    ])?;
    for c in &report.comparisons {
        w.write_record(vec![
            c.record_id.clone(),
            c.eis_timescale_id.clone(),
            c.transient_timescale_id.clone(),
            opt(c.ratio),
            opt(c.log10_distance),
            opt(c.symmetric_relative_difference),
            c.confidence_interval_overlap
                .map(|v| v.to_string())
                .unwrap_or_default(),
            opt(c.compatibility_probability),
            format!("{:?}", c.evidence_level),
        ])?;
    }
    w.flush()?;
    Ok(())
}
fn write_trends(path: &Path, report: &MechanismAnalysisReport) -> Result<(), RunnerError> {
    let mut w = csv::Writer::from_path(path)?;
    w.write_record([
        "variable",
        "independent_variable",
        "records",
        "absolute_change",
        "relative_change",
        "log_change",
        "slope",
        "robust_slope",
        "rank_correlation",
        "replicate_variability",
    ])?;
    for t in &report.trends {
        w.write_record(vec![
            t.variable.clone(),
            t.independent_variable.clone(),
            t.records.to_string(),
            opt(t.absolute_change),
            opt(t.relative_change),
            opt(t.log_change),
            opt(t.slope),
            opt(t.robust_slope),
            opt(t.rank_correlation),
            opt(t.replicate_variability),
        ])?;
    }
    w.flush()?;
    Ok(())
}
fn opt(v: Option<f64>) -> String {
    v.filter(|v| v.is_finite())
        .map(|v| v.to_string())
        .unwrap_or_default()
}
fn human_report(report: &MechanismAnalysisReport) -> String {
    let mut s = String::from(
        "Mechanism evidence report\n=========================\n\nThese interpretations are conditional on the selected models, preprocessing choices, parameter identifiability, and data quality. They do not establish a unique physical or chemical mechanism and should not be treated as causal proof.\n\nNumerical agreement is reported as temporal compatibility or supporting evidence only; it does not prove a shared electrochemical mechanism, causal confirmation, or definitive identification.\n\n",
    );
    s.push_str(&format!("Records: {}\nEIS-derived timescales: {}\nTransient-fitted timescales: {}\nComparisons: {}\n\n",report.records.len(),report.eis_timescales.len(),report.transient_timescales.len(),report.comparisons.len()));
    for c in &report.comparisons {
        s.push_str(&format!(
            "{}: {:?}, ratio={}, log10 distance={}, overlap={:?}\n",
            c.comparison_id,
            c.evidence_level,
            opt(c.ratio),
            opt(c.log10_distance),
            c.confidence_interval_overlap
        ));
        for e in &c.supporting_evidence {
            s.push_str(&format!("  supporting: {e}\n"));
        }
        for e in &c.contradictory_evidence {
            s.push_str(&format!("  contradictory: {e}\n"));
        }
        for e in &c.alternative_explanations {
            s.push_str(&format!("  alternative explanation: {e}\n"));
        }
    }
    s.push_str("\nDirectly fitted parameters and derived timescales are distinct quantities. Calibration context, when present, is contextual covariate information rather than mechanism proof.\n");
    s
}

fn assess_hypotheses(
    hypotheses: &[crate::results::MechanismHypothesis],
    eis_timescales: &[crate::results::CharacteristicTimescale],
    transient_timescales: &[crate::results::CharacteristicTimescale],
    comparisons: &[crate::results::TimescaleComparison],
) -> Vec<crate::results::HypothesisAssessment> {
    hypotheses.iter().map(|hypothesis| {
        let transient_matches = transient_timescales.iter().filter(|t| t.label.to_ascii_lowercase().contains(&hypothesis.transient_timescale.to_ascii_lowercase())).collect::<Vec<_>>();
        let eis_matches = eis_timescales.iter().filter(|t| t.semantic_role.as_deref() == Some(hypothesis.eis_role.as_str())).collect::<Vec<_>>();
        let relevant = comparisons.iter().filter(|c| transient_matches.iter().any(|t| t.timescale_id == c.transient_timescale_id) && eis_matches.iter().any(|e| e.timescale_id == c.eis_timescale_id)).collect::<Vec<_>>();
        let assessment = if relevant.is_empty() {
            "not_evaluable"
        } else if relevant.iter().any(|c| {
            matches!(
                c.evidence_level,
                crate::results::EvidenceLevel::Strong | crate::results::EvidenceLevel::Moderate
            )
        }) {
            "supported"
        } else if relevant
            .iter()
            .any(|c| matches!(c.evidence_level, crate::results::EvidenceLevel::Weak))
        {
            "weakly_supported"
        } else {
            "indeterminate"
        };
        crate::results::HypothesisAssessment { hypothesis_id: hypothesis.hypothesis_id.clone(), transient_timescale: hypothesis.transient_timescale.clone(), eis_role: hypothesis.eis_role.clone(), description: hypothesis.description.clone(), assessment: assessment.to_string(), supporting_observations: relevant.iter().flat_map(|c| c.supporting_evidence.clone()).collect(), contradictory_observations: relevant.iter().flat_map(|c| c.contradictory_evidence.clone()).collect(), missing_evidence: if eis_matches.is_empty() { vec!["no explicit EIS semantic-role annotation matched the hypothesis".to_string()] } else { Vec::new() }, assumptions: vec!["user-supplied hypothesis is evaluated against observations; it is not automatically discovered".to_string()], alternative_explanations: vec!["temporal compatibility alone does not identify a mechanism".to_string()] }
    }).collect()
}
