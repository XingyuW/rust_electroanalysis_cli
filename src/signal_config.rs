//! Configuration for non-destructive signal-quality analysis.

use crate::domain::ConfigurationError;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SamplingPolicy {
    #[default]
    RequireRegular,
    AllowIrregularTimeDomainOnly,
    ResampleLinear,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SignalWindowSource {
    #[default]
    EntireMeasurement,
    ExplicitInterval,
    EventRelative,
    StableExperimentRegion,
    ResidualArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DetrendKind {
    None,
    Mean,
    #[default]
    Linear,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WindowingConfig {
    pub source: SignalWindowSource,
    pub start_s: Option<f64>,
    pub end_s: Option<f64>,
    pub relative_start_s: Option<f64>,
    pub relative_end_s: Option<f64>,
    pub exclude_before_event_s: f64,
    pub exclude_after_event_s: f64,
    pub eligible_event_kinds: Vec<String>,
}
impl Default for WindowingConfig {
    fn default() -> Self {
        Self {
            source: Default::default(),
            start_s: None,
            end_s: None,
            relative_start_s: None,
            relative_end_s: None,
            exclude_before_event_s: 0.0,
            exclude_after_event_s: 0.0,
            eligible_event_kinds: vec![
                "concentration_step".into(),
                "flow_change".into(),
                "temperature_change".into(),
                "interferent_addition".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SamplingConfig {
    pub policy: SamplingPolicy,
    pub regularity_relative_tolerance: f64,
    pub resample_interval_s: Option<f64>,
    pub maximum_interpolation_gap_s: f64,
}
impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            policy: Default::default(),
            regularity_relative_tolerance: 0.01,
            resample_interval_s: None,
            maximum_interpolation_gap_s: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct StatisticsConfig {
    pub confidence_level: f64,
    pub quantiles: Vec<f64>,
}
impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            quantiles: vec![0.01, 0.05, 0.25, 0.5, 0.75, 0.95, 0.99],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FrequencyBand {
    pub name: String,
    pub minimum_hz: f64,
    pub maximum_hz: f64,
}
impl Default for FrequencyBand {
    fn default() -> Self {
        Self {
            name: "all".into(),
            minimum_hz: 0.0,
            maximum_hz: f64::INFINITY,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PsdConfig {
    pub enabled: bool,
    pub segment_points: usize,
    pub segment_duration_s: Option<f64>,
    pub overlap_fraction: f64,
    pub fft_length: Option<usize>,
    pub window: String,
    pub detrend: DetrendKind,
    pub parseval_tolerance: f64,
    pub minimum_frequency_hz: Option<f64>,
    pub maximum_frequency_hz: Option<f64>,
    pub frequency_bands: Vec<FrequencyBand>,
}
impl Default for PsdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            segment_points: 256,
            segment_duration_s: None,
            overlap_fraction: 0.5,
            fft_length: None,
            window: "hann".into(),
            detrend: DetrendKind::Linear,
            parseval_tolerance: 0.10,
            minimum_frequency_hz: None,
            maximum_frequency_hz: None,
            frequency_bands: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AllanConfig {
    pub enabled: bool,
    pub minimum_clusters: usize,
    pub tau_points: usize,
}
impl Default for AllanConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            minimum_clusters: 8,
            tau_points: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DriftConfig {
    pub models: Vec<String>,
    pub minimum_duration_s: f64,
}
impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            models: vec!["ordinary_linear".into(), "theil_sen".into()],
            minimum_duration_s: 300.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SpikesConfig {
    pub enabled: bool,
    pub method: String,
    pub window_points: usize,
    pub window_duration_s: Option<f64>,
    pub mad_threshold: f64,
    pub minimum_local_observations: usize,
    pub maximum_flagged_fraction: f64,
}
impl Default for SpikesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: "hampel".into(),
            window_points: 11,
            window_duration_s: None,
            mad_threshold: 4.0,
            minimum_local_observations: 5,
            maximum_flagged_fraction: 0.25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct CorrelationConfig {
    pub enabled: bool,
    pub maximum_lag_s: f64,
    pub lag_step_s: Option<f64>,
    pub minimum_observations: usize,
}
impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            maximum_lag_s: 60.0,
            lag_step_s: None,
            minimum_observations: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SignalPlotConfig {
    pub enabled: bool,
}
impl Default for SignalPlotConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct SignalExportConfig {
    pub results_filename: String,
    pub summary_filename: String,
    pub psd_filename: String,
    pub allan_filename: String,
    pub drift_filename: String,
    pub spikes_filename: String,
    pub correlations_filename: String,
    pub report_filename: String,
}
impl Default for SignalExportConfig {
    fn default() -> Self {
        Self {
            results_filename: "signal_results.json".into(),
            summary_filename: "signal_summary.csv".into(),
            psd_filename: "signal_psd.csv".into(),
            allan_filename: "signal_allan.csv".into(),
            drift_filename: "signal_drift.csv".into(),
            spikes_filename: "signal_spikes.csv".into(),
            correlations_filename: "signal_correlations.csv".into(),
            report_filename: "signal_report.txt".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ResolvedSignalConfig {
    pub schema_version: u32,
    pub windowing: WindowingConfig,
    pub sampling: SamplingConfig,
    pub statistics: StatisticsConfig,
    pub psd: PsdConfig,
    pub allan: AllanConfig,
    pub drift: DriftConfig,
    pub spikes: SpikesConfig,
    pub correlation: CorrelationConfig,
    pub plotting: SignalPlotConfig,
    pub export: SignalExportConfig,
}
impl Default for ResolvedSignalConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            windowing: Default::default(),
            sampling: Default::default(),
            statistics: Default::default(),
            psd: Default::default(),
            allan: Default::default(),
            drift: Default::default(),
            spikes: Default::default(),
            correlation: Default::default(),
            plotting: Default::default(),
            export: Default::default(),
        }
    }
}

pub struct LoadedSignalConfig {
    pub config: ResolvedSignalConfig,
    pub source_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}
impl LoadedSignalConfig {
    pub fn load(
        workspace: &Path,
        override_path: Option<&Path>,
    ) -> Result<Self, ConfigurationError> {
        let path = override_path
            .map(|p| {
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    workspace.join(p)
                }
            })
            .or_else(|| {
                let p = workspace.join("config/signal.toml");
                p.exists().then_some(p)
            });
        let Some(path) = path else {
            return Ok(Self {
                config: Default::default(),
                source_path: None,
                warnings: vec!["signal configuration not found; defaults used".into()],
            });
        };
        let text = fs::read_to_string(&path).map_err(|e| ConfigurationError::io(&path, e))?;
        let config: ResolvedSignalConfig =
            toml::from_str(&text).map_err(|e| ConfigurationError::parse(&path, e))?;
        if config.schema_version != 1 {
            return Err(ConfigurationError::invalid(format!(
                "unsupported signal configuration schema {}",
                config.schema_version
            )));
        }
        Ok(Self {
            config,
            source_path: Some(path),
            warnings: Vec::new(),
        })
    }
}
