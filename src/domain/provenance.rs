//! Reproducibility metadata for parsed scientific experiments.

use super::errors::ProvenanceError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Input/configuration identity recorded alongside a scientific experiment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisProvenance {
    pub software_version: String,
    pub input_path: PathBuf,
    pub input_sha256: String,
    #[serde(default)]
    pub configuration_path: Option<PathBuf>,
    #[serde(default)]
    pub configuration_sha256: Option<String>,
    pub generation_timestamp: u64,
    #[serde(default)]
    pub git_commit: Option<String>,
}

impl AnalysisProvenance {
    pub fn from_paths(
        input_path: impl AsRef<Path>,
        configuration_path: Option<&Path>,
    ) -> Result<Self, ProvenanceError> {
        let input_path = input_path.as_ref().to_path_buf();
        let configuration_path = configuration_path.map(Path::to_path_buf);
        let input_sha256 = sha256_file(&input_path)?;
        let configuration_sha256 = configuration_path.as_deref().map(sha256_file).transpose()?;
        let generation_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ProvenanceError::Timestamp)?
            .as_secs();

        Ok(Self {
            software_version: env!("CARGO_PKG_VERSION").to_string(),
            input_path,
            input_sha256,
            configuration_path,
            configuration_sha256,
            generation_timestamp,
            git_commit: option_env!("GIT_COMMIT").map(str::to_string),
        })
    }

    pub fn with_git_commit(mut self, git_commit: impl Into<String>) -> Self {
        self.git_commit = Some(git_commit.into());
        self
    }
}

fn sha256_file(path: &Path) -> Result<String, ProvenanceError> {
    let bytes = fs::read(path).map_err(|error| ProvenanceError::io(path, error))?;
    let digest = Sha256::digest(bytes);
    Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::AnalysisProvenance;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn hashes_input_and_configuration_files() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let input = std::env::temp_dir().join(format!("phase1_input_{suffix}.csv"));
        let config = std::env::temp_dir().join(format!("phase1_config_{suffix}.toml"));
        fs::write(&input, "time/sec,value/V\n0,1\n").expect("input");
        fs::write(&config, "experiment_id = 'test'\n").expect("config");

        let provenance =
            AnalysisProvenance::from_paths(&input, Some(config.as_path())).expect("provenance");
        assert_eq!(provenance.software_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(provenance.input_path, PathBuf::from(&input));
        assert_eq!(provenance.input_sha256.len(), 64);
        assert_eq!(
            provenance.configuration_sha256.as_deref().map(str::len),
            Some(64)
        );
        assert!(provenance.generation_timestamp > 0);

        fs::remove_file(input).ok();
        fs::remove_file(config).ok();
    }
}
