use crate::domain::WorkspaceError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_CONFIG_SCHEMA_VERSION: u32 = 1;

pub const CONFIG_DIR_NAME: &str = "config";
pub const DATA_DIR_NAME: &str = "data";
pub const OUTPUT_DIR_NAME: &str = "output";
pub const LOGS_DIR_NAME: &str = "logs";

pub const APP_CONFIG_PATH: &str = "config/app.toml";
pub const PLOTTING_CONFIG_PATH: &str = "config/plotting.toml";
pub const ANALYSIS_CONFIG_PATH: &str = "config/analysis.toml";
pub const PARSING_CONFIG_PATH: &str = "config/parsing.toml";
pub const TRANSIENT_CONFIG_PATH: &str = "config/transient.toml";
pub const CALIBRATION_CONFIG_PATH: &str = "config/calibration.toml";
pub const MECHANISM_CONFIG_PATH: &str = "config/mechanism.toml";
pub const SIGNAL_CONFIG_PATH: &str = "config/signal.toml";
pub const HEALTH_CONFIG_PATH: &str = "config/health.toml";

const LEGACY_PLOTTING_CONFIG_PATH: &str = "plot_config.toml";
const LEGACY_ANALYSIS_CONFIG_PATH: &str = "ecm_search.toml";
const LEGACY_PARSING_CONFIG_PATH: &str = "circuit_models.toml";

#[derive(Debug, Clone)]
pub struct WorkspacePaths {
    pub root: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub output_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub app_config_path: PathBuf,
    pub plotting_config_path: PathBuf,
    pub analysis_config_path: PathBuf,
    pub parsing_config_path: PathBuf,
    pub transient_config_path: PathBuf,
    pub calibration_config_path: PathBuf,
    pub mechanism_config_path: PathBuf,
    pub signal_config_path: PathBuf,
    pub health_config_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct WorkspaceSetup {
    pub paths: WorkspacePaths,
    pub warnings: Vec<String>,
    app_config: AppConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastRunMode {
    PlotAll,
    PlotEis,
    PlotRegular,
    PlotGeneric,
    Search,
    EisFit,
    TransientFit,
    CalibrationExtract,
    CalibrationFit,
    CalibrationValidate,
    CalibrationPredict,
    MechanismCompare,
    MechanismTrend,
    MechanismReport,
}

impl LastRunMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::PlotAll => "plot-all",
            Self::PlotEis => "plot-eis",
            Self::PlotRegular => "plot-regular",
            Self::PlotGeneric => "plot-generic",
            Self::Search => "search",
            Self::EisFit => "eis-fit",
            Self::TransientFit => "transient-fit",
            Self::CalibrationExtract => "calibration-extract",
            Self::CalibrationFit => "calibration-fit",
            Self::CalibrationValidate => "calibration-validate",
            Self::CalibrationPredict => "calibration-predict",
            Self::MechanismCompare => "mechanism-compare",
            Self::MechanismTrend => "mechanism-trend",
            Self::MechanismReport => "mechanism-report",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
struct AppConfig {
    schema_version: u32,
    logging: LoggingConfig,
    last_run: LastRunConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: APP_CONFIG_SCHEMA_VERSION,
            logging: LoggingConfig::default(),
            last_run: LastRunConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
struct LoggingConfig {
    level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
struct LastRunConfig {
    mode: String,
    plot_config_override: Option<String>,
    analysis_config_override: Option<String>,
    search_output_override: Option<String>,
    search_top_override: Option<usize>,
    calibration_config_override: Option<String>,
    calibration_output_override: Option<String>,
}

impl WorkspaceSetup {
    pub fn record_last_run(
        &mut self,
        mode: LastRunMode,
        plot_config_override: Option<&Path>,
        analysis_config_override: Option<&Path>,
        search_output_override: Option<&Path>,
        search_top_override: Option<usize>,
    ) -> Result<(), WorkspaceError> {
        self.app_config.last_run = LastRunConfig {
            mode: mode.as_str().to_string(),
            plot_config_override: plot_config_override.map(path_to_string),
            analysis_config_override: analysis_config_override.map(path_to_string),
            search_output_override: search_output_override.map(path_to_string),
            search_top_override,
            calibration_config_override: self
                .app_config
                .last_run
                .calibration_config_override
                .clone(),
            calibration_output_override: self
                .app_config
                .last_run
                .calibration_output_override
                .clone(),
        };
        save_app_config(&self.paths.app_config_path, &self.app_config)
    }

    pub fn record_calibration_run(
        &mut self,
        mode: LastRunMode,
        config_override: Option<&Path>,
        output_override: Option<&Path>,
    ) -> Result<(), WorkspaceError> {
        self.app_config.last_run.mode = mode.as_str().to_string();
        self.app_config.last_run.calibration_config_override = config_override.map(path_to_string);
        self.app_config.last_run.calibration_output_override = output_override.map(path_to_string);
        save_app_config(&self.paths.app_config_path, &self.app_config)
    }
}

pub fn prepare_workspace(root: &Path) -> Result<WorkspaceSetup, WorkspaceError> {
    let paths = WorkspacePaths {
        root: root.to_path_buf(),
        config_dir: root.join(CONFIG_DIR_NAME),
        data_dir: root.join(DATA_DIR_NAME),
        output_dir: root.join(OUTPUT_DIR_NAME),
        logs_dir: root.join(LOGS_DIR_NAME),
        app_config_path: root.join(APP_CONFIG_PATH),
        plotting_config_path: root.join(PLOTTING_CONFIG_PATH),
        analysis_config_path: root.join(ANALYSIS_CONFIG_PATH),
        parsing_config_path: root.join(PARSING_CONFIG_PATH),
        transient_config_path: root.join(TRANSIENT_CONFIG_PATH),
        calibration_config_path: root.join(CALIBRATION_CONFIG_PATH),
        mechanism_config_path: root.join(MECHANISM_CONFIG_PATH),
        signal_config_path: root.join(SIGNAL_CONFIG_PATH),
        health_config_path: root.join(HEALTH_CONFIG_PATH),
    };
    let mut warnings = Vec::new();

    fs::create_dir_all(&paths.config_dir)
        .map_err(|error| WorkspaceError::io(&paths.config_dir, error))?;
    fs::create_dir_all(&paths.data_dir)
        .map_err(|error| WorkspaceError::io(&paths.data_dir, error))?;
    fs::create_dir_all(&paths.output_dir)
        .map_err(|error| WorkspaceError::io(&paths.output_dir, error))?;
    fs::create_dir_all(&paths.logs_dir)
        .map_err(|error| WorkspaceError::io(&paths.logs_dir, error))?;

    ensure_runtime_config_file(
        root,
        &paths.plotting_config_path,
        LEGACY_PLOTTING_CONFIG_PATH,
        DEFAULT_PLOTTING_CONFIG,
        "plotting config",
        &mut warnings,
    )?;
    ensure_runtime_config_file(
        root,
        &paths.signal_config_path,
        "signal.toml",
        DEFAULT_SIGNAL_CONFIG,
        "signal config",
        &mut warnings,
    )?;
    ensure_runtime_config_file(
        root,
        &paths.health_config_path,
        "health.toml",
        DEFAULT_HEALTH_CONFIG,
        "health config",
        &mut warnings,
    )?;
    ensure_runtime_config_file(
        root,
        &paths.calibration_config_path,
        "calibration.toml",
        DEFAULT_CALIBRATION_CONFIG,
        "calibration config",
        &mut warnings,
    )?;
    ensure_runtime_config_file(
        root,
        &paths.analysis_config_path,
        LEGACY_ANALYSIS_CONFIG_PATH,
        DEFAULT_ANALYSIS_CONFIG,
        "analysis config",
        &mut warnings,
    )?;
    ensure_runtime_config_file(
        root,
        &paths.parsing_config_path,
        LEGACY_PARSING_CONFIG_PATH,
        DEFAULT_PARSING_CONFIG,
        "parsing config",
        &mut warnings,
    )?;
    ensure_runtime_config_file(
        root,
        &paths.transient_config_path,
        "transient.toml",
        DEFAULT_TRANSIENT_CONFIG,
        "transient config",
        &mut warnings,
    )?;
    ensure_runtime_config_file(
        root,
        &paths.mechanism_config_path,
        "mechanism.toml",
        DEFAULT_MECHANISM_CONFIG,
        "mechanism config",
        &mut warnings,
    )?;

    let app_config = load_app_config(&paths.app_config_path, &mut warnings)?;
    save_app_config(&paths.app_config_path, &app_config)?;

    Ok(WorkspaceSetup {
        paths,
        warnings,
        app_config,
    })
}

fn load_app_config(path: &Path, warnings: &mut Vec<String>) -> Result<AppConfig, WorkspaceError> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let text = fs::read_to_string(path).map_err(|error| WorkspaceError::io(path, error))?;
    if text.trim().is_empty() {
        warnings.push(format!(
            "app config {} was empty; defaults were restored",
            path.display()
        ));
        return Ok(AppConfig::default());
    }

    match toml::from_str::<AppConfig>(&text) {
        Ok(mut config) => {
            if config.schema_version != APP_CONFIG_SCHEMA_VERSION {
                warnings.push(format!(
                    "app config schema version {} is not supported; migrated to {}",
                    config.schema_version, APP_CONFIG_SCHEMA_VERSION
                ));
                config.schema_version = APP_CONFIG_SCHEMA_VERSION;
            }
            Ok(config)
        }
        Err(error) => {
            let backup = corrupt_backup_path(path);
            fs::rename(path, &backup).map_err(|rename_error| {
                WorkspaceError::invalid(format!(
                    "failed to parse {}: {error}; additionally failed to move corrupt file: {rename_error}",
                    path.display()
                ))
            })?;
            warnings.push(format!(
                "app config {} is corrupted and was moved to {}; defaults were restored",
                path.display(),
                backup.display()
            ));
            Ok(AppConfig::default())
        }
    }
}

fn save_app_config(path: &Path, config: &AppConfig) -> Result<(), WorkspaceError> {
    let text = toml::to_string_pretty(config).map_err(crate::domain::ConfigurationError::from)?;
    atomic_write(path, &text)
}

fn ensure_runtime_config_file(
    root: &Path,
    target_path: &Path,
    legacy_relative_path: &str,
    default_content: &str,
    label: &str,
    warnings: &mut Vec<String>,
) -> Result<(), WorkspaceError> {
    if target_path.exists() {
        return Ok(());
    }

    let legacy_path = root.join(legacy_relative_path);
    if legacy_path.exists() {
        let content = fs::read_to_string(&legacy_path).map_err(|error| {
            WorkspaceError::invalid(format!(
                "failed to read legacy {} {}: {error}",
                label,
                legacy_path.display()
            ))
        })?;
        atomic_write(target_path, &content)?;
        warnings.push(format!(
            "migrated legacy {} from {} to {}",
            label,
            legacy_path.display(),
            target_path.display()
        ));
        return Ok(());
    }

    atomic_write(target_path, default_content)?;
    Ok(())
}

fn corrupt_backup_path(path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("app.toml");
    path.with_file_name(format!("{file_name}.corrupt-{timestamp}.toml"))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

fn atomic_write(path: &Path, text: &str) -> Result<(), WorkspaceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| WorkspaceError::io(parent, error))?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, text).map_err(|error| WorkspaceError::io(&tmp_path, error))?;
    fs::rename(&tmp_path, path).map_err(|error| WorkspaceError::io(path, error))?;
    Ok(())
}

const DEFAULT_PLOTTING_CONFIG: &str = r#"schema_version = 1

[shared]
input_is_directory = true
output_path = "../output"
output_prefix = ""
"#;

const DEFAULT_SIGNAL_CONFIG: &str = r#"schema_version = 1

[windowing]
source = "stable_experiment_region"
exclude_before_event_s = 10.0
exclude_after_event_s = 300.0

[sampling]
policy = "require_regular"
regularity_relative_tolerance = 0.01
maximum_interpolation_gap_s = 5.0

[psd]
enabled = true
segment_points = 256
overlap_fraction = 0.5
window = "hann"
detrend = "linear"
parseval_tolerance = 0.10

[allan]
enabled = true
minimum_clusters = 8
tau_points = 30

[drift]
models = ["ordinary_linear", "theil_sen"]
minimum_duration_s = 300.0

[spikes]
enabled = true
method = "hampel"
window_points = 11
mad_threshold = 4.0

[correlation]
enabled = true
maximum_lag_s = 60.0

[plotting]
enabled = true
"#;

const DEFAULT_HEALTH_CONFIG: &str = r#"schema_version = 1

[baseline]
minimum_records = 3
robust_statistics = true

[comparability]
require_same_analyte = true
require_same_sample_matrix = true
maximum_temperature_difference_k = 2.0
require_same_sensor_design = true

[normalization]
use_relative_difference = true
use_robust_z_score = true
minimum_baseline_records_for_z_score = 5

[assessment]
minimum_domains_for_assessment = 2
minimum_domains_for_mechanistic_finding = 2
allow_warning_artifacts = true

[[rules]]
rule_id = "elevated-noise"
finding = "elevated_noise"
severity = "moderate"
minimum_evidence_domains = 1

[[rules.all_of]]
feature = "signal.robust_noise_standard_deviation"
operator = "robust_z_greater_than"
value = 3.0

[[rules]]
rule_id = "probable-fouling"
finding = "probable_fouling"
severity = "major"
minimum_evidence_domains = 2
alternative_explanations = ["environmental mismatch", "incomplete baseline context"]

[[rules.all_of]]
feature = "transient.tau_slow"
operator = "relative_increase_greater_than"
value = 1.0

[[rules.any_of]]
feature = "calibration.slope_efficiency"
operator = "relative_decrease_greater_than"
value = 0.20

[[rules.any_of]]
feature = "eis.role.transport.relaxation_timescale"
operator = "relative_increase_greater_than"
value = 1.0

[plotting]
enabled = true
"#;

const DEFAULT_ANALYSIS_CONFIG: &str = r#"schema_version = 1
max_ranked_results = 10

[evolution]
population_size = 24
generation_limit = 12
num_individuals_per_parents = 2
selection_ratio = 0.7
mutation_rate = 0.35
reinsertion_ratio = 0.75

[plotting]
output_dir = "../output"
"#;

const DEFAULT_PARSING_CONFIG: &str = r#"schema_version = 1
fallback_model = "R0-p(CPE1,R1)"

[model_selection]
ranking_metric = "aic"
warburg_aic_threshold = 4.0
"#;

const DEFAULT_TRANSIENT_CONFIG: &str = r#"schema_version = 1

[segmentation]
pre_event_s = 30.0
post_event_s = 300.0
baseline_window_s = 20.0
minimum_points = 20
minimum_duration_s = 10.0
maximum_missing_fraction = 0.20
duplicate_timestamp_policy = "error"
non_monotonic_policy = "sort"
irregular_sampling_policy = "allow"

[baseline]
method = "median"
response_mode = "baseline_relative"

[models]
enabled = ["single", "double", "double_drift", "stretched"]
beta_min = 0.05
beta_max = 1.0

[optimizer]
maximum_iterations = 400
ftol = 1e-10
xtol = 1e-10
gtol = 1e-10
patience = 400
step_bound = 50.0
multiple_starts = 8

[selection]
criterion = "aic"

[validation]
minimum_tau_ratio = 3.0
maximum_tau_to_window_ratio = 1.0
negligible_amplitude_fraction = 0.05
high_autocorrelation_threshold = 0.8
bound_proximity_fraction = 0.01

[uncertainty]
bootstrap_iterations = 500
confidence_level = 0.95
seed = 42
minimum_success_fraction = 0.80

[plotting]
enabled = true
include_components = true
include_residuals = true
include_model_comparison = true

[export]
json_filename = "transient_results.json"
features_filename = "transient_features.csv"
model_comparison_filename = "transient_model_comparison.csv"
report_filename = "transient_report.txt"
"#;

const DEFAULT_CALIBRATION_CONFIG: &str = r#"schema_version = 1

[observation_extraction]
preferred_source = "transient_equilibrium"
allow_warning_fits = true
fallback_source = "steady_state_median"
steady_state_start_s = 180.0
steady_state_end_s = 300.0
minimum_points = 20
maximum_missing_fraction = 0.20
maximum_absolute_slope_v_per_s = 0.00001

[analyte]
name = "auto"
charge = 1

[temperature]
mode = "observation_specific"
default_celsius = 25.0
reference_celsius = 25.0
environmental_series = "temperature"
alignment = "linear_interpolation"
maximum_gap_s = 30.0

[activity]
model = "ideal"

[nernst]
slope_mode = "free"
response_sign = "auto"

[hysteresis]
analyze = true
log_activity_matching_tolerance = 0.05
warning_threshold_v = 0.010

[weighting]
mode = "potential_standard_error"
minimum_standard_error_v = 0.000001

[selection]
criterion = "aicc"
branch = "mixed"

[validation]
mode = "leave_one_concentration_level_out"
folds = 5
seed = 42
prediction_interval_confidence = 0.95

[uncertainty]
confidence_level = 0.95
bootstrap_iterations = 1000
seed = 42
minimum_success_fraction = 0.80

[plotting]
enabled = true
include_residuals = true
include_hysteresis = true
include_validation = true
include_confidence_band = true

[export]
observations_filename = "calibration_observations.json"
model_filename = "calibration_model.json"
results_filename = "calibration_results.json"
features_filename = "calibration_summary.csv"
residuals_filename = "calibration_residuals.csv"
validation_filename = "calibration_validation.csv"
report_filename = "calibration_report.txt"
"#;

const DEFAULT_MECHANISM_CONFIG: &str = r#"schema_version = 1

[eis]
allow_warning_fits = true
confidence_level = 0.95
seed = 42

[transient]
allow_warning_fits = true
selected_model_only = true

[timescale]
monte_carlo_samples = 10000
seed = 42
frequency_boundary_margin = 0.1

[matching]
require_experiment_id = true
require_sensor_id = false

[comparison]
ratio_weak = 10.0
ratio_moderate = 3.0
ratio_strong = 1.5
log_distance_weak = 1.0
log_distance_moderate = 0.5
log_distance_strong = 0.1761
compatibility_ratio_lower = 0.5
compatibility_ratio_upper = 2.0

[evidence]
minimum_replicates_for_strong = 3

[trend]
enabled = true
minimum_records = 3
independent_variable = "sensor_age_days"
"#;
