#![allow(clippy::collapsible_if)]

use crate::impedance::{EcmEvolutionConfig, EcmSearchConfig};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_ECM_SEARCH_CONFIG_PATH: &str = "config/analysis.toml";
pub const LEGACY_ECM_SEARCH_CONFIG_PATH: &str = "ecm_search.toml";
pub const ANALYSIS_CONFIG_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct LoadedEcmSearchConfig {
    /// Validated runtime config after defaults and migrations are applied.
    pub config: RuntimeEcmSearchConfig,
    /// Base directory used to resolve relative paths from this config source.
    pub base_dir: PathBuf,
    /// Source file on disk, if one existed and was loaded.
    pub source_path: Option<PathBuf>,
    /// Non-fatal load-time compatibility warnings.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RuntimeEcmSearchConfig {
    #[serde(default)]
    pub schema_version: Option<u32>,
    pub max_ranked_results: Option<usize>,
    #[serde(default)]
    pub evolution: RawEvolutionConfig,
    #[serde(default)]
    pub plotting: RawSearchPlottingConfig,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawEvolutionConfig {
    pub population_size: Option<usize>,
    pub generation_limit: Option<u64>,
    pub num_individuals_per_parents: Option<usize>,
    pub selection_ratio: Option<f64>,
    pub mutation_rate: Option<f64>,
    pub reinsertion_ratio: Option<f64>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RawSearchPlottingConfig {
    pub top_n: Option<usize>,
    pub output_dir: Option<String>,
}

impl RuntimeEcmSearchConfig {
    /// Load runtime search config from either explicit CLI override or default
    /// workspace location, returning defaults when file is absent/empty.
    pub fn load(
        workspace_dir: &Path,
        override_path: Option<&Path>,
    ) -> Result<LoadedEcmSearchConfig, String> {
        let config_path = override_path.map(|path| resolve_cli_config_path(path, workspace_dir));
        let default_path = workspace_dir.join(DEFAULT_ECM_SEARCH_CONFIG_PATH);
        let legacy_default_path = workspace_dir.join(LEGACY_ECM_SEARCH_CONFIG_PATH);
        let resolved_default_path = if default_path.exists() {
            default_path
        } else if legacy_default_path.exists() {
            legacy_default_path
        } else {
            workspace_dir.join(DEFAULT_ECM_SEARCH_CONFIG_PATH)
        };
        let resolved_path = config_path.unwrap_or(resolved_default_path);
        let source_path = if resolved_path.exists() {
            Some(resolved_path.clone())
        } else {
            None
        };

        if override_path.is_some() && !resolved_path.exists() {
            return Err(format!(
                "search config override does not exist: {}",
                resolved_path.display()
            ));
        }

        if !resolved_path.exists() {
            return Ok(LoadedEcmSearchConfig {
                config: Self::default(),
                base_dir: workspace_dir.to_path_buf(),
                source_path: None,
                warnings: Vec::new(),
            });
        }

        let config_text = fs::read_to_string(&resolved_path)
            .map_err(|error| format!("failed to read {}: {error}", resolved_path.display()))?;

        if config_text.trim().is_empty() {
            return Ok(LoadedEcmSearchConfig {
                config: Self::default(),
                base_dir: workspace_dir.to_path_buf(),
                source_path,
                warnings: Vec::new(),
            });
        }

        let config: Self = match toml::from_str(&config_text) {
            Ok(config) => config,
            Err(error) if override_path.is_none() => {
                return Ok(LoadedEcmSearchConfig {
                    config: Self::default(),
                    base_dir: workspace_dir.to_path_buf(),
                    source_path,
                    warnings: vec![format!(
                        "failed to parse {}: {error}; defaults were used",
                        resolved_path.display()
                    )],
                });
            }
            Err(error) => {
                return Err(format!(
                    "failed to parse {}: {error}",
                    resolved_path.display()
                ));
            }
        };
        config.validate()?;

        let mut warnings = Vec::new();
        if let Some(schema_version) = config.schema_version {
            if schema_version != ANALYSIS_CONFIG_SCHEMA_VERSION {
                warnings.push(format!(
                    "analysis config schema_version {} does not match supported version {}",
                    schema_version, ANALYSIS_CONFIG_SCHEMA_VERSION
                ));
            }
        }

        Ok(LoadedEcmSearchConfig {
            config,
            base_dir: resolved_path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| workspace_dir.to_path_buf()),
            source_path,
            warnings,
        })
    }

    /// Validate user-supplied numeric ranges before runtime resolution.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(value) = self.max_ranked_results {
            if value == 0 {
                return Err("max_ranked_results must be greater than zero".to_string());
            }
        }

        if let Some(value) = self.evolution.population_size {
            if value == 0 {
                return Err("evolution.population_size must be greater than zero".to_string());
            }
        }

        if let Some(value) = self.evolution.generation_limit {
            if value == 0 {
                return Err("evolution.generation_limit must be greater than zero".to_string());
            }
        }

        if let Some(value) = self.evolution.num_individuals_per_parents {
            if value == 0 {
                return Err(
                    "evolution.num_individuals_per_parents must be greater than zero".to_string(),
                );
            }
        }

        if let Some(value) = self.evolution.selection_ratio {
            if !(0.0..=1.0).contains(&value) || value == 0.0 {
                return Err("evolution.selection_ratio must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(value) = self.evolution.mutation_rate {
            if !(0.0..=1.0).contains(&value) {
                return Err("evolution.mutation_rate must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(value) = self.evolution.reinsertion_ratio {
            if !(0.0..=1.0).contains(&value) || value == 0.0 {
                return Err("evolution.reinsertion_ratio must be between 0.0 and 1.0".to_string());
            }
        }

        if let Some(value) = self.plotting.top_n {
            if value == 0 {
                return Err("plotting.top_n must be greater than zero".to_string());
            }
        }

        Ok(())
    }

    /// Resolve effective search settings with CLI top-N precedence over file
    /// values, then file values over library defaults.
    pub fn resolve_search_config(&self, cli_top: Option<usize>) -> EcmSearchConfig {
        let defaults = EcmSearchConfig::default();
        let evolution_defaults = EcmEvolutionConfig::default();

        EcmSearchConfig {
            evolution: EcmEvolutionConfig {
                population_size: self
                    .evolution
                    .population_size
                    .unwrap_or(evolution_defaults.population_size),
                generation_limit: self
                    .evolution
                    .generation_limit
                    .unwrap_or(evolution_defaults.generation_limit),
                num_individuals_per_parents: self
                    .evolution
                    .num_individuals_per_parents
                    .unwrap_or(evolution_defaults.num_individuals_per_parents),
                selection_ratio: self
                    .evolution
                    .selection_ratio
                    .unwrap_or(evolution_defaults.selection_ratio),
                mutation_rate: self
                    .evolution
                    .mutation_rate
                    .unwrap_or(evolution_defaults.mutation_rate),
                reinsertion_ratio: self
                    .evolution
                    .reinsertion_ratio
                    .unwrap_or(evolution_defaults.reinsertion_ratio),
            },
            max_ranked_results: cli_top
                .or(self.max_ranked_results)
                .unwrap_or(defaults.max_ranked_results),
        }
    }

    /// Resolve top-N plotting count; `0` means plotting disabled.
    pub fn resolved_plot_top_n(&self) -> usize {
        self.plotting.top_n.unwrap_or(0)
    }

    /// Resolve optional plotting output directory relative to `base_dir` when
    /// the configured path is not absolute.
    pub fn resolve_plot_output_dir(&self, base_dir: &Path) -> Option<PathBuf> {
        self.plotting.output_dir.as_ref().map(|path| {
            let candidate = Path::new(path);
            if candidate.is_absolute() {
                candidate.to_path_buf()
            } else {
                base_dir.join(candidate)
            }
        })
    }
}

fn resolve_cli_config_path(path: &Path, workspace_dir: &Path) -> PathBuf {
    // Relative CLI paths are interpreted from the current workspace root.
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeEcmSearchConfig;

    #[test]
    fn runtime_search_config_validates_ranges() {
        let config: RuntimeEcmSearchConfig = toml::from_str(
            r#"
                max_ranked_results = 8

                [evolution]
                population_size = 24
                generation_limit = 15
                num_individuals_per_parents = 2
                selection_ratio = 0.7
                mutation_rate = 0.4
                reinsertion_ratio = 0.75

                [plotting]
                top_n = 3
            "#,
        )
        .expect("parse search config");

        config.validate().expect("valid search config");
        let resolved = config.resolve_search_config(None);
        assert_eq!(resolved.evolution.population_size, 24);
        assert_eq!(resolved.evolution.generation_limit, 15);
        assert_eq!(resolved.evolution.num_individuals_per_parents, 2);
        assert!((resolved.evolution.selection_ratio - 0.7).abs() < f64::EPSILON);
        assert!((resolved.evolution.mutation_rate - 0.4).abs() < f64::EPSILON);
        assert!((resolved.evolution.reinsertion_ratio - 0.75).abs() < f64::EPSILON);
        assert_eq!(resolved.max_ranked_results, 8);
        assert_eq!(config.resolved_plot_top_n(), 3);
    }
}
