//! Dependency-clean direction adapter from legacy estimation into the compiled ISM graph.

use super::{
    calibration_adapter::{CalibrationObservationModel, StoredCalibrationObservationModel},
    environment::AlignedEnvironment,
    error::EstimationError,
    state::{StateDefinition, StateTransform},
};
use crate::{
    estimation_config::ResolvedEstimationConfig,
    model::{
        ComponentBindings, ComponentDescriptor, ComponentFactory, ComponentRegistry, ComponentRole,
        EvidenceRequirement, InputSpec, InputValue, InterpretationStatus, IsmComponent, Jacobian,
        ModelDefinition, ModelError, ModelInput, ModelState, ParameterSpec, ParameterValueSource,
        ParameterValues, StateInitializationSource, StateJacobian, StateSpec, StateTransformation,
        UncertaintySpec, compile_model,
    },
    results::StoredCalibrationModel,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Provenance for the only supported custom-definition resolution path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ResolvedModelDefinitionSource {
    Profile(crate::estimation_config::CompiledEstimationProfile),
    File { path: PathBuf, sha256: String },
}

pub type InputId = String;

/// Standard environmental channels understood by the estimation boundary.
/// User-defined channels are represented by `EnvironmentNamed` below.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentSource {
    Temperature,
    Conductivity,
    IonicStrength,
    Flow,
    PolarizationInput,
    Interferent(String),
}

/// Typed origin of a compiled model input. The declaration string is kept in
/// `InputBindingProvenance`; this enum is the only runtime dispatch path.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ModelInputSource {
    EstimatedTargetActivity,
    ActivityStep,
    TransductionDrive,
    Environment(EnvironmentSource),
    EnvironmentNamed(String),
    EventField { field: String },
    Constant { value: f64, unit: String },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum InputUnitConversion {
    Identity,
    Converted {
        source_unit: String,
        target_unit: String,
    },
    Deferred,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InputBindingProvenance {
    pub target_input_id: InputId,
    pub source_declaration: String,
    pub model_id: String,
    pub expected_unit: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResolvedInputBinding {
    pub target_input_id: InputId,
    pub source: ModelInputSource,
    pub target_unit: String,
    pub source_unit: Option<String>,
    pub conversion: InputUnitConversion,
    pub provenance: InputBindingProvenance,
    pub required: bool,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResolvedModelInputBindings {
    pub bindings: BTreeMap<InputId, ResolvedInputBinding>,
}

impl ResolvedModelInputBindings {
    pub fn binding(&self, input_id: &str) -> Option<&ResolvedInputBinding> {
        self.bindings.get(input_id)
    }
}

#[derive(Debug, Clone)]
struct RuntimeInputValue {
    value: f64,
    unit: String,
}

fn standard_binding_declarations(config: &ResolvedEstimationConfig) -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "target_activity".into(),
            config.model.input_bindings.target_activity.clone(),
        ),
        (
            "delta_log10_activity".into(),
            config.model.input_bindings.delta_log10_activity.clone(),
        ),
        (
            "temperature".into(),
            config.model.input_bindings.temperature.clone(),
        ),
        (
            "conductivity".into(),
            config.model.input_bindings.conductivity.clone(),
        ),
    ])
}

fn default_binding_declaration(input_id: &str) -> Option<&'static str> {
    match input_id {
        "target_activity" => Some("estimated_activity"),
        "delta_log10_activity" => Some("experiment_activity_step"),
        "temperature" => Some("environment:temperature"),
        "conductivity" => Some("environment:conductivity"),
        "ionic_strength" => Some("environment:ionic_strength"),
        "flow" => Some("environment:flow"),
        "polarization_input_v" => Some("environment:polarization_input_v"),
        "transduction_drive" => Some("transduction_drive"),
        input_id if input_id.starts_with("interferent.") => Some("environment:interferent"),
        _ => None,
    }
}

fn unsupported_source(
    target_input_id: &str,
    source_declaration: &str,
    model_id: &str,
) -> EstimationError {
    EstimationError::UnsupportedModelInputSource {
        target_input_id: target_input_id.into(),
        source_declaration: source_declaration.into(),
        model_id: model_id.into(),
    }
}

fn parse_constant(
    target_input_id: &str,
    source_declaration: &str,
    model_id: &str,
    payload: &str,
) -> Result<ModelInputSource, EstimationError> {
    let payload = payload.trim();
    let (value_text, unit) = if let Some((value, unit)) = payload.split_once(',') {
        (value.trim().to_string(), unit.trim().to_string())
    } else if let Some((value, unit)) = payload.split_once(':') {
        (value.trim().to_string(), unit.trim().to_string())
    } else {
        let mut parts = payload.split_whitespace();
        let value = parts.next().unwrap_or_default().to_string();
        (value, parts.collect::<Vec<_>>().join(" "))
    };
    let value = value_text
        .parse::<f64>()
        .map_err(|_| unsupported_source(target_input_id, source_declaration, model_id))?;
    if !value.is_finite() || unit.trim().is_empty() {
        return Err(unsupported_source(
            target_input_id,
            source_declaration,
            model_id,
        ));
    }
    Ok(ModelInputSource::Constant {
        value,
        unit: unit.trim().into(),
    })
}

fn parse_binding_source(
    target_input_id: &str,
    source_declaration: &str,
    model_id: &str,
) -> Result<ModelInputSource, EstimationError> {
    let declaration = source_declaration.trim();
    if declaration.is_empty() {
        return Err(unsupported_source(
            target_input_id,
            source_declaration,
            model_id,
        ));
    }
    match declaration {
        "estimated_activity" | "target_activity" => Ok(ModelInputSource::EstimatedTargetActivity),
        "experiment_activity_step" | "activity_step" | "delta_log10_activity" => {
            Ok(ModelInputSource::ActivityStep)
        }
        "transduction_drive" => Ok(ModelInputSource::TransductionDrive),
        _ if declaration.starts_with("environment:") => {
            let name = declaration.trim_start_matches("environment:").trim();
            if name.is_empty() {
                return Err(unsupported_source(
                    target_input_id,
                    source_declaration,
                    model_id,
                ));
            }
            let source = match name {
                "temperature" => EnvironmentSource::Temperature,
                "conductivity" => EnvironmentSource::Conductivity,
                "ionic_strength" | "ionic-strength" => EnvironmentSource::IonicStrength,
                "flow" => EnvironmentSource::Flow,
                "polarization_input_v" => EnvironmentSource::PolarizationInput,
                "interferent" => EnvironmentSource::Interferent(
                    target_input_id
                        .strip_prefix("interferent.")
                        .unwrap_or(target_input_id)
                        .into(),
                ),
                value if value.starts_with("interferent.") => {
                    EnvironmentSource::Interferent(value.trim_start_matches("interferent.").into())
                }
                value => return Ok(ModelInputSource::EnvironmentNamed(value.into())),
            };
            Ok(ModelInputSource::Environment(source))
        }
        _ if declaration.starts_with("event:") || declaration.starts_with("event_field:") => {
            let field = declaration
                .split_once(':')
                .map(|(_, field)| field.trim())
                .unwrap_or_default();
            if field.is_empty() {
                return Err(unsupported_source(
                    target_input_id,
                    source_declaration,
                    model_id,
                ));
            }
            Ok(ModelInputSource::EventField {
                field: field.into(),
            })
        }
        _ if declaration.starts_with("constant:") => parse_constant(
            target_input_id,
            source_declaration,
            model_id,
            declaration.trim_start_matches("constant:"),
        ),
        _ if declaration.starts_with("constant(") && declaration.ends_with(')') => parse_constant(
            target_input_id,
            source_declaration,
            model_id,
            &declaration[9..declaration.len() - 1],
        ),
        _ => Err(unsupported_source(
            target_input_id,
            source_declaration,
            model_id,
        )),
    }
}

fn source_unit(source: &ModelInputSource) -> Option<String> {
    match source {
        ModelInputSource::EstimatedTargetActivity
        | ModelInputSource::ActivityStep
        | ModelInputSource::TransductionDrive => Some("activity".into()),
        ModelInputSource::Environment(source) => Some(
            match source {
                EnvironmentSource::Temperature => "K",
                EnvironmentSource::Conductivity => "S/m",
                EnvironmentSource::IonicStrength => "mol/L",
                EnvironmentSource::Flow => "m/s",
                EnvironmentSource::PolarizationInput => "V",
                EnvironmentSource::Interferent(_) => "activity",
            }
            .into(),
        ),
        ModelInputSource::EnvironmentNamed(_) | ModelInputSource::EventField { .. } => None,
        ModelInputSource::Constant { unit, .. } => Some(unit.clone()),
    }
}

fn unit_conversion(source_unit: &str, target_unit: &str) -> Option<InputUnitConversion> {
    if crate::model::units_compatible(source_unit, target_unit) {
        return Some(InputUnitConversion::Identity);
    }
    can_convert_units(source_unit, target_unit).then_some(InputUnitConversion::Converted {
        source_unit: source_unit.into(),
        target_unit: target_unit.into(),
    })
}

fn normalized_unit(unit: &str) -> String {
    unit.trim().to_ascii_lowercase().replace(['μ', 'µ'], "u")
}

fn can_convert_units(source_unit: &str, target_unit: &str) -> bool {
    let source = normalized_unit(source_unit);
    let target = normalized_unit(target_unit);
    (matches!(source.as_str(), "c" | "°c" | "degc" | "celsius") && target == "k")
        || (source == "k" && matches!(target.as_str(), "c" | "°c" | "degc" | "celsius"))
        || (matches!(source.as_str(), "v" | "mv" | "uv")
            && matches!(target.as_str(), "v" | "mv" | "uv"))
        || (matches!(source.as_str(), "s/m" | "s/cm" | "ms/cm" | "us/cm")
            && matches!(target.as_str(), "s/m" | "s/cm" | "ms/cm" | "us/cm"))
        || (matches!(source.as_str(), "m/s" | "cm/s" | "mm/s")
            && matches!(target.as_str(), "m/s" | "cm/s" | "mm/s"))
}

fn convert_value(value: f64, source_unit: &str, target_unit: &str) -> Option<f64> {
    if crate::model::units_compatible(source_unit, target_unit) {
        return Some(value);
    }
    let source = normalized_unit(source_unit);
    let target = normalized_unit(target_unit);
    let result = if matches!(source.as_str(), "c" | "°c" | "degc" | "celsius") && target == "k" {
        value + 273.15
    } else if source == "k" && matches!(target.as_str(), "c" | "°c" | "degc" | "celsius") {
        value - 273.15
    } else if matches!(source.as_str(), "v" | "mv" | "uv")
        && matches!(target.as_str(), "v" | "mv" | "uv")
    {
        let volts = value
            * match source.as_str() {
                "mv" => 1e-3,
                "uv" => 1e-6,
                _ => 1.0,
            };
        volts
            * match target.as_str() {
                "mv" => 1e3,
                "uv" => 1e6,
                _ => 1.0,
            }
    } else if matches!(source.as_str(), "s/m" | "s/cm" | "ms/cm" | "us/cm")
        && matches!(target.as_str(), "s/m" | "s/cm" | "ms/cm" | "us/cm")
    {
        let siemens_per_m = value
            * match source.as_str() {
                "s/cm" => 100.0,
                "ms/cm" => 0.1,
                "us/cm" => 1e-4,
                _ => 1.0,
            };
        siemens_per_m
            / match target.as_str() {
                "s/cm" => 100.0,
                "ms/cm" => 0.1,
                "us/cm" => 1e-4,
                _ => 1.0,
            }
    } else if matches!(source.as_str(), "m/s" | "cm/s" | "mm/s")
        && matches!(target.as_str(), "m/s" | "cm/s" | "mm/s")
    {
        let meters_per_s = value
            * match source.as_str() {
                "cm/s" => 1e-2,
                "mm/s" => 1e-3,
                _ => 1.0,
            };
        meters_per_s
            / match target.as_str() {
                "cm/s" => 1e-2,
                "mm/s" => 1e-3,
                _ => 1.0,
            }
    } else {
        return None;
    };
    result.is_finite().then_some(result)
}

/// Resolve every configured declaration against a compiled definition once.
/// Runtime evaluation consumes this deterministic map and never reparses the
/// source strings.
pub fn resolve_model_input_bindings(
    compiled: &crate::model::CompiledIsmModel,
    config: &ResolvedEstimationConfig,
) -> Result<ResolvedModelInputBindings, EstimationError> {
    let definition = compiled.definition();
    let model_id = definition.model_id.clone();
    let standard = standard_binding_declarations(config);
    for target_input_id in config.model.input_bindings.custom.keys() {
        if !definition
            .inputs
            .iter()
            .any(|input| input.id == *target_input_id)
        {
            return Err(EstimationError::UnknownModelInputBindingTarget {
                target_input_id: target_input_id.clone(),
                source_declaration: config
                    .model
                    .input_bindings
                    .custom
                    .get(target_input_id)
                    .cloned()
                    .unwrap_or_default(),
                model_id,
            });
        }
    }
    let mut bindings = BTreeMap::new();
    for input in &definition.inputs {
        let declaration = config
            .model
            .input_bindings
            .custom
            .get(&input.id)
            .cloned()
            .or_else(|| standard.get(&input.id).cloned())
            .or_else(|| default_binding_declaration(&input.id).map(str::to_string));
        let Some(declaration) = declaration else {
            if input.required {
                return Err(EstimationError::MissingModelInputSource {
                    target_input_id: input.id.clone(),
                    source_declaration: String::new(),
                    expected_unit: input.unit.clone(),
                    model_id: model_id.clone(),
                });
            }
            continue;
        };
        let source = parse_binding_source(&input.id, &declaration, &model_id)?;
        let source_unit = source_unit(&source);
        let conversion = source_unit
            .as_deref()
            .map(|unit| {
                unit_conversion(unit, &input.unit).ok_or_else(|| {
                    EstimationError::ModelInputUnitMismatch {
                        target_input_id: input.id.clone(),
                        source_declaration: declaration.clone(),
                        expected_unit: input.unit.clone(),
                        actual_unit: unit.into(),
                        model_id: model_id.clone(),
                    }
                })
            })
            .transpose()?
            .unwrap_or(InputUnitConversion::Deferred);
        let binding = ResolvedInputBinding {
            target_input_id: input.id.clone(),
            source,
            target_unit: input.unit.clone(),
            source_unit,
            conversion,
            provenance: InputBindingProvenance {
                target_input_id: input.id.clone(),
                source_declaration: declaration,
                model_id: model_id.clone(),
                expected_unit: input.unit.clone(),
            },
            required: input.required,
        };
        if bindings.insert(input.id.clone(), binding).is_some() {
            return Err(EstimationError::DuplicateModelInputBinding {
                target_input_id: input.id.clone(),
                declarations: vec![input.id.clone()],
                model_id: model_id.clone(),
            });
        }
    }
    Ok(ResolvedModelInputBindings { bindings })
}

fn environment_value(
    environment: &AlignedEnvironment,
    source: &EnvironmentSource,
) -> Option<RuntimeInputValue> {
    match source {
        EnvironmentSource::Temperature => {
            environment.temperature_k.map(|value| RuntimeInputValue {
                value,
                unit: "K".into(),
            })
        }
        EnvironmentSource::Conductivity => {
            environment
                .conductivity_s_per_m
                .map(|value| RuntimeInputValue {
                    value,
                    unit: "S/m".into(),
                })
        }
        EnvironmentSource::IonicStrength => {
            environment
                .ionic_strength_mol_l
                .map(|value| RuntimeInputValue {
                    value,
                    unit: "mol/L".into(),
                })
        }
        EnvironmentSource::Flow => environment.flow.map(|value| RuntimeInputValue {
            value,
            unit: "m/s".into(),
        }),
        EnvironmentSource::PolarizationInput => {
            environment
                .polarization_input_v
                .map(|value| RuntimeInputValue {
                    value,
                    unit: "V".into(),
                })
        }
        EnvironmentSource::Interferent(id) => environment
            .interferent_activities
            .get(id)
            .copied()
            .map(|value| RuntimeInputValue {
                value,
                unit: "activity".into(),
            }),
    }
}

fn runtime_source_value(
    source: &ModelInputSource,
    estimated_log10_activity: Option<f64>,
    environment: &AlignedEnvironment,
) -> Option<RuntimeInputValue> {
    match source {
        ModelInputSource::EstimatedTargetActivity => estimated_log10_activity
            .map(|value| 10_f64.powf(value))
            .filter(|value| value.is_finite())
            .map(|value| RuntimeInputValue {
                value,
                unit: "activity".into(),
            }),
        ModelInputSource::ActivityStep => environment
            .delta_log10_activity
            .filter(|value| value.is_finite())
            .map(|value| RuntimeInputValue {
                value,
                unit: "activity".into(),
            }),
        ModelInputSource::TransductionDrive => environment
            .transduction_drive
            .filter(|value| value.is_finite())
            .map(|value| RuntimeInputValue {
                value,
                unit: "activity".into(),
            }),
        ModelInputSource::Environment(source) => environment_value(environment, source),
        ModelInputSource::EnvironmentNamed(name) => environment
            .values
            .iter()
            .find(|value| value.source_series == *name)
            .map(|value| RuntimeInputValue {
                value: value.value,
                unit: value
                    .source_unit
                    .clone()
                    .unwrap_or_else(|| "dimensionless".into()),
            }),
        ModelInputSource::EventField { field } => {
            environment
                .event_fields
                .get(field)
                .map(|value| RuntimeInputValue {
                    value: value.value,
                    unit: value.unit.clone(),
                })
        }
        ModelInputSource::Constant { value, unit } => Some(RuntimeInputValue {
            value: *value,
            unit: unit.clone(),
        }),
    }
}

/// Execute the resolved plan for one observation or transition. This is
/// shared by EKF, UKF, compiled simulation, and all estimate workflows.
pub fn execute_model_input_bindings(
    plan: &ResolvedModelInputBindings,
    estimated_log10_activity: Option<f64>,
    environment: &AlignedEnvironment,
) -> Result<ModelInput, EstimationError> {
    let mut input = ModelInput::empty(environment.timestamp_s);
    let model_id = plan
        .bindings
        .values()
        .next()
        .map(|binding| binding.provenance.model_id.clone())
        .unwrap_or_default();
    for binding in plan.bindings.values() {
        let Some(source_value) =
            runtime_source_value(&binding.source, estimated_log10_activity, environment)
        else {
            if binding.required {
                return Err(EstimationError::MissingModelInputSource {
                    target_input_id: binding.target_input_id.clone(),
                    source_declaration: binding.provenance.source_declaration.clone(),
                    expected_unit: binding.target_unit.clone(),
                    model_id: model_id.clone(),
                });
            }
            continue;
        };
        let value = convert_value(source_value.value, &source_value.unit, &binding.target_unit)
            .ok_or_else(|| EstimationError::ModelInputUnitMismatch {
                target_input_id: binding.target_input_id.clone(),
                source_declaration: binding.provenance.source_declaration.clone(),
                expected_unit: binding.target_unit.clone(),
                actual_unit: source_value.unit.clone(),
                model_id: model_id.clone(),
            })?;
        if !value.is_finite() {
            return Err(EstimationError::InvalidInput(format!(
                "compiled model input '{}' converted to a non-finite value",
                binding.target_input_id
            )));
        }
        input.values.insert(
            binding.target_input_id.clone(),
            InputValue {
                value,
                unit: binding.target_unit.clone(),
            },
        );
    }
    Ok(input)
}

pub fn compile_legacy_model(
    config: &ResolvedEstimationConfig,
    definitions: &[StateDefinition],
    tau_s: f64,
    tau_uncertainty_s: Option<f64>,
    calibration: &StoredCalibrationModel,
) -> Result<crate::model::CompiledIsmModel, EstimationError> {
    let definition =
        legacy_model_definition(config, definitions, tau_s, tau_uncertainty_s, calibration)?;
    let registry = ComponentRegistry::from_static_factories([
        ("estimation.legacy_equilibrium", factory as ComponentFactory),
        ("estimation.legacy_baseline", factory as ComponentFactory),
        (
            "estimation.legacy_polarization",
            factory as ComponentFactory,
        ),
        ("estimation.legacy_sensitivity", factory as ComponentFactory),
    ]);
    compile_model(definition, &registry)
        .map_err(|error| EstimationError::compiled("legacy compilation", error))
}

/// Builds the opt-in V1 reduced profile while keeping calibration equations in
/// the established stored-calibration adapter.  The dynamic/reference/
/// observation-variance components are the approved V1 built-ins; only the
/// calibration bridge is estimation-owned.
pub fn compile_reduced_v1_model(
    config: &ResolvedEstimationConfig,
    tau_s: f64,
    tau_uncertainty_s: Option<f64>,
    calibration: &StoredCalibrationModel,
) -> Result<super::model::StateModel, EstimationError> {
    let mut model = super::model::StateModel::new(config, tau_s, tau_uncertainty_s)?;
    let legacy = legacy_model_definition(
        config,
        &model.definitions,
        tau_s,
        tau_uncertainty_s,
        calibration,
    )?;
    let activity = legacy.states.first().cloned().ok_or_else(|| {
        EstimationError::config("reduced profile requires a log10 activity state")
    })?;
    let mut definition = if matches!(
        config.model.transduction_drive,
        crate::estimation_config::TransductionDriveSource::None
    ) {
        crate::model::reduced_ism_v1_definition()
    } else {
        crate::model::reduced_ism_v1_with_transduction_definition()
    };
    definition.states.insert(0, activity);
    definition
        .components
        .retain(|component| component.id != "equilibrium_nernst");
    let calibration_json = serde_json::to_string(calibration)
        .map_err(|error| EstimationError::config(format!("calibration serialization: {error}")))?;
    definition.components.insert(
        1,
        descriptor(
            "calibrated_equilibrium",
            "estimation.legacy_equilibrium",
            ComponentRole::Equilibrium,
            vec!["log10_activity"],
            Vec::new(),
            "equilibrium",
            BTreeMap::from([
                ("calibration_json".into(), calibration_json),
                ("activity_transform".into(), "Identity".into()),
            ]),
        ),
    );
    let mut registry = crate::model::built_in_registry().clone();
    registry
        .register("estimation.legacy_equilibrium", factory as ComponentFactory)
        .map_err(|error| EstimationError::config(format!("compiled registry: {error}")))?;
    let compiled = compile_model(definition, &registry)
        .map_err(|error| EstimationError::compiled("reduced-v1 compilation", error))?;
    let resolved_bindings = resolve_model_input_bindings(&compiled, config)?;
    model.definitions = compiled
        .state_definitions()
        .iter()
        .map(|binding| StateDefinition {
            name: binding.spec.id.clone(),
            unit: binding.spec.unit.clone(),
            transform: StateTransform::Identity,
            lower_bound: Some(binding.spec.lower_bound),
            upper_bound: Some(binding.spec.upper_bound),
            interpretation: binding.spec.description.clone(),
        })
        .collect();
    // Preserve the latent activity coordinate semantics even though the model
    // definition declares a dimensionless state.
    if let Some(activity) = model.definitions.first_mut() {
        activity.unit = "log10(activity)".into();
        activity.lower_bound = None;
        activity.upper_bound = None;
    }
    model.compiled_parameters = Some(compiled.default_parameters());
    model.compiled = Some(std::sync::Arc::new(compiled));
    model.resolved_input_bindings = Some(std::sync::Arc::new(resolved_bindings));
    model.definition_source = Some(ResolvedModelDefinitionSource::Profile(
        crate::estimation_config::CompiledEstimationProfile::ReducedIsmV1,
    ));
    Ok(model)
}

/// Loads and compiles a user definition through the approved core registry.
/// The configuration validator ensures this cannot be silently combined with
/// a built-in profile. `ModelConfig` performs schema migration and validation
/// before graph compilation.
pub fn compile_custom_model(
    config: &ResolvedEstimationConfig,
    tau_s: f64,
    tau_uncertainty_s: Option<f64>,
) -> Result<super::model::StateModel, EstimationError> {
    let configured_path =
        config.model.definition.as_ref().ok_or_else(|| {
            EstimationError::config("custom compiled profile has no definition path")
        })?;
    let path = if configured_path.is_absolute() {
        configured_path.clone()
    } else {
        config
            .source_path
            .as_deref()
            .and_then(|source| source.parent())
            .map(|base| base.join(configured_path))
            .unwrap_or_else(|| configured_path.clone())
    };
    let bytes = std::fs::read(&path).map_err(|source| EstimationError::io(&path, source))?;
    let definition = crate::model_config::ModelConfig::load(&path)
        .map_err(|error| {
            EstimationError::config(format!(
                "custom model definition '{}': {error}",
                path.display()
            ))
        })?
        .model;
    let compiled = compile_model(definition, crate::model::built_in_registry())
        .map_err(|error| EstimationError::compiled("custom-definition compilation", error))?;
    let resolved_bindings = resolve_model_input_bindings(&compiled, config)?;
    let mut model = super::model::StateModel::new(config, tau_s, tau_uncertainty_s)?;
    model.definitions = compiled
        .state_definitions()
        .iter()
        .map(|binding| StateDefinition {
            name: binding.spec.id.clone(),
            unit: binding.spec.unit.clone(),
            transform: StateTransform::Identity,
            lower_bound: Some(binding.spec.lower_bound),
            upper_bound: Some(binding.spec.upper_bound),
            interpretation: binding.spec.description.clone(),
        })
        .collect();
    if model.index("log10_activity").is_none() {
        return Err(EstimationError::config(
            "custom compiled definition must declare stable state ID 'log10_activity' for estimator-owned activity",
        ));
    }
    model.compiled_parameters = Some(compiled.default_parameters());
    model.compiled = Some(std::sync::Arc::new(compiled));
    model.resolved_input_bindings = Some(std::sync::Arc::new(resolved_bindings));
    model.definition_source = Some(ResolvedModelDefinitionSource::File {
        path,
        sha256: format!("{:x}", Sha256::digest(bytes)),
    });
    Ok(model)
}

pub fn legacy_model_definition(
    config: &ResolvedEstimationConfig,
    definitions: &[StateDefinition],
    tau_s: f64,
    tau_uncertainty_s: Option<f64>,
    calibration: &StoredCalibrationModel,
) -> Result<ModelDefinition, EstimationError> {
    let calibration_json = serde_json::to_string(calibration)
        .map_err(|error| EstimationError::config(format!("calibration serialization: {error}")))?;
    let activity_definition = definitions
        .first()
        .ok_or_else(|| EstimationError::config("legacy estimator has no activity state"))?;
    let states = definitions
        .iter()
        .map(|state| {
            let (initialization_source, initial_uncertainty) =
                state_initial_uncertainty(config, &state.name);
            StateSpec {
                id: state.name.clone(),
                name: state.name.replace('_', " "),
                description: state.interpretation.clone(),
                unit: if state.name == "log10_activity" {
                    "dimensionless".into()
                } else {
                    state.unit.clone()
                },
                transformation: StateTransformation::Custom(format!("{:?}", state.transform)),
                initialization_source,
                lower_bound: if matches!(state.transform, StateTransform::LogisticBounded) {
                    -30.0
                } else {
                    state.lower_bound.unwrap_or_else(|| {
                        if state.name == "log10_activity" {
                            -30.0
                        } else {
                            -10.0
                        }
                    })
                },
                upper_bound: if matches!(state.transform, StateTransform::LogisticBounded) {
                    30.0
                } else {
                    state.upper_bound.unwrap_or_else(|| {
                        if state.name == "log10_activity" {
                            30.0
                        } else {
                            10.0
                        }
                    })
                },
                initial_value: match state.name.as_str() {
                    "baseline_offset" => config.initialization.baseline_v,
                    "polarization" => config.initialization.polarization_v,
                    "sensitivity_scale"
                        if matches!(state.transform, StateTransform::LogisticBounded) =>
                    {
                        0.0
                    }
                    "sensitivity_scale" => config.initialization.condition_value,
                    _ => 0.0,
                },
                source: "legacy estimation state adapter".into(),
                process_equation_version: 1,
                observability_requirements: vec![
                    "Estimator observability must be retained in the compatibility report.".into(),
                ],
                validity_domain: state.interpretation.clone(),
                initial_uncertainty,
            }
        })
        .collect::<Vec<_>>();
    let mut parameters = Vec::new();
    let mut components = Vec::new();
    components.push(descriptor(
        "legacy.equilibrium",
        "estimation.legacy_equilibrium",
        ComponentRole::Equilibrium,
        vec!["log10_activity"],
        Vec::new(),
        "equilibrium",
        BTreeMap::from([
            ("calibration_json".into(), calibration_json.clone()),
            (
                "activity_transform".into(),
                format!("{:?}", activity_definition.transform),
            ),
        ]),
    ));
    if definitions
        .iter()
        .any(|state| state.name == "baseline_offset")
    {
        components.push(descriptor(
            "legacy.reference.baseline",
            "estimation.legacy_baseline",
            ComponentRole::Reference,
            vec!["baseline_offset"],
            Vec::new(),
            "reference",
            BTreeMap::new(),
        ));
    }
    if definitions.iter().any(|state| state.name == "polarization") {
        parameters.extend([
            parameter(
                "legacy_polarization_tau_s",
                "s",
                1e-12,
                1e12,
                tau_s,
                tau_uncertainty_s.map(|value| UncertaintySpec::StandardDeviation {
                    value,
                    unit: "s".into(),
                }),
            ),
            parameter(
                "legacy_polarization_gain",
                "dimensionless",
                -1e6,
                1e6,
                config.polarization.gain,
                None,
            ),
        ]);
        components.push(descriptor(
            "legacy.transport.polarization",
            "estimation.legacy_polarization",
            ComponentRole::Transport,
            vec!["polarization"],
            vec!["legacy_polarization_tau_s", "legacy_polarization_gain"],
            "transport",
            BTreeMap::new(),
        ));
    }
    if let Some(state) = definitions
        .iter()
        .find(|state| state.name == "sensitivity_scale")
    {
        components.push(descriptor(
            "legacy.transduction.sensitivity",
            "estimation.legacy_sensitivity",
            ComponentRole::Transduction,
            vec!["log10_activity", "sensitivity_scale"],
            Vec::new(),
            "transduction",
            BTreeMap::from([
                ("calibration_json".into(), calibration_json),
                (
                    "activity_transform".into(),
                    format!("{:?}", activity_definition.transform),
                ),
                (
                    "sensitivity_transform".into(),
                    format!("{:?}", state.transform),
                ),
                (
                    "sensitivity_lower".into(),
                    state.lower_bound.unwrap_or(0.5).to_string(),
                ),
                (
                    "sensitivity_upper".into(),
                    state.upper_bound.unwrap_or(1.5).to_string(),
                ),
            ]),
        ));
    }
    let mut inputs = vec![
        input("temperature", "K"),
        input("conductivity", "S/m"),
        input("ionic_strength", "mol/L"),
        input("flow", "m/s"),
        input("polarization_input_v", "V"),
    ];
    for coefficient in &calibration.selectivity_coefficients {
        inputs.push(input(
            &format!("interferent.{}", coefficient.interferent),
            "activity",
        ));
    }
    Ok(ModelDefinition {
        schema_version: crate::model::MODEL_DEFINITION_SCHEMA_VERSION,
        model_id: format!("legacy-estimation-{:?}", config.state_model.kind),
        description: "Compiled compatibility representation of the legacy estimation equations"
            .into(),
        validity_domain: "Stored calibration domain and configured legacy estimator bounds".into(),
        uncertainty_incomplete: true,
        states,
        parameters,
        inputs,
        components,
    })
}

pub fn model_input(environment: &AlignedEnvironment) -> ModelInput {
    let mut values = BTreeMap::new();
    let mut insert = |id: &str, value: Option<f64>, unit: &str| {
        if let Some(value) = value {
            values.insert(
                id.into(),
                InputValue {
                    value,
                    unit: unit.into(),
                },
            );
        }
    };
    insert("temperature", environment.temperature_k, "K");
    insert("conductivity", environment.conductivity_s_per_m, "S/m");
    insert("ionic_strength", environment.ionic_strength_mol_l, "mol/L");
    insert("flow", environment.flow, "m/s");
    insert(
        "polarization_input_v",
        environment.polarization_input_v,
        "V",
    );
    for (ion, activity) in &environment.interferent_activities {
        values.insert(
            format!("interferent.{ion}"),
            InputValue {
                value: *activity,
                unit: "activity".into(),
            },
        );
    }
    ModelInput {
        time_s: environment.timestamp_s,
        values,
    }
}

fn state_initial_uncertainty(
    config: &ResolvedEstimationConfig,
    state_id: &str,
) -> (StateInitializationSource, UncertaintySpec) {
    let (variance, unit, process_variance) = match state_id {
        "log10_activity" => (
            config.initial_covariance.log10_activity_variance,
            "dimensionless^2",
            config.process_noise.activity_variance_per_s,
        ),
        "baseline_offset" => (
            config.initial_covariance.baseline_variance_v2,
            "V^2",
            config.process_noise.baseline_variance_v2_per_s,
        ),
        "polarization" => (
            config.initial_covariance.polarization_variance_v2,
            "V^2",
            config.process_noise.polarization_variance_v2_per_s,
        ),
        "sensitivity_scale" => (
            config.initial_covariance.condition_variance,
            "dimensionless^2",
            config.process_noise.condition_variance_per_s,
        ),
        _ => (f64::NAN, "dimensionless^2", f64::NAN),
    };
    if variance == 0.0 && process_variance == 0.0 {
        (
            StateInitializationSource::DeclaredDefault,
            UncertaintySpec::Deterministic,
        )
    } else {
        (
            StateInitializationSource::Estimated,
            UncertaintySpec::Variance {
                value: variance,
                unit: unit.into(),
            },
        )
    }
}

fn factory(descriptor: &ComponentDescriptor) -> Result<Box<dyn IsmComponent>, ModelError> {
    let calibration = descriptor
        .metadata
        .get("calibration_json")
        .map(|text| {
            serde_json::from_str::<StoredCalibrationModel>(text)
                .map_err(|error| evaluation(descriptor, error))
                .and_then(|model| {
                    StoredCalibrationObservationModel::new(model)
                        .map_err(|error| evaluation(descriptor, error))
                })
        })
        .transpose()?;
    Ok(Box::new(LegacyComponent {
        descriptor: descriptor.clone(),
        bindings: ComponentBindings::default(),
        calibration,
    }))
}

struct LegacyComponent {
    descriptor: ComponentDescriptor,
    bindings: ComponentBindings,
    calibration: Option<StoredCalibrationObservationModel>,
}

impl LegacyComponent {
    fn state(&self, state: &ModelState, id: &str) -> Result<f64, ModelError> {
        self.bindings
            .state_indices
            .get(id)
            .and_then(|index| state.values.get(*index))
            .copied()
            .ok_or_else(|| ModelError::MissingReference {
                component: self.descriptor.id.clone(),
                kind: "state",
                id: id.into(),
            })
    }
    fn parameter(&self, parameters: &ParameterValues, id: &str) -> Result<f64, ModelError> {
        self.bindings
            .parameter_indices
            .get(id)
            .and_then(|index| parameters.values.get(*index))
            .copied()
            .ok_or_else(|| ModelError::MissingReference {
                component: self.descriptor.id.clone(),
                kind: "parameter",
                id: id.into(),
            })
    }
    fn environment(&self, input: &ModelInput) -> AlignedEnvironment {
        let get = |id: &str| input.values.get(id).map(|value| value.value);
        AlignedEnvironment {
            timestamp_s: input.time_s,
            temperature_k: get("temperature"),
            conductivity_s_per_m: get("conductivity"),
            ionic_strength_mol_l: get("ionic_strength"),
            flow: get("flow"),
            polarization_input_v: get("polarization_input_v"),
            interferent_activities: input
                .values
                .iter()
                .filter_map(|(id, value)| {
                    id.strip_prefix("interferent.")
                        .map(|ion| (ion.into(), value.value))
                })
                .collect(),
            ..Default::default()
        }
    }
    fn log_activity(&self, state: &ModelState) -> Result<f64, ModelError> {
        let latent = self.state(state, "log10_activity")?;
        Ok(
            if self
                .descriptor
                .metadata
                .get("activity_transform")
                .is_some_and(|value| value == "LogPositive")
            {
                latent / std::f64::consts::LN_10
            } else {
                latent
            },
        )
    }
    fn sensitivity(&self, state: &ModelState) -> Result<f64, ModelError> {
        let latent = self.state(state, "sensitivity_scale")?;
        if self
            .descriptor
            .metadata
            .get("sensitivity_transform")
            .is_some_and(|value| value == "LogisticBounded")
        {
            let lower = parse_metadata(&self.descriptor, "sensitivity_lower")?;
            let upper = parse_metadata(&self.descriptor, "sensitivity_upper")?;
            Ok(lower + (upper - lower) / (1.0 + (-latent).exp()))
        } else {
            Ok(latent)
        }
    }
}

impl IsmComponent for LegacyComponent {
    fn descriptor(&self) -> &ComponentDescriptor {
        &self.descriptor
    }
    fn bind(&mut self, bindings: &ComponentBindings) -> Result<(), ModelError> {
        self.bindings = bindings.clone();
        Ok(())
    }
    fn process_transition(
        &self,
        state: &mut ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        dt_s: f64,
    ) -> Result<(), ModelError> {
        if self.descriptor.kind != "estimation.legacy_polarization" {
            return Ok(());
        }
        let index = *self
            .bindings
            .state_indices
            .get("polarization")
            .ok_or_else(|| evaluation(&self.descriptor, "missing polarization state"))?;
        let tau = self.parameter(parameters, "legacy_polarization_tau_s")?;
        let gain = self.parameter(parameters, "legacy_polarization_gain")?;
        let drive = input
            .values
            .get("polarization_input_v")
            .map_or(0.0, |value| value.value);
        state.values[index] = (-dt_s / tau).exp() * state.values[index] + gain * drive;
        Ok(())
    }
    fn process_jacobian(
        &self,
        dimension: usize,
        _state: &ModelState,
        parameters: &ParameterValues,
        _input: &ModelInput,
        dt_s: f64,
    ) -> Result<Jacobian, ModelError> {
        let mut jacobian = (0..dimension)
            .map(|row| {
                (0..dimension)
                    .map(|column| if row == column { 1.0 } else { 0.0 })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        if self.descriptor.kind == "estimation.legacy_polarization" {
            let index = self.bindings.state_indices["polarization"];
            jacobian[index][index] =
                (-dt_s / self.parameter(parameters, "legacy_polarization_tau_s")?).exp();
        }
        Ok(jacobian)
    }
    fn observation_voltage(
        &self,
        state: &ModelState,
        _parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<Option<f64>, ModelError> {
        let value = match self.descriptor.kind.as_str() {
            "estimation.legacy_equilibrium" => self
                .calibration
                .as_ref()
                .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?
                .predict_potential(self.log_activity(state)?, &self.environment(input))
                .map_err(|error| evaluation(&self.descriptor, error))?,
            "estimation.legacy_baseline" => self.state(state, "baseline_offset")?,
            "estimation.legacy_polarization" => self.state(state, "polarization")?,
            "estimation.legacy_sensitivity" => {
                let calibration = self
                    .calibration
                    .as_ref()
                    .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?;
                let environment = self.environment(input);
                let activity = self.log_activity(state)?;
                (self.sensitivity(state)? - 1.0)
                    * (calibration
                        .predict_potential(activity, &environment)
                        .map_err(|error| evaluation(&self.descriptor, error))?
                        - calibration
                            .predict_potential(0.0, &environment)
                            .map_err(|error| evaluation(&self.descriptor, error))?)
            }
            _ => return Err(evaluation(&self.descriptor, "unknown legacy component")),
        };
        Ok(Some(value))
    }
    fn observation_state_jacobian(
        &self,
        state: &ModelState,
        _parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<StateJacobian, ModelError> {
        let dimension = self.bindings.state_indices.len();
        let mut result = vec![0.0; dimension];
        match self.descriptor.kind.as_str() {
            "estimation.legacy_equilibrium" => {
                let index = self.bindings.state_indices["log10_activity"];
                result[index] = self
                    .calibration
                    .as_ref()
                    .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?
                    .jacobian_log10_activity(self.log_activity(state)?, &self.environment(input))
                    .map_err(|error| evaluation(&self.descriptor, error))?;
            }
            "estimation.legacy_baseline" => {
                result[self.bindings.state_indices["baseline_offset"]] = 1.0
            }
            "estimation.legacy_polarization" => {
                result[self.bindings.state_indices["polarization"]] = 1.0
            }
            "estimation.legacy_sensitivity" => {
                let calibration = self
                    .calibration
                    .as_ref()
                    .ok_or_else(|| evaluation(&self.descriptor, "missing calibration"))?;
                let environment = self.environment(input);
                let activity = self.log_activity(state)?;
                let signal = calibration
                    .predict_potential(activity, &environment)
                    .map_err(|error| evaluation(&self.descriptor, error))?
                    - calibration
                        .predict_potential(0.0, &environment)
                        .map_err(|error| evaluation(&self.descriptor, error))?;
                result[self.bindings.state_indices["log10_activity"]] =
                    (self.sensitivity(state)? - 1.0)
                        * calibration
                            .jacobian_log10_activity(activity, &environment)
                            .map_err(|error| evaluation(&self.descriptor, error))?;
                result[self.bindings.state_indices["sensitivity_scale"]] = signal;
            }
            _ => {}
        }
        Ok(StateJacobian::analytic(
            self.descriptor
                .observation_state_ids
                .iter()
                .map(|id| {
                    let index = self.bindings.state_indices[id];
                    (id.clone(), result[index])
                })
                .collect::<Vec<_>>(),
        ))
    }
}

fn descriptor(
    id: &str,
    kind: &str,
    role: ComponentRole,
    states: Vec<&str>,
    parameters: Vec<&str>,
    owner: &str,
    metadata: BTreeMap<String, String>,
) -> ComponentDescriptor {
    let state_ids = states.into_iter().map(str::to_string).collect::<Vec<_>>();
    ComponentDescriptor { id: id.into(), kind: kind.into(), role, interpretation_status: InterpretationStatus::Phenomenological, depends_on: Vec::new(), required_inputs: Vec::new(), observation_state_ids: state_ids.clone(), observation_parameter_ids: Vec::new(), numerical_jacobian_supported: false, state_ids, parameter_ids: parameters.into_iter().map(str::to_string).collect(), output_unit: Some("V".into()), voltage_contribution_owner: Some(owner.into()), contribution_semantics: crate::model::ContributionSemantics::AdditivePotential, legacy_composition_rule: None, source: "legacy estimation compatibility adapter".into(), validity_domain: "stored calibration and configured legacy estimator domain".into(), equation: "legacy Phase 6 estimator equation adapter".into(), equation_version: 1, assumptions: vec!["Compatibility adapter preserves legacy numerical semantics without assigning a physical mechanism.".into()], evidence_requirements: vec![EvidenceRequirement { hypothesis_id: format!("{id}.identity"), proposed_mechanism_label: "unassigned".into(), independent_evidence_types: vec!["independent experiment".into()], minimum_independent_observations: 2, validity_domain: "declared calibration domain".into(), alternatives_to_consider: vec!["other reduced-order explanations".into()], required_uncertainty_statement: "state and calibration uncertainty must be retained".into() }], applicability_constraints: Vec::new(), metadata }
}
fn parameter(
    id: &str,
    unit: &str,
    lower: f64,
    upper: f64,
    default_value: f64,
    uncertainty: Option<UncertaintySpec>,
) -> ParameterSpec {
    ParameterSpec {
        id: id.into(),
        name: id.replace('_', " "),
        description: "Legacy estimator compatibility parameter.".into(),
        unit: unit.into(),
        lower_bound: lower,
        upper_bound: upper,
        default_value,
        uncertainty: uncertainty.unwrap_or_else(|| UncertaintySpec::Unknown {
            reason: "legacy estimator compatibility parameter has no configured covariance".into(),
        }),
        source: "legacy estimation configuration".into(),
        equation_version: 1,
        identifiability_requirements: vec![
            "Retain legacy estimator observability and covariance evidence.".into(),
        ],
        value_source: ParameterValueSource::ExternallySupplied,
        characteristic: crate::model::ParameterCharacteristic::Continuous,
        validity_domain: "configured legacy estimator domain".into(),
    }
}
fn input(id: &str, unit: &str) -> InputSpec {
    InputSpec {
        id: id.into(),
        unit: unit.into(),
        required: false,
        source: "aligned estimation environment".into(),
        validity_domain: "finite aligned input when available".into(),
    }
}
fn evaluation(descriptor: &ComponentDescriptor, error: impl std::fmt::Display) -> ModelError {
    ModelError::ComponentEvaluation {
        component: descriptor.id.clone(),
        message: error.to_string(),
    }
}
fn parse_metadata(descriptor: &ComponentDescriptor, id: &str) -> Result<f64, ModelError> {
    descriptor
        .metadata
        .get(id)
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| evaluation(descriptor, format!("invalid metadata '{id}'")))
}
