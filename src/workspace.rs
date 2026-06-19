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
}

impl LastRunMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::PlotAll => "plot-all",
            Self::PlotEis => "plot-eis",
            Self::PlotRegular => "plot-regular",
            Self::PlotGeneric => "plot-generic",
            Self::Search => "search",
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
}

impl WorkspaceSetup {
    pub fn record_last_run(
        &mut self,
        mode: LastRunMode,
        plot_config_override: Option<&Path>,
        analysis_config_override: Option<&Path>,
        search_output_override: Option<&Path>,
        search_top_override: Option<usize>,
    ) -> Result<(), String> {
        self.app_config.last_run = LastRunConfig {
            mode: mode.as_str().to_string(),
            plot_config_override: plot_config_override.map(path_to_string),
            analysis_config_override: analysis_config_override.map(path_to_string),
            search_output_override: search_output_override.map(path_to_string),
            search_top_override,
        };
        save_app_config(&self.paths.app_config_path, &self.app_config)
    }
}

pub fn prepare_workspace(root: &Path) -> Result<WorkspaceSetup, String> {
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
    };
    let mut warnings = Vec::new();

    fs::create_dir_all(&paths.config_dir)
        .map_err(|error| format!("failed to create {}: {error}", paths.config_dir.display()))?;
    fs::create_dir_all(&paths.data_dir)
        .map_err(|error| format!("failed to create {}: {error}", paths.data_dir.display()))?;
    fs::create_dir_all(&paths.output_dir)
        .map_err(|error| format!("failed to create {}: {error}", paths.output_dir.display()))?;
    fs::create_dir_all(&paths.logs_dir)
        .map_err(|error| format!("failed to create {}: {error}", paths.logs_dir.display()))?;

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

    let app_config = load_app_config(&paths.app_config_path, &mut warnings)?;
    save_app_config(&paths.app_config_path, &app_config)?;

    Ok(WorkspaceSetup {
        paths,
        warnings,
        app_config,
    })
}

fn load_app_config(path: &Path, warnings: &mut Vec<String>) -> Result<AppConfig, String> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }

    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
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
                format!(
                    "failed to parse {}: {error}; additionally failed to move corrupt file: {rename_error}",
                    path.display()
                )
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

fn save_app_config(path: &Path, config: &AppConfig) -> Result<(), String> {
    let text = toml::to_string_pretty(config)
        .map_err(|error| format!("failed to serialize app config: {error}"))?;
    atomic_write(path, &text)
}

fn ensure_runtime_config_file(
    root: &Path,
    target_path: &Path,
    legacy_relative_path: &str,
    default_content: &str,
    label: &str,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    if target_path.exists() {
        return Ok(());
    }

    let legacy_path = root.join(legacy_relative_path);
    if legacy_path.exists() {
        let content = fs::read_to_string(&legacy_path).map_err(|error| {
            format!(
                "failed to read legacy {} {}: {error}",
                label,
                legacy_path.display()
            )
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

fn atomic_write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, text)
        .map_err(|error| format!("failed to write {}: {error}", tmp_path.display()))?;
    fs::rename(&tmp_path, path).map_err(|error| {
        format!(
            "failed to move {} to {}: {error}",
            tmp_path.display(),
            path.display()
        )
    })?;
    Ok(())
}

const DEFAULT_PLOTTING_CONFIG: &str = r#"schema_version = 1

[shared]
input_is_directory = true
output_path = "../output"
output_prefix = ""
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
