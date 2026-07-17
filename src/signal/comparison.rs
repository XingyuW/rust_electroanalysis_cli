use crate::{
    data_file::parse_measurement_file, domain::AnalysisProvenance, results::SignalComparisonRecord,
    signal_config::ResolvedSignalConfig,
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Deserialize)]
pub struct SignalComparisonManifest {
    pub schema_version: u32,
    pub records: Vec<SignalComparisonManifestRecord>,
}
#[derive(Debug, Clone, Deserialize)]
pub struct SignalComparisonManifestRecord {
    pub record_id: String,
    pub category: String,
    pub input: PathBuf,
    pub metadata: Option<PathBuf>,
    pub channel: String,
}
pub fn load_manifest(path: &Path) -> Result<SignalComparisonManifest, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    toml::from_str(&text).map_err(|e| e.to_string())
}
pub fn compare(
    base: &Path,
    manifest: &SignalComparisonManifest,
    config: &ResolvedSignalConfig,
) -> Result<(Vec<SignalComparisonRecord>, AnalysisProvenance), String> {
    let mut out = Vec::new();
    let mut provenance = None;
    for r in &manifest.records {
        let input = if r.input.is_absolute() {
            r.input.clone()
        } else {
            base.join(&r.input)
        };
        let parsed = parse_measurement_file(&input).map_err(|e| e.to_string())?;
        let record_provenance =
            AnalysisProvenance::from_paths(&input, None).map_err(|e| e.to_string())?;
        let report = crate::signal::analyze_measurement(
            &parsed.measurement,
            &r.channel,
            None,
            config,
            Some(record_provenance),
        )
        .map_err(|e| e.to_string())?;
        provenance.get_or_insert(report.provenance.clone());
        out.push(SignalComparisonRecord {
            record_id: r.record_id.clone(),
            category: r.category.clone(),
            channel: r.channel.clone(),
            count: report.descriptive.count,
            mean: report.descriptive.mean,
            standard_deviation: report.descriptive.standard_deviation,
            robust_standard_deviation: report.descriptive.robust_standard_deviation,
            drift_slope_v_per_s: report.drift.first().and_then(|d| d.slope_v_per_s),
            spike_fraction: report.spikes.flagged_fraction,
            warnings: report.warnings,
        });
    }
    Ok((
        out,
        provenance.ok_or_else(|| "signal comparison manifest is empty".to_string())?,
    ))
}
