use crate::{
    data_file::parse_measurement_file,
    domain::{AnalysisProvenance, DataParsingError, ProvenanceError},
    results::SignalComparisonRecord,
    signal::SignalError,
    signal_config::ResolvedSignalConfig,
};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignalComparisonError {
    #[error("signal comparison manifest I/O error for {path}: {source}")]
    ManifestIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("signal comparison manifest parse error for {path}: {source}")]
    ManifestParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("canonical input failure for comparison record {record_id} ({path}): {source}")]
    Input {
        record_id: String,
        path: PathBuf,
        #[source]
        source: Box<DataParsingError>,
    },
    #[error("provenance failure for comparison record {record_id} ({path}): {source}")]
    Provenance {
        record_id: String,
        path: PathBuf,
        #[source]
        source: Box<ProvenanceError>,
    },
    #[error("signal analysis failure for comparison record {record_id} ({path}): {source}")]
    Analysis {
        record_id: String,
        path: PathBuf,
        #[source]
        source: Box<SignalError>,
    },
    #[error("signal comparison manifest is empty")]
    Empty,
}
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
pub fn load_manifest(path: &Path) -> Result<SignalComparisonManifest, SignalComparisonError> {
    let text = fs::read_to_string(path).map_err(|source| SignalComparisonError::ManifestIo {
        path: path.into(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| SignalComparisonError::ManifestParse {
        path: path.into(),
        source,
    })
}
pub fn compare(
    base: &Path,
    manifest: &SignalComparisonManifest,
    config: &ResolvedSignalConfig,
) -> Result<(Vec<SignalComparisonRecord>, AnalysisProvenance), SignalComparisonError> {
    let mut out = Vec::new();
    let mut provenance = None;
    for r in &manifest.records {
        let input = if r.input.is_absolute() {
            r.input.clone()
        } else {
            base.join(&r.input)
        };
        let mut parsed =
            parse_measurement_file(&input).map_err(|source| SignalComparisonError::Input {
                record_id: r.record_id.clone(),
                path: input.clone(),
                source: Box::new(source),
            })?;
        parsed.measurement = parsed
            .measurement
            .normalized_to_seconds()
            .map_err(|source| SignalComparisonError::Input {
                record_id: r.record_id.clone(),
                path: input.clone(),
                source: Box::new(source),
            })?;
        let record_provenance = AnalysisProvenance::from_paths(&input, None).map_err(|source| {
            SignalComparisonError::Provenance {
                record_id: r.record_id.clone(),
                path: input.clone(),
                source: Box::new(source),
            }
        })?;
        let report = crate::signal::analyze_measurement(
            &parsed.measurement,
            &r.channel,
            None,
            config,
            Some(record_provenance),
        )
        .map_err(|source| SignalComparisonError::Analysis {
            record_id: r.record_id.clone(),
            path: input.clone(),
            source: Box::new(source),
        })?;
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
    Ok((out, provenance.ok_or(SignalComparisonError::Empty)?))
}
