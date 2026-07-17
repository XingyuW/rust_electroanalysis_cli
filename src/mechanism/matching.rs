//! Explicit record matching; filenames are never used as an implicit key.

use crate::mechanism::error::MechanismError;
use crate::results::{
    MechanismManifest, MechanismRecordInput, MechanismRecordSummary, MechanismWarning,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub fn load_manifest(path: &Path) -> Result<MechanismManifest, MechanismError> {
    let text = fs::read_to_string(path).map_err(|source| MechanismError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let manifest: MechanismManifest = toml::from_str(&text)?;
    if manifest.schema_version != 1 {
        return Err(MechanismError::invalid(format!(
            "unsupported mechanism manifest schema version {}",
            manifest.schema_version
        )));
    }
    let mut ids = BTreeSet::new();
    for record in &manifest.records {
        if record.record_id.trim().is_empty() {
            return Err(MechanismError::invalid("record_id cannot be empty"));
        }
        if !ids.insert(record.record_id.clone()) {
            return Err(MechanismError::invalid(format!(
                "duplicate record_id '{}'",
                record.record_id
            )));
        }
    }
    Ok(manifest)
}

pub fn resolve_path(workspace: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    }
}

pub fn summarize_record(
    record: &MechanismRecordInput,
    experiment_id: Option<String>,
    sensor_id: Option<String>,
    calibration_available: bool,
    mut warnings: Vec<MechanismWarning>,
) -> MechanismRecordSummary {
    if experiment_id.is_none() {
        warnings.push(MechanismWarning {
            kind: "missing_experiment_id".to_string(),
            message: "record has no explicit experiment identifier".to_string(),
        });
    }
    if sensor_id.is_none() {
        warnings.push(MechanismWarning {
            kind: "missing_sensor_id".to_string(),
            message: "record has no explicit sensor identifier".to_string(),
        });
    }
    MechanismRecordSummary {
        record_id: record.record_id.clone(),
        experiment_id,
        sensor_id,
        condition: record.condition.clone(),
        sensor_age_days: record.sensor_age_days,
        metadata: Default::default(),
        calibration_context_available: calibration_available,
        warnings,
    }
}
