//! Independent calibration configuration, validation, and precedence resolution.

use crate::domain::ConfigurationError;
use crate::results::calibration::{
    ActivityModelKind, CalibrationBranch, CalibrationModelKind, CalibrationPotentialSource,
    CalibrationSelectionCriterion, CrossValidationMode, EnvironmentalAlignment, NernstSlopeMode,
    ResponseSign, TemperatureMode, WeightingMode,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub const CALIBRATION_CONFIG_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_CALIBRATION_CONFIG_PATH: &str = "config/calibration.toml";

#[derive(Debug, Clone)]
pub struct LoadedCalibrationConfig {
    pub config: ResolvedCalibrationConfig,
    pub source_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ResolvedCalibrationConfig {
    pub schema_version: u32,
    pub observation_extraction: ObservationExtractionConfig,
    pub analyte: AnalyteConfig,
    pub temperature: TemperatureConfig,
    pub activity: ActivityConfig,
    pub nernst: NernstConfig,
    pub nicolsky_eisenman: NicolskyEisenmanConfig,
    pub hysteresis: HysteresisConfig,
    pub weighting: WeightingConfig,
    pub models: CalibrationModelsConfig,
    pub selection: CalibrationSelectionConfig,
    pub validation: CalibrationValidationConfig,
    pub uncertainty: CalibrationUncertaintyConfig,
    pub plotting: CalibrationPlottingConfig,
    pub export: CalibrationExportConfig,
    pub source_path: Option<PathBuf>,
}

impl Default for ResolvedCalibrationConfig {
    fn default() -> Self {
        Self {
            schema_version: CALIBRATION_CONFIG_SCHEMA_VERSION,
            observation_extraction: ObservationExtractionConfig::default(),
            analyte: AnalyteConfig::default(),
            temperature: TemperatureConfig::default(),
            activity: ActivityConfig::default(),
            nernst: NernstConfig::default(),
            nicolsky_eisenman: NicolskyEisenmanConfig::default(),
            hysteresis: HysteresisConfig::default(),
            weighting: WeightingConfig::default(),
            models: CalibrationModelsConfig::default(),
            selection: CalibrationSelectionConfig::default(),
            validation: CalibrationValidationConfig::default(),
            uncertainty: CalibrationUncertaintyConfig::default(),
            plotting: CalibrationPlottingConfig::default(),
            export: CalibrationExportConfig::default(),
            source_path: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ObservationExtractionConfig {
    pub preferred_source: CalibrationPotentialSource,
    pub allow_warning_fits: bool,
    pub fallback_source: Option<CalibrationPotentialSource>,
    pub steady_state_start_s: f64,
    pub steady_state_end_s: f64,
    pub minimum_points: usize,
    pub maximum_missing_fraction: f64,
    pub maximum_absolute_slope_v_per_s: f64,
}

impl Default for ObservationExtractionConfig {
    fn default() -> Self {
        Self {
            preferred_source: CalibrationPotentialSource::TransientEquilibrium,
            allow_warning_fits: true,
            fallback_source: None,
            steady_state_start_s: 180.0,
            steady_state_end_s: 300.0,
            minimum_points: 20,
            maximum_missing_fraction: 0.20,
            maximum_absolute_slope_v_per_s: 1e-5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AnalyteConfig {
    pub name: String,
    pub charge: i32,
    pub molar_mass_g_per_mol: Option<f64>,
}

impl Default for AnalyteConfig {
    fn default() -> Self {
        Self {
            name: "auto".to_string(),
            charge: 1,
            molar_mass_g_per_mol: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TemperatureConfig {
    pub mode: TemperatureMode,
    pub default_celsius: f64,
    pub reference_celsius: f64,
    pub environmental_series: Option<String>,
    pub alignment: EnvironmentalAlignment,
    pub maximum_gap_s: f64,
}

impl Default for TemperatureConfig {
    fn default() -> Self {
        Self {
            mode: TemperatureMode::ObservationSpecific,
            default_celsius: 25.0,
            reference_celsius: 25.0,
            environmental_series: Some("temperature".to_string()),
            alignment: EnvironmentalAlignment::LinearInterpolation,
            maximum_gap_s: 30.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActivityConfig {
    pub model: ActivityModelKind,
    pub davies: DaviesConfig,
    pub extended_debye_huckel: ExtendedDebyeHuckelConfig,
    pub conductivity_empirical: ConductivityEmpiricalConfig,
    pub user_provided_activity_field: String,
    pub solution_composition: Vec<SolutionComponentConfig>,
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            model: ActivityModelKind::Ideal,
            davies: DaviesConfig::default(),
            extended_debye_huckel: ExtendedDebyeHuckelConfig::default(),
            conductivity_empirical: ConductivityEmpiricalConfig::default(),
            user_provided_activity_field: "activity".to_string(),
            solution_composition: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SolutionComponentConfig {
    pub name: String,
    pub concentration_mol_l: f64,
    pub charge: i32,
}

impl Default for SolutionComponentConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            concentration_mol_l: 0.0,
            charge: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DaviesConfig {
    pub maximum_ionic_strength_mol_l: f64,
    pub a_constant: f64,
}

impl Default for DaviesConfig {
    fn default() -> Self {
        Self {
            maximum_ionic_strength_mol_l: 0.5,
            a_constant: 0.509,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExtendedDebyeHuckelConfig {
    pub a_constant: f64,
    pub b_constant: f64,
    pub ion_size_parameter: Option<f64>,
    pub ion_size_unit: String,
    pub maximum_ionic_strength_mol_l: f64,
}

impl Default for ExtendedDebyeHuckelConfig {
    fn default() -> Self {
        Self {
            a_constant: 0.509,
            b_constant: 0.328,
            ion_size_parameter: None,
            ion_size_unit: "angstrom".to_string(),
            maximum_ionic_strength_mol_l: 0.1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConductivityEmpiricalConfig {
    pub enabled: bool,
    pub conductivity_series: Option<String>,
    pub form: String,
    pub b0: f64,
    pub b1: f64,
    pub fit_b1: bool,
    pub minimum_conductivity_s_per_m: Option<f64>,
    pub maximum_conductivity_s_per_m: Option<f64>,
}

impl Default for ConductivityEmpiricalConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            conductivity_series: Some("conductivity".to_string()),
            form: "linear_log_activity_correction".to_string(),
            b0: 0.0,
            b1: 0.0,
            fit_b1: false,
            minimum_conductivity_s_per_m: None,
            maximum_conductivity_s_per_m: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NernstConfig {
    pub slope_mode: NernstSlopeMode,
    pub response_sign: ResponseSign,
    pub prior_slope_v_per_decade: Option<f64>,
    pub prior_standard_deviation_v_per_decade: Option<f64>,
}

impl Default for NernstConfig {
    fn default() -> Self {
        Self {
            slope_mode: NernstSlopeMode::Free,
            response_sign: ResponseSign::Auto,
            prior_slope_v_per_decade: None,
            prior_standard_deviation_v_per_decade: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NicolskyEisenmanConfig {
    pub enabled: bool,
    pub fit_selectivity_coefficients: bool,
    pub interferents: Vec<InterferentConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InterferentConfig {
    pub name: String,
    pub charge: i32,
    pub selectivity_coefficient: Option<f64>,
    pub source: String,
}

impl Default for InterferentConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            charge: 1,
            selectivity_coefficient: None,
            source: "user_supplied".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct HysteresisConfig {
    pub analyze: bool,
    pub log_activity_matching_tolerance: f64,
    pub warning_threshold_v: f64,
}

impl Default for HysteresisConfig {
    fn default() -> Self {
        Self {
            analyze: true,
            log_activity_matching_tolerance: 0.05,
            warning_threshold_v: 0.010,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WeightingConfig {
    pub mode: WeightingMode,
    pub minimum_standard_error_v: f64,
}

impl Default for WeightingConfig {
    fn default() -> Self {
        Self {
            mode: WeightingMode::PotentialStandardError,
            minimum_standard_error_v: 1e-6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalibrationModelsConfig {
    pub enabled: Vec<CalibrationModelKind>,
}

impl Default for CalibrationModelsConfig {
    fn default() -> Self {
        Self {
            enabled: vec![CalibrationModelKind::Nernst],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalibrationSelectionConfig {
    pub criterion: CalibrationSelectionCriterion,
    pub branch: CalibrationBranch,
}

impl Default for CalibrationSelectionConfig {
    fn default() -> Self {
        Self {
            criterion: CalibrationSelectionCriterion::Aicc,
            branch: CalibrationBranch::Mixed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalibrationValidationConfig {
    pub mode: CrossValidationMode,
    pub folds: usize,
    pub seed: u64,
    pub prediction_interval_confidence: f64,
}

impl Default for CalibrationValidationConfig {
    fn default() -> Self {
        Self {
            mode: CrossValidationMode::LeaveOneConcentrationLevelOut,
            folds: 5,
            seed: 42,
            prediction_interval_confidence: 0.95,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalibrationUncertaintyConfig {
    pub confidence_level: f64,
    pub bootstrap_iterations: usize,
    pub seed: u64,
    pub minimum_success_fraction: f64,
}

impl Default for CalibrationUncertaintyConfig {
    fn default() -> Self {
        Self {
            confidence_level: 0.95,
            bootstrap_iterations: 1000,
            seed: 42,
            minimum_success_fraction: 0.80,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalibrationPlottingConfig {
    pub enabled: bool,
    pub include_residuals: bool,
    pub include_hysteresis: bool,
    pub include_validation: bool,
    pub include_confidence_band: bool,
}

impl Default for CalibrationPlottingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_residuals: true,
            include_hysteresis: true,
            include_validation: true,
            include_confidence_band: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalibrationExportConfig {
    pub observations_filename: String,
    pub model_filename: String,
    pub results_filename: String,
    pub features_filename: String,
    pub residuals_filename: String,
    pub validation_filename: String,
    pub report_filename: String,
}

impl Default for CalibrationExportConfig {
    fn default() -> Self {
        Self {
            observations_filename: "calibration_observations.json".to_string(),
            model_filename: "calibration_model.json".to_string(),
            results_filename: "calibration_results.json".to_string(),
            features_filename: "calibration_summary.csv".to_string(),
            residuals_filename: "calibration_residuals.csv".to_string(),
            validation_filename: "calibration_validation.csv".to_string(),
            report_filename: "calibration_report.txt".to_string(),
        }
    }
}

impl ResolvedCalibrationConfig {
    pub fn load(
        workspace_dir: &Path,
        override_path: Option<&Path>,
    ) -> Result<LoadedCalibrationConfig, ConfigurationError> {
        let path = override_path
            .map(|value| resolve_path(value, workspace_dir))
            .unwrap_or_else(|| workspace_dir.join(DEFAULT_CALIBRATION_CONFIG_PATH));
        if override_path.is_some() && !path.exists() {
            return Err(ConfigurationError::invalid(format!(
                "calibration config override does not exist: {}",
                path.display()
            )));
        }
        if !path.exists() {
            let config = Self::default();
            config.validate()?;
            return Ok(LoadedCalibrationConfig {
                config,
                source_path: None,
                warnings: Vec::new(),
            });
        }
        let text =
            fs::read_to_string(&path).map_err(|error| ConfigurationError::io(&path, error))?;
        let mut config: Self = toml::from_str(&text).map_err(|error| {
            ConfigurationError::invalid(format!(
                "failed to parse calibration config {}: {error}",
                path.display()
            ))
        })?;
        config.source_path = Some(path.clone());
        config.validate()?;
        let mut warnings = Vec::new();
        if config.schema_version != CALIBRATION_CONFIG_SCHEMA_VERSION {
            warnings.push(format!(
                "calibration config schema_version {} does not match supported version {}",
                config.schema_version, CALIBRATION_CONFIG_SCHEMA_VERSION
            ));
        }
        Ok(LoadedCalibrationConfig {
            config,
            source_path: Some(path),
            warnings,
        })
    }

    pub fn apply_cli_overrides(
        &mut self,
        model: Option<&str>,
        selection: Option<&str>,
        bootstrap: Option<usize>,
        seed: Option<u64>,
    ) -> Result<(), ConfigurationError> {
        if let Some(model) = model {
            self.models.enabled = if model.eq_ignore_ascii_case("all") {
                vec![
                    CalibrationModelKind::Nernst,
                    CalibrationModelKind::NicolskyEisenman,
                    CalibrationModelKind::ConductivityEmpirical,
                ]
            } else {
                vec![
                    model
                        .parse()
                        .map_err(|error: String| ConfigurationError::invalid(error))?,
                ]
            };
        }
        if let Some(selection) = selection {
            self.selection.criterion = selection
                .parse()
                .map_err(|error: String| ConfigurationError::invalid(error))?;
        }
        if let Some(value) = bootstrap {
            self.uncertainty.bootstrap_iterations = value;
        }
        if let Some(value) = seed {
            self.uncertainty.seed = value;
            self.validation.seed = value;
        }
        self.validate()
    }

    pub fn validate(&self) -> Result<(), ConfigurationError> {
        let extraction = &self.observation_extraction;
        if !extraction.steady_state_start_s.is_finite()
            || !extraction.steady_state_end_s.is_finite()
            || extraction.steady_state_start_s < 0.0
            || extraction.steady_state_end_s <= extraction.steady_state_start_s
            || extraction.minimum_points == 0
            || !(0.0..=1.0).contains(&extraction.maximum_missing_fraction)
            || !extraction.maximum_absolute_slope_v_per_s.is_finite()
            || extraction.maximum_absolute_slope_v_per_s < 0.0
        {
            return Err(ConfigurationError::invalid(
                "observation extraction windows or thresholds are invalid",
            ));
        }
        if self.analyte.name.trim().is_empty()
            || self.analyte.charge == 0
            || self
                .analyte
                .molar_mass_g_per_mol
                .is_some_and(|value| !value.is_finite() || value <= 0.0)
        {
            return Err(ConfigurationError::invalid(
                "analyte charge must be nonzero and molar mass must be positive",
            ));
        }
        if !self.temperature.default_celsius.is_finite()
            || !self.temperature.reference_celsius.is_finite()
            || self.temperature.default_celsius <= -273.15
            || self.temperature.reference_celsius <= -273.15
            || !self.temperature.maximum_gap_s.is_finite()
            || self.temperature.maximum_gap_s <= 0.0
        {
            return Err(ConfigurationError::invalid(
                "temperature defaults or environmental gap are invalid",
            ));
        }
        if !self
            .activity
            .davies
            .maximum_ionic_strength_mol_l
            .is_finite()
            || !self.activity.davies.a_constant.is_finite()
            || !self
                .activity
                .extended_debye_huckel
                .maximum_ionic_strength_mol_l
                .is_finite()
            || !self.activity.extended_debye_huckel.a_constant.is_finite()
            || !self.activity.extended_debye_huckel.b_constant.is_finite()
            || self.activity.davies.maximum_ionic_strength_mol_l <= 0.0
            || self.activity.davies.a_constant <= 0.0
            || self
                .activity
                .extended_debye_huckel
                .maximum_ionic_strength_mol_l
                <= 0.0
            || self.activity.extended_debye_huckel.a_constant <= 0.0
            || self.activity.extended_debye_huckel.b_constant <= 0.0
        {
            return Err(ConfigurationError::invalid(
                "activity-model constants and validity ranges must be positive",
            ));
        }
        if self.activity.model == ActivityModelKind::ExtendedDebyeHuckel
            && self
                .activity
                .extended_debye_huckel
                .ion_size_parameter
                .is_none_or(|value| !value.is_finite() || value <= 0.0)
        {
            return Err(ConfigurationError::invalid(
                "extended Debye-Huckel requires ion_size_parameter",
            ));
        }
        if self.activity.model == ActivityModelKind::ExtendedDebyeHuckel
            && !matches!(
                self.activity
                    .extended_debye_huckel
                    .ion_size_unit
                    .trim()
                    .to_ascii_lowercase()
                    .as_str(),
                "angstrom" | "å" | "a" | "nm" | "nanometer" | "nanometers"
            )
        {
            return Err(ConfigurationError::invalid(
                "extended Debye-Huckel ion_size_unit must be angstrom or nm",
            ));
        }
        if self.activity.model == ActivityModelKind::ConductivityEmpirical
            && (!self.activity.conductivity_empirical.enabled
                || self
                    .activity
                    .conductivity_empirical
                    .conductivity_series
                    .is_none())
        {
            return Err(ConfigurationError::invalid(
                "conductivity empirical activity requires enabled=true and a series",
            ));
        }
        if !self.activity.conductivity_empirical.b0.is_finite()
            || !self.activity.conductivity_empirical.b1.is_finite()
            || self
                .activity
                .conductivity_empirical
                .minimum_conductivity_s_per_m
                .is_some_and(|value| !value.is_finite() || value < 0.0)
            || self
                .activity
                .conductivity_empirical
                .maximum_conductivity_s_per_m
                .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            return Err(ConfigurationError::invalid(
                "empirical conductivity coefficients and ranges must be finite and nonnegative where applicable",
            ));
        }
        if self.activity.solution_composition.iter().any(|component| {
            component.name.trim().is_empty()
                || !component.concentration_mol_l.is_finite()
                || component.concentration_mol_l < 0.0
                || component.charge == 0
        }) {
            return Err(ConfigurationError::invalid(
                "solution composition requires nonnegative finite concentrations and nonzero charges",
            ));
        }
        if self.models.enabled.is_empty() {
            return Err(ConfigurationError::invalid(
                "models.enabled must not be empty",
            ));
        }
        if self.nicolsky_eisenman.enabled {
            if self
                .nicolsky_eisenman
                .interferents
                .iter()
                .any(|item| item.name.trim().is_empty() || item.charge == 0)
            {
                return Err(ConfigurationError::invalid(
                    "Nicolsky-Eisenman interferents require names and nonzero charges",
                ));
            }
            if self
                .nicolsky_eisenman
                .interferents
                .iter()
                .any(|item| item.selectivity_coefficient.is_none())
                && !self.nicolsky_eisenman.fit_selectivity_coefficients
            {
                return Err(ConfigurationError::invalid(
                    "fixed Nicolsky-Eisenman interferents require selectivity coefficients",
                ));
            }
        }
        if !self.hysteresis.log_activity_matching_tolerance.is_finite()
            || !self.hysteresis.warning_threshold_v.is_finite()
            || !self.weighting.minimum_standard_error_v.is_finite()
            || self.hysteresis.log_activity_matching_tolerance <= 0.0
            || self.hysteresis.warning_threshold_v < 0.0
            || self.weighting.minimum_standard_error_v <= 0.0
        {
            return Err(ConfigurationError::invalid(
                "hysteresis and weighting thresholds are invalid",
            ));
        }
        if self.validation.folds == 0
            || !self.validation.prediction_interval_confidence.is_finite()
            || !(0.0..1.0).contains(&self.validation.prediction_interval_confidence)
        {
            return Err(ConfigurationError::invalid(
                "validation folds or confidence are invalid",
            ));
        }
        if !self.uncertainty.confidence_level.is_finite()
            || !self.uncertainty.minimum_success_fraction.is_finite()
            || !(0.0..1.0).contains(&self.uncertainty.confidence_level)
            || !(0.0..=1.0).contains(&self.uncertainty.minimum_success_fraction)
        {
            return Err(ConfigurationError::invalid(
                "uncertainty ranges are invalid",
            ));
        }
        if self.nernst.slope_mode == NernstSlopeMode::PriorConstrained
            && (self.nernst.prior_slope_v_per_decade.is_none()
                || self.nernst.prior_standard_deviation_v_per_decade.is_none())
        {
            return Err(ConfigurationError::invalid(
                "prior-constrained Nernst mode requires prior slope and standard deviation",
            ));
        }
        let filenames = [
            &self.export.observations_filename,
            &self.export.model_filename,
            &self.export.results_filename,
            &self.export.features_filename,
            &self.export.residuals_filename,
            &self.export.validation_filename,
            &self.export.report_filename,
        ];
        if filenames.iter().any(|value| value.trim().is_empty()) {
            return Err(ConfigurationError::invalid(
                "calibration export filenames must not be empty",
            ));
        }
        Ok(())
    }
}

fn resolve_path(path: &Path, workspace_dir: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_dir.join(path)
    }
}

impl FromStr for CalibrationModelKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "nernst" | "linear" => Ok(Self::Nernst),
            "nicolsky-eisenman" | "nicolsky_eisenman" | "ne" => Ok(Self::NicolskyEisenman),
            "conductivity-empirical" | "conductivity_empirical" | "empirical-conductivity" => {
                Ok(Self::ConductivityEmpirical)
            }
            other => Err(format!("unsupported calibration model '{other}'")),
        }
    }
}

impl FromStr for CalibrationSelectionCriterion {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "aic" => Ok(Self::Aic),
            "aicc" => Ok(Self::Aicc),
            "bic" => Ok(Self::Bic),
            "cross-validation" | "cross_validation" | "cv" => Ok(Self::CrossValidation),
            other => Err(format!(
                "unsupported calibration selection criterion '{other}'"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ResolvedCalibrationConfig;

    #[test]
    fn default_calibration_configuration_is_valid() {
        ResolvedCalibrationConfig::default()
            .validate()
            .expect("defaults");
    }
}
