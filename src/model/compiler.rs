use super::{
    component::{
        ComponentBindings, ComponentDescriptor, ContributionSemantics, IsmComponent, Jacobian,
        JacobianMethod, JacobianStatus, ParameterJacobian, StateJacobianStatus, identity,
    },
    definition::ModelDefinition,
    error::ModelError,
    graph::dependency_order,
    identifiability::{
        IdentifiabilityMetadata, IdentifiabilityReport, ParameterIdentifiabilityRequirement,
    },
    input::{ModelInput, units_compatible, validate_unit},
    output::{
        ComponentContribution, DEFAULT_POTENTIAL_RECONSTRUCTION_TOLERANCE_V, ModelWarning,
        ObservationPrediction, PredictionUncertainty, PredictionUncertaintyInput,
        UncertaintyStatus,
    },
    parameter::{CompiledParameterSpec, ParameterValues},
    registry::ComponentRegistry,
    state::{
        CompiledStateSpec, DeclaredUncertaintyClass, InitializationContext, InitializedModelState,
        ModelState,
    },
    validity::{ComponentValidityReport, ValidityReport, ValidityStatus},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Serializable compiler output. Trait-object component implementations are
/// intentionally excluded; this preserves the graph and bindings needed for
/// reproducibility without pretending a compiled binary plugin is portable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledModelSummary {
    pub schema_version: u32,
    pub compiler_version: String,
    pub model_id: String,
    pub component_order: Vec<String>,
    pub component_descriptors: Vec<ComponentDescriptor>,
    pub state_bindings: Vec<CompiledBindingSummary>,
    pub parameter_bindings: Vec<CompiledBindingSummary>,
    pub dependency_graph: BTreeMap<String, Vec<String>>,
    pub equation_versions: BTreeMap<String, u32>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledBindingSummary {
    pub id: String,
    pub global_index: usize,
    pub component_ids: Vec<String>,
}

/// Stable component-local bindings in global vector coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentBindingSummary {
    pub component_id: String,
    pub state_indices: Vec<usize>,
    pub parameter_indices: Vec<usize>,
}

/// Deterministically validated and factory-resolved ISM model graph.
pub struct CompiledIsmModel {
    definition: ModelDefinition,
    state_definitions: Vec<CompiledStateSpec>,
    parameter_definitions: Vec<CompiledParameterSpec>,
    state_indices: BTreeMap<String, usize>,
    parameter_indices: BTreeMap<String, usize>,
    component_bindings: BTreeMap<String, ComponentBindingSummary>,
    components: Vec<Box<dyn IsmComponent>>,
}

impl CompiledIsmModel {
    pub fn definition(&self) -> &ModelDefinition {
        &self.definition
    }

    pub fn state_definitions(&self) -> &[CompiledStateSpec] {
        &self.state_definitions
    }

    pub fn parameter_definitions(&self) -> &[CompiledParameterSpec] {
        &self.parameter_definitions
    }

    pub fn state_index(&self, id: &str) -> Option<usize> {
        self.state_indices.get(id).copied()
    }

    pub fn parameter_index(&self, id: &str) -> Option<usize> {
        self.parameter_indices.get(id).copied()
    }

    pub fn state_spec(&self, id: &str) -> Option<&CompiledStateSpec> {
        self.state_index(id)
            .and_then(|index| self.state_definitions.get(index))
    }

    pub fn parameter_spec(&self, id: &str) -> Option<&CompiledParameterSpec> {
        self.parameter_index(id)
            .and_then(|index| self.parameter_definitions.get(index))
    }

    pub fn state_id(&self, global_index: usize) -> Option<&str> {
        self.state_definitions
            .get(global_index)
            .map(|binding| binding.spec.id.as_str())
    }

    pub fn parameter_id(&self, global_index: usize) -> Option<&str> {
        self.parameter_definitions
            .get(global_index)
            .map(|binding| binding.spec.id.as_str())
    }

    /// Component-local state indices in stable global state order.
    pub fn state_slice(&self, component_id: &str) -> Option<&[usize]> {
        self.component_bindings
            .get(component_id)
            .map(|binding| binding.state_indices.as_slice())
    }

    /// Component-local parameter indices in stable global parameter order.
    pub fn parameter_slice(&self, component_id: &str) -> Option<&[usize]> {
        self.component_bindings
            .get(component_id)
            .map(|binding| binding.parameter_indices.as_slice())
    }

    pub fn component_bindings(&self) -> &BTreeMap<String, ComponentBindingSummary> {
        &self.component_bindings
    }

    pub fn default_parameters(&self) -> ParameterValues {
        ParameterValues::new(
            self.parameter_definitions
                .iter()
                .map(|parameter| parameter.spec.default_value)
                .collect(),
        )
    }

    pub fn initialize(&self, parameters: &ParameterValues) -> Result<ModelState, ModelError> {
        Ok(self
            .initialize_with_context(parameters, &InitializationContext::default())?
            .state)
    }

    /// Initializes the state from caller values first, then definition
    /// defaults, and finally component initialization hooks. Caller context is
    /// deliberately domain-neutral and does not import estimation settings.
    pub fn initialize_with_context(
        &self,
        parameters: &ParameterValues,
        context: &InitializationContext,
    ) -> Result<InitializedModelState, ModelError> {
        self.validate_parameters(parameters)?;
        let mut sources = Vec::with_capacity(self.state_definitions.len());
        let values = self
            .state_definitions
            .iter()
            .map(|binding| {
                if let Some(value) = context.state_values.get(&binding.spec.id) {
                    sources.push(
                        context
                            .source
                            .clone()
                            .unwrap_or(super::state::StateInitializationSource::External),
                    );
                    *value
                } else {
                    sources.push(binding.spec.initialization_source.clone());
                    binding.spec.initial_value
                }
            })
            .collect();
        for id in context.state_values.keys() {
            if !self.state_indices.contains_key(id) {
                return Err(ModelError::MissingReference {
                    component: "initialization context".into(),
                    kind: "state",
                    id: id.clone(),
                });
            }
        }
        let mut state = ModelState::new(values);
        for component in &self.components {
            component.initialize(&mut state, parameters)?;
        }
        self.validate_state(&state)?;
        Ok(InitializedModelState { state, sources })
    }

    /// Evaluates the sum of only the continuous component derivatives. It is
    /// not an integrator and never implies Euler propagation for discrete
    /// components.
    pub fn process_derivative(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<Vec<f64>, ModelError> {
        self.validate_state(state)?;
        self.validate_parameters(parameters)?;
        self.validate_input(input)?;
        let mut derivative = vec![0.0; self.state_definitions.len()];
        for component in &self.components {
            if let Some(local) = component.process_derivative(
                state,
                parameters,
                input,
                self.state_definitions.len(),
            )? {
                if local.len() != derivative.len() || local.iter().any(|value| !value.is_finite()) {
                    return Err(ModelError::JacobianDimension {
                        component: component.descriptor().id.clone(),
                    });
                }
                for (total, value) in derivative.iter_mut().zip(local) {
                    *total += value;
                }
            }
        }
        Ok(derivative)
    }

    pub fn process_transition(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        dt_s: f64,
    ) -> Result<ModelState, ModelError> {
        if !dt_s.is_finite() || dt_s < 0.0 {
            return Err(ModelError::InvalidTimeStep { dt_s });
        }
        self.validate_state(state)?;
        self.validate_parameters(parameters)?;
        self.validate_input(input)?;
        let mut next = state.clone();
        for component in &self.components {
            component.process_transition(&mut next, parameters, input, dt_s)?;
        }
        self.validate_state(&next)?;
        Ok(next)
    }

    pub fn process_jacobian(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        dt_s: f64,
    ) -> Result<Jacobian, ModelError> {
        if !dt_s.is_finite() || dt_s < 0.0 {
            return Err(ModelError::InvalidTimeStep { dt_s });
        }
        self.validate_state(state)?;
        self.validate_parameters(parameters)?;
        self.validate_input(input)?;
        let dimension = self.state_definitions.len();
        let mut result = identity(dimension);
        for component in &self.components {
            let jacobian = component.process_jacobian(dimension, state, parameters, input, dt_s)?;
            if !valid_matrix(&jacobian, dimension, dimension) {
                return Err(ModelError::JacobianDimension {
                    component: component.descriptor().id.clone(),
                });
            }
            result = multiply(&jacobian, &result);
        }
        Ok(result)
    }

    pub fn component_contributions(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<Vec<ComponentContribution>, ModelError> {
        self.validate_state(state)?;
        self.validate_parameters(parameters)?;
        self.validate_input(input)?;
        let mut contributions = Vec::new();
        for component in &self.components {
            let descriptor = component.descriptor();
            let voltage = component.observation_voltage(state, parameters, input)?;
            let variance = component.observation_variance_v2(state, parameters, input)?;
            let component_warnings = component
                .validity_warnings(state, parameters, input)
                .map_err(|error| ModelError::ComponentEvaluation {
                    component: descriptor.id.clone(),
                    message: error.to_string(),
                })?;
            let auxiliary_outputs = component.auxiliary_outputs(state, parameters, input)?;
            if auxiliary_outputs.values().any(|value| !value.is_finite()) {
                return Err(ModelError::ComponentEvaluation {
                    component: descriptor.id.clone(),
                    message: "component emitted a non-finite auxiliary output".into(),
                });
            }
            let (potential_v, variance_v2) = match descriptor.contribution_semantics {
                ContributionSemantics::AdditivePotential => {
                    let value =
                        voltage.ok_or_else(|| ModelError::IncompatibleContributionOutput {
                            component: descriptor.id.clone(),
                            semantics: descriptor.contribution_semantics,
                        })?;
                    if variance.is_some() || !value.is_finite() {
                        return Err(ModelError::IncompatibleContributionOutput {
                            component: descriptor.id.clone(),
                            semantics: descriptor.contribution_semantics,
                        });
                    }
                    (Some(value), None)
                }
                ContributionSemantics::ObservationVariance => {
                    let value =
                        variance.ok_or_else(|| ModelError::IncompatibleContributionOutput {
                            component: descriptor.id.clone(),
                            semantics: descriptor.contribution_semantics,
                        })?;
                    if voltage.is_some() || !value.is_finite() || value < 0.0 {
                        return Err(ModelError::IncompatibleContributionOutput {
                            component: descriptor.id.clone(),
                            semantics: descriptor.contribution_semantics,
                        });
                    }
                    (None, Some(value))
                }
                ContributionSemantics::StateOnly | ContributionSemantics::Auxiliary => {
                    if voltage.is_some() || variance.is_some() {
                        return Err(ModelError::IncompatibleContributionOutput {
                            component: descriptor.id.clone(),
                            semantics: descriptor.contribution_semantics,
                        });
                    }
                    (None, None)
                }
            };
            contributions.push(ComponentContribution {
                component_id: descriptor.id.clone(),
                owner: descriptor.voltage_contribution_owner.clone(),
                role: descriptor.role,
                semantics: descriptor.contribution_semantics,
                potential_v,
                variance_v2,
                source: descriptor.source.clone(),
                validity_domain: descriptor.validity_domain.clone(),
                interpretation_status: descriptor.interpretation_status,
                equation_version: descriptor.equation_version,
                validity_status: if component_warnings.is_empty() {
                    ValidityStatus::Valid
                } else {
                    ValidityStatus::ValidWithWarnings
                },
                warnings: component_warnings
                    .into_iter()
                    .map(ModelWarning::Validity)
                    .collect(),
                uncertainty_status: UncertaintyStatus::NotRequested,
                state_output_ids: descriptor.state_ids.clone(),
                auxiliary_outputs,
            });
        }
        Ok(contributions)
    }

    pub fn observation_prediction(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        observed_voltage_v: Option<f64>,
    ) -> Result<ObservationPrediction, ModelError> {
        self.observation_prediction_with_uncertainty(
            state,
            parameters,
            input,
            observed_voltage_v,
            PredictionUncertaintyInput::default(),
        )
    }

    pub fn observation_prediction_with_uncertainty(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        observed_voltage_v: Option<f64>,
        uncertainty_input: PredictionUncertaintyInput,
    ) -> Result<ObservationPrediction, ModelError> {
        let mut contributions = self.component_contributions(state, parameters, input)?;
        let uncertainty = if uncertainty_input.requested {
            self.prediction_uncertainty(
                state,
                parameters,
                input,
                &contributions,
                uncertainty_input,
            )?
        } else {
            PredictionUncertainty::not_requested()
        };
        for contribution in &mut contributions {
            contribution.uncertainty_status = uncertainty.status;
        }
        let prediction =
            ObservationPrediction::new(contributions, observed_voltage_v, uncertainty)?;
        prediction.verify_reconstruction(DEFAULT_POTENTIAL_RECONSTRUCTION_TOLERANCE_V)?;
        Ok(prediction)
    }

    pub fn observation_jacobian(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<Vec<f64>, ModelError> {
        self.validate_state(state)?;
        self.validate_parameters(parameters)?;
        self.validate_input(input)?;
        let result = self.aggregate_state_jacobian(state, parameters, input)?;
        if let Some((component, _, message)) = result.missing.first() {
            return Err(ModelError::JacobianCoverage {
                component: component.clone(),
                message: message.clone(),
            });
        }
        Ok(result.values)
    }

    pub fn observation_parameter_jacobian(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<ParameterJacobian, ModelError> {
        self.validate_state(state)?;
        self.validate_parameters(parameters)?;
        self.validate_input(input)?;
        let result = self.aggregate_parameter_jacobian(state, parameters, input)?;
        let covered_parameters = result
            .relevant_ids
            .iter()
            .filter(|id| !result.missing_ids.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let values = covered_parameters
            .iter()
            .map(|id| result.values[self.parameter_indices[id]])
            .collect();
        let status = if result.relevant_ids.is_empty() {
            JacobianStatus::NotApplicable
        } else if result.missing_ids.is_empty() {
            JacobianStatus::Complete
        } else {
            JacobianStatus::Partial {
                missing_parameters: result.missing_ids.iter().cloned().collect(),
            }
        };
        Ok(ParameterJacobian {
            values,
            covered_parameters,
            status,
            method: combined_method(&result.methods),
        })
    }

    fn aggregate_state_jacobian(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<AggregatedJacobian, ModelError> {
        let mut aggregate = AggregatedJacobian::new(self.state_definitions.len());
        for component in &self.components {
            let descriptor = component.descriptor();
            if descriptor.contribution_semantics != ContributionSemantics::AdditivePotential {
                continue;
            }
            let expected = descriptor
                .observation_state_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            aggregate.relevant_ids.extend(expected.iter().cloned());
            let local = component.observation_state_jacobian(state, parameters, input)?;
            validate_method(descriptor, &local.method)?;
            validate_local_values(
                descriptor,
                &local.covered_states,
                &local.values,
                &expected,
                "state",
            )?;
            let covered = local
                .covered_states
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let missing = expected
                .difference(&covered)
                .cloned()
                .collect::<BTreeSet<_>>();
            match &local.status {
                StateJacobianStatus::Complete if missing.is_empty() => {}
                StateJacobianStatus::Partial { missing_states }
                    if missing_states.iter().cloned().collect::<BTreeSet<_>>() == missing => {}
                StateJacobianStatus::Unavailable { reason } if !reason.trim().is_empty() => {}
                StateJacobianStatus::NotApplicable if expected.is_empty() && covered.is_empty() => {
                }
                _ => {
                    return Err(ModelError::JacobianCoverage {
                        component: descriptor.id.clone(),
                        message: "state Jacobian status is inconsistent with declared coverage"
                            .into(),
                    });
                }
            }
            for (id, value) in local.covered_states.iter().zip(local.values) {
                let index =
                    self.state_indices
                        .get(id)
                        .ok_or_else(|| ModelError::JacobianCoverage {
                            component: descriptor.id.clone(),
                            message: format!("covered state '{id}' has no compiled index"),
                        })?;
                aggregate.values[*index] += value;
            }
            let unavailable_reason = match local.status {
                StateJacobianStatus::Unavailable { reason } => Some(reason),
                _ => None,
            };
            for id in missing {
                aggregate.missing_ids.insert(id.clone());
                aggregate.missing.push((
                    descriptor.id.clone(),
                    id.clone(),
                    unavailable_reason.as_ref().map_or_else(
                        || format!("state '{id}' derivative is missing"),
                        |reason| format!("state '{id}' derivative unavailable: {reason}"),
                    ),
                ));
            }
            if !expected.is_empty() {
                aggregate.methods.push(local.method);
            }
        }
        Ok(aggregate)
    }

    fn aggregate_parameter_jacobian(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Result<AggregatedJacobian, ModelError> {
        let mut aggregate = AggregatedJacobian::new(self.parameter_definitions.len());
        for component in &self.components {
            let descriptor = component.descriptor();
            if descriptor.contribution_semantics != ContributionSemantics::AdditivePotential {
                continue;
            }
            let expected = descriptor
                .observation_parameter_ids
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            aggregate.relevant_ids.extend(expected.iter().cloned());
            let local = component.observation_parameter_jacobian(state, parameters, input)?;
            validate_method(descriptor, &local.method)?;
            validate_local_values(
                descriptor,
                &local.covered_parameters,
                &local.values,
                &expected,
                "parameter",
            )?;
            let covered = local
                .covered_parameters
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let missing = expected
                .difference(&covered)
                .cloned()
                .collect::<BTreeSet<_>>();
            match &local.status {
                JacobianStatus::Complete if missing.is_empty() => {}
                JacobianStatus::Partial { missing_parameters }
                    if missing_parameters.iter().cloned().collect::<BTreeSet<_>>() == missing => {}
                JacobianStatus::Unavailable { reason } if !reason.trim().is_empty() => {}
                JacobianStatus::NotApplicable if expected.is_empty() && covered.is_empty() => {}
                _ => {
                    return Err(ModelError::JacobianCoverage {
                        component: descriptor.id.clone(),
                        message: "parameter Jacobian status is inconsistent with declared coverage"
                            .into(),
                    });
                }
            }
            for (id, value) in local.covered_parameters.iter().zip(local.values) {
                let index =
                    self.parameter_indices
                        .get(id)
                        .ok_or_else(|| ModelError::JacobianCoverage {
                            component: descriptor.id.clone(),
                            message: format!("covered parameter '{id}' has no compiled index"),
                        })?;
                aggregate.values[*index] += value;
            }
            let unavailable_reason = match local.status {
                JacobianStatus::Unavailable { reason } => Some(reason),
                _ => None,
            };
            for id in missing {
                aggregate.missing_ids.insert(id.clone());
                aggregate.missing.push((
                    descriptor.id.clone(),
                    id.clone(),
                    unavailable_reason.as_ref().map_or_else(
                        || format!("parameter '{id}' derivative is missing"),
                        |reason| format!("parameter '{id}' derivative unavailable: {reason}"),
                    ),
                ));
            }
            if !expected.is_empty() {
                aggregate.methods.push(local.method);
            }
        }
        Ok(aggregate)
    }

    fn prediction_uncertainty(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
        contributions: &[ComponentContribution],
        supplied: PredictionUncertaintyInput,
    ) -> Result<PredictionUncertainty, ModelError> {
        let mut missing_sources = Vec::new();
        let assumptions = Vec::new();
        let state_jacobian = self.aggregate_state_jacobian(state, parameters, input)?;
        let parameter_jacobian = self.aggregate_parameter_jacobian(state, parameters, input)?;

        let resolved_state_covariance = resolve_covariance(
            supplied.state_covariance,
            self.state_definitions.len(),
            &state_jacobian.relevant_ids,
            self.state_definitions.iter().map(|item| {
                (
                    item.spec.id.as_str(),
                    item.spec.declared_uncertainty_class(),
                )
            }),
            "state",
            &mut missing_sources,
        )?;
        let resolved_parameter_covariance = resolve_covariance(
            supplied.parameter_covariance,
            self.parameter_definitions.len(),
            &parameter_jacobian.relevant_ids,
            self.parameter_definitions.iter().map(|item| {
                (
                    item.spec.id.as_str(),
                    item.spec.declared_uncertainty_class(),
                )
            }),
            "parameter",
            &mut missing_sources,
        )?;
        let state_has_information = resolved_state_covariance.has_information;
        let state_covariance_complete = resolved_state_covariance.complete;
        let state_covariance = resolved_state_covariance.matrix;
        let parameter_has_information = resolved_parameter_covariance.has_information;
        let parameter_covariance_complete = resolved_parameter_covariance.complete;
        let parameter_covariance = resolved_parameter_covariance.matrix;

        record_relevant_missing_derivatives(
            &state_jacobian,
            &self.state_indices,
            self.state_definitions.iter().map(|item| {
                (
                    item.spec.id.as_str(),
                    item.spec.declared_uncertainty_class(),
                )
            }),
            "state",
            &mut missing_sources,
        );
        record_relevant_missing_derivatives(
            &parameter_jacobian,
            &self.parameter_indices,
            self.parameter_definitions.iter().map(|item| {
                (
                    item.spec.id.as_str(),
                    item.spec.declared_uncertainty_class(),
                )
            }),
            "parameter",
            &mut missing_sources,
        );

        let state_derivatives_complete = !state_jacobian.missing_ids.iter().any(|id| {
            derivative_is_required(
                id,
                &self.state_indices,
                self.state_definitions.iter().map(|item| {
                    (
                        item.spec.id.as_str(),
                        item.spec.declared_uncertainty_class(),
                    )
                }),
            )
        });
        let parameter_derivatives_complete = !parameter_jacobian.missing_ids.iter().any(|id| {
            derivative_is_required(
                id,
                &self.parameter_indices,
                self.parameter_definitions.iter().map(|item| {
                    (
                        item.spec.id.as_str(),
                        item.spec.declared_uncertainty_class(),
                    )
                }),
            )
        });

        let state_variance_v2 = if state_derivatives_complete && state_covariance_complete {
            state_covariance
                .as_ref()
                .map(|matrix| quadratic_form(&state_jacobian.values, matrix))
                .transpose()?
        } else {
            None
        };
        let parameter_variance_v2 =
            if parameter_derivatives_complete && parameter_covariance_complete {
                parameter_covariance
                    .as_ref()
                    .map(|matrix| quadratic_form(&parameter_jacobian.values, matrix))
                    .transpose()?
            } else {
                None
            };
        let observation_variance_v2 = resolve_observation_variance(
            supplied.observation_variance_v2,
            contributions,
            &mut missing_sources,
        )?;
        let known_total = match (
            state_variance_v2,
            parameter_variance_v2,
            observation_variance_v2,
        ) {
            (Some(state), Some(parameter), Some(observation)) => {
                let total = state + parameter + observation;
                if !total.is_finite() || total < 0.0 {
                    return Err(ModelError::NonFinite {
                        subject: "total prediction variance".into(),
                    });
                }
                Some(total)
            }
            _ => None,
        };
        let status = if missing_sources.is_empty() && known_total.is_some() {
            UncertaintyStatus::Complete
        } else if state_has_information
            || parameter_has_information
            || observation_variance_v2.is_some_and(|value| value > 0.0)
        {
            UncertaintyStatus::Partial
        } else {
            UncertaintyStatus::Unavailable
        };
        Ok(PredictionUncertainty {
            status,
            total_variance_v2: known_total,
            standard_error_v: known_total.map(f64::sqrt),
            state_variance_v2,
            parameter_variance_v2,
            observation_variance_v2,
            missing_sources,
            assumptions,
            state_jacobian_methods: state_jacobian.methods,
            parameter_jacobian_methods: parameter_jacobian.methods,
        })
    }

    pub fn validity_report(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> ValidityReport {
        let mut violations = Vec::new();
        if let Err(error) = self.validate_state(state) {
            violations.push(error.to_string());
        }
        if let Err(error) = self.validate_parameters(parameters) {
            violations.push(error.to_string());
        }
        if let Err(error) = self.validate_input(input) {
            violations.push(error.to_string());
        }
        let mut warnings = Vec::new();
        for component in &self.components {
            match component.validity_warnings(state, parameters, input) {
                Ok(component_warnings) => warnings.extend(component_warnings),
                Err(error) => violations.push(error.to_string()),
            }
        }
        let mut report = if violations.is_empty() {
            ValidityReport::valid(self.definition.validity_domain.clone())
        } else {
            ValidityReport::invalid(self.definition.validity_domain.clone(), violations)
        };
        report.warnings = warnings;
        report
    }

    /// Returns one explicit validity record per component. Evaluation errors
    /// are preserved as rejected evaluations, rather than re-labelled as a
    /// health or mechanism conclusion.
    pub fn component_validity_reports(
        &self,
        state: &ModelState,
        parameters: &ParameterValues,
        input: &ModelInput,
    ) -> Vec<ComponentValidityReport> {
        self.components
            .iter()
            .map(|component| {
                let descriptor = component.descriptor();
                match component.validity_warnings(state, parameters, input) {
                    Ok(warnings) if warnings.is_empty() => ComponentValidityReport {
                        component_id: descriptor.id.clone(),
                        status: ValidityStatus::Valid,
                        assumptions_checked: descriptor.assumptions.clone(),
                        validity_domain: descriptor.validity_domain.clone(),
                        violations: Vec::new(),
                        warnings,
                        evaluation_rejected: false,
                    },
                    Ok(warnings) => ComponentValidityReport {
                        component_id: descriptor.id.clone(),
                        status: ValidityStatus::ValidWithWarnings,
                        assumptions_checked: descriptor.assumptions.clone(),
                        validity_domain: descriptor.validity_domain.clone(),
                        violations: Vec::new(),
                        warnings,
                        evaluation_rejected: false,
                    },
                    Err(error) => ComponentValidityReport {
                        component_id: descriptor.id.clone(),
                        status: ValidityStatus::Unavailable,
                        assumptions_checked: descriptor.assumptions.clone(),
                        validity_domain: descriptor.validity_domain.clone(),
                        violations: vec![error.to_string()],
                        warnings: Vec::new(),
                        evaluation_rejected: true,
                    },
                }
            })
            .collect()
    }

    pub fn identifiability_report(&self) -> IdentifiabilityReport {
        IdentifiabilityReport::not_assessed(
            self.parameter_definitions
                .iter()
                .map(|parameter| parameter.spec.id.clone())
                .collect(),
        )
    }

    pub fn identifiability_metadata(&self) -> IdentifiabilityMetadata {
        IdentifiabilityMetadata {
            states_requiring_independent_observations: self
                .state_definitions
                .iter()
                .filter(|state| !state.spec.observability_requirements.is_empty())
                .map(|state| state.spec.id.clone())
                .collect(),
            parameter_requirements: self
                .parameter_definitions
                .iter()
                .map(|parameter| ParameterIdentifiabilityRequirement {
                    parameter_id: parameter.spec.id.clone(),
                    requirements: parameter.spec.identifiability_requirements.clone(),
                })
                .collect(),
            component_sensitivity_targets: self
                .components
                .iter()
                .filter(|component| {
                    !component.descriptor().observation_state_ids.is_empty()
                        || !component.descriptor().observation_parameter_ids.is_empty()
                })
                .map(|component| component.descriptor().id.clone())
                .collect(),
        }
    }

    pub fn compiled_summary(&self) -> CompiledModelSummary {
        let component_order = self
            .components
            .iter()
            .map(|component| component.descriptor().id.clone())
            .collect::<Vec<_>>();
        let descriptors = self
            .components
            .iter()
            .map(|component| component.descriptor().clone())
            .collect::<Vec<_>>();
        let component_ids_for = |id: &str, is_state: bool| {
            self.components
                .iter()
                .filter(|component| {
                    if is_state {
                        component
                            .descriptor()
                            .state_ids
                            .iter()
                            .any(|candidate| candidate == id)
                    } else {
                        component
                            .descriptor()
                            .parameter_ids
                            .iter()
                            .any(|candidate| candidate == id)
                    }
                })
                .map(|component| component.descriptor().id.clone())
                .collect::<Vec<_>>()
        };
        CompiledModelSummary {
            schema_version: self.definition.schema_version,
            compiler_version: env!("CARGO_PKG_VERSION").into(),
            model_id: self.definition.model_id.clone(),
            component_order,
            component_descriptors: descriptors,
            state_bindings: self
                .state_definitions
                .iter()
                .map(|binding| CompiledBindingSummary {
                    id: binding.spec.id.clone(),
                    global_index: binding.index,
                    component_ids: component_ids_for(&binding.spec.id, true),
                })
                .collect(),
            parameter_bindings: self
                .parameter_definitions
                .iter()
                .map(|binding| CompiledBindingSummary {
                    id: binding.spec.id.clone(),
                    global_index: binding.index,
                    component_ids: component_ids_for(&binding.spec.id, false),
                })
                .collect(),
            dependency_graph: self
                .components
                .iter()
                .map(|component| {
                    (
                        component.descriptor().id.clone(),
                        component.descriptor().depends_on.clone(),
                    )
                })
                .collect(),
            equation_versions: self
                .components
                .iter()
                .map(|component| {
                    (
                        component.descriptor().id.clone(),
                        component.descriptor().equation_version,
                    )
                })
                .collect(),
            warnings: Vec::new(),
        }
    }

    pub fn validate_parameters(&self, parameters: &ParameterValues) -> Result<(), ModelError> {
        if parameters.values.len() != self.parameter_definitions.len() {
            return Err(ModelError::ParameterDimension {
                expected: self.parameter_definitions.len(),
                actual: parameters.values.len(),
            });
        }
        for (parameter, value) in self.parameter_definitions.iter().zip(&parameters.values) {
            if !value.is_finite() {
                return Err(ModelError::NonFinite {
                    subject: format!("parameter '{}'", parameter.spec.id),
                });
            }
            if *value < parameter.spec.lower_bound || *value > parameter.spec.upper_bound {
                return Err(ModelError::BoundViolation {
                    kind: "parameter",
                    id: parameter.spec.id.clone(),
                    value: *value,
                    lower: parameter.spec.lower_bound,
                    upper: parameter.spec.upper_bound,
                });
            }
        }
        Ok(())
    }

    pub fn validate_state(&self, state: &ModelState) -> Result<(), ModelError> {
        if state.values.len() != self.state_definitions.len() {
            return Err(ModelError::StateDimension {
                expected: self.state_definitions.len(),
                actual: state.values.len(),
            });
        }
        for (specification, value) in self.state_definitions.iter().zip(&state.values) {
            if !value.is_finite() {
                return Err(ModelError::NonFinite {
                    subject: format!("state '{}'", specification.spec.id),
                });
            }
            if *value < specification.spec.lower_bound || *value > specification.spec.upper_bound {
                return Err(ModelError::BoundViolation {
                    kind: "state",
                    id: specification.spec.id.clone(),
                    value: *value,
                    lower: specification.spec.lower_bound,
                    upper: specification.spec.upper_bound,
                });
            }
        }
        Ok(())
    }

    fn validate_input(&self, input: &ModelInput) -> Result<(), ModelError> {
        if !input.time_s.is_finite() {
            return Err(ModelError::NonFinite {
                subject: "model input time".into(),
            });
        }
        for specification in &self.definition.inputs {
            match input.values.get(&specification.id) {
                Some(value) => {
                    if !value.value.is_finite() {
                        return Err(ModelError::NonFinite {
                            subject: format!("input '{}'", specification.id),
                        });
                    }
                    if !units_compatible(&specification.unit, &value.unit) {
                        return Err(ModelError::UnitMismatch {
                            component: "runtime input".into(),
                            input: specification.id.clone(),
                            expected: specification.unit.clone(),
                            found: value.unit.clone(),
                        });
                    }
                }
                None if specification.required => {
                    return Err(ModelError::MissingInput {
                        component: "runtime model".into(),
                        input: specification.id.clone(),
                    });
                }
                None => {}
            }
        }
        for component in &self.components {
            for requirement in &component.descriptor().required_inputs {
                if !input.values.contains_key(&requirement.id) {
                    return Err(ModelError::MissingInput {
                        component: component.descriptor().id.clone(),
                        input: requirement.id.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Compile a versioned model definition against an immutable static registry.
pub fn compile_model(
    definition: ModelDefinition,
    registry: &ComponentRegistry,
) -> Result<CompiledIsmModel, ModelError> {
    definition.validate_schema()?;
    if definition.schema_version < super::definition::MODEL_DEFINITION_SCHEMA_VERSION {
        return Err(ModelError::LegacyMigrationRequired {
            found: definition.schema_version,
            expected: super::definition::MODEL_DEFINITION_SCHEMA_VERSION,
        });
    }
    let state_indices = stable_indices(definition.states.iter().map(|state| state.id.as_str()));
    let parameter_indices = stable_indices(
        definition
            .parameters
            .iter()
            .map(|parameter| parameter.id.as_str()),
    );
    validate_component_references(&definition, &state_indices, &parameter_indices)?;
    let ordered_ids = dependency_order(&definition.components)?;
    validate_contribution_owners(&definition.components)?;
    let component_by_id: BTreeMap<&str, &ComponentDescriptor> = definition
        .components
        .iter()
        .map(|component| (component.id.as_str(), component))
        .collect();
    let mut components = Vec::with_capacity(ordered_ids.len());
    let mut component_bindings = BTreeMap::new();
    for id in ordered_ids {
        let descriptor =
            component_by_id
                .get(id.as_str())
                .ok_or_else(|| ModelError::MissingReference {
                    component: id.clone(),
                    kind: "component",
                    id: id.clone(),
                })?;
        let mut component = registry.create(descriptor)?;
        if component.descriptor() != *descriptor {
            return Err(ModelError::FactoryDescriptorMismatch {
                component: descriptor.id.clone(),
            });
        }
        component.bind(&ComponentBindings {
            state_indices: state_indices.clone(),
            parameter_indices: parameter_indices.clone(),
        })?;
        component_bindings.insert(
            descriptor.id.clone(),
            ComponentBindingSummary {
                component_id: descriptor.id.clone(),
                state_indices: descriptor
                    .state_ids
                    .iter()
                    .filter_map(|state_id| state_indices.get(state_id).copied())
                    .collect(),
                parameter_indices: descriptor
                    .parameter_ids
                    .iter()
                    .filter_map(|parameter_id| parameter_indices.get(parameter_id).copied())
                    .collect(),
            },
        );
        components.push(component);
    }
    Ok(CompiledIsmModel {
        state_definitions: definition
            .states
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, spec)| CompiledStateSpec { index, spec })
            .collect(),
        parameter_definitions: definition
            .parameters
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, spec)| CompiledParameterSpec { index, spec })
            .collect(),
        definition,
        state_indices,
        parameter_indices,
        component_bindings,
        components,
    })
}

fn stable_indices<'a>(ids: impl Iterator<Item = &'a str>) -> BTreeMap<String, usize> {
    ids.enumerate()
        .map(|(index, id)| (id.to_string(), index))
        .collect()
}

fn validate_component_references(
    definition: &ModelDefinition,
    states: &BTreeMap<String, usize>,
    parameters: &BTreeMap<String, usize>,
) -> Result<(), ModelError> {
    let inputs: BTreeMap<&str, &str> = definition
        .inputs
        .iter()
        .map(|input| (input.id.as_str(), input.unit.as_str()))
        .collect();
    for component in &definition.components {
        for id in &component.state_ids {
            if !states.contains_key(id) {
                return Err(ModelError::MissingReference {
                    component: component.id.clone(),
                    kind: "state",
                    id: id.clone(),
                });
            }
        }
        for id in &component.parameter_ids {
            if !parameters.contains_key(id) {
                return Err(ModelError::MissingReference {
                    component: component.id.clone(),
                    kind: "parameter",
                    id: id.clone(),
                });
            }
        }
        for requirement in &component.required_inputs {
            validate_unit(
                &requirement.unit,
                format!("component '{}' input '{}'", component.id, requirement.id),
            )?;
            let Some(found) = inputs.get(requirement.id.as_str()) else {
                return Err(ModelError::MissingInput {
                    component: component.id.clone(),
                    input: requirement.id.clone(),
                });
            };
            if !units_compatible(&requirement.unit, found) {
                return Err(ModelError::UnitMismatch {
                    component: component.id.clone(),
                    input: requirement.id.clone(),
                    expected: requirement.unit.clone(),
                    found: (*found).to_string(),
                });
            }
        }
        if let Some(unit) = &component.output_unit {
            validate_unit(unit, format!("component '{}' output", component.id))?;
        }
        if component.contribution_semantics == ContributionSemantics::AdditivePotential
            && !component
                .output_unit
                .as_deref()
                .is_some_and(|unit| units_compatible("V", unit))
        {
            return Err(ModelError::UnitMismatch {
                component: component.id.clone(),
                input: "voltage contribution output".into(),
                expected: "V".into(),
                found: component.output_unit.clone().unwrap_or_default(),
            });
        }
    }
    Ok(())
}

fn validate_contribution_owners(components: &[ComponentDescriptor]) -> Result<(), ModelError> {
    let mut owners = BTreeSet::new();
    for component in components {
        if let Some(owner) = &component.voltage_contribution_owner {
            if owner.trim().is_empty() {
                return Err(ModelError::EmptyIdentifier {
                    kind: "voltage contribution owner",
                });
            }
            if !owners.insert(owner) {
                return Err(ModelError::DuplicateContributionOwner {
                    owner: owner.clone(),
                });
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
struct AggregatedJacobian {
    values: Vec<f64>,
    relevant_ids: BTreeSet<String>,
    missing_ids: BTreeSet<String>,
    missing: Vec<(String, String, String)>,
    methods: Vec<JacobianMethod>,
}

struct ResolvedCovariance {
    matrix: Option<Vec<Vec<f64>>>,
    has_information: bool,
    complete: bool,
}

impl AggregatedJacobian {
    fn new(dimension: usize) -> Self {
        Self {
            values: vec![0.0; dimension],
            relevant_ids: BTreeSet::new(),
            missing_ids: BTreeSet::new(),
            missing: Vec::new(),
            methods: Vec::new(),
        }
    }
}

fn validate_local_values(
    descriptor: &ComponentDescriptor,
    covered_ids: &[String],
    values: &[f64],
    expected: &BTreeSet<String>,
    kind: &str,
) -> Result<(), ModelError> {
    let covered = covered_ids.iter().cloned().collect::<BTreeSet<_>>();
    if covered_ids.len() != values.len()
        || covered.len() != covered_ids.len()
        || !covered.is_subset(expected)
        || values.iter().any(|value| !value.is_finite())
    {
        return Err(ModelError::JacobianCoverage {
            component: descriptor.id.clone(),
            message: format!("{kind} Jacobian values and stable-ID coverage are inconsistent"),
        });
    }
    Ok(())
}

fn validate_method(
    descriptor: &ComponentDescriptor,
    method: &JacobianMethod,
) -> Result<(), ModelError> {
    match method {
        JacobianMethod::Numerical {
            relative_step,
            absolute_step,
        } => {
            if !descriptor.numerical_jacobian_supported
                || !relative_step.is_finite()
                || *relative_step <= 0.0
                || !absolute_step.is_finite()
                || *absolute_step <= 0.0
            {
                return Err(ModelError::JacobianCoverage {
                    component: descriptor.id.clone(),
                    message: "numerical Jacobian use was not declared or has an invalid step rule"
                        .into(),
                });
            }
        }
        JacobianMethod::Mixed if !descriptor.numerical_jacobian_supported => {
            return Err(ModelError::JacobianCoverage {
                component: descriptor.id.clone(),
                message: "mixed Jacobian use requires declared numerical support".into(),
            });
        }
        JacobianMethod::Analytic | JacobianMethod::Mixed | JacobianMethod::NotEvaluated => {}
    }
    Ok(())
}

fn combined_method(methods: &[JacobianMethod]) -> JacobianMethod {
    if methods.is_empty() {
        return JacobianMethod::NotEvaluated;
    }
    if methods
        .iter()
        .all(|method| matches!(method, JacobianMethod::Analytic))
    {
        JacobianMethod::Analytic
    } else if methods.len() == 1 {
        methods[0].clone()
    } else {
        JacobianMethod::Mixed
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_covariance<'a>(
    supplied: Option<Vec<Vec<f64>>>,
    dimension: usize,
    relevant_ids: &BTreeSet<String>,
    specifications: impl Iterator<Item = (&'a str, DeclaredUncertaintyClass)>,
    subject: &'static str,
    missing: &mut Vec<String>,
) -> Result<ResolvedCovariance, ModelError> {
    if let Some(matrix) = supplied {
        validate_covariance(&matrix, dimension, subject)?;
        let missing_before = missing.len();
        validate_covariance_contract(&matrix, specifications, relevant_ids, subject, missing)?;
        let complete = missing.len() == missing_before;
        let has_nonzero_entry = matrix.iter().flatten().any(|value| *value != 0.0);
        return Ok(ResolvedCovariance {
            matrix: Some(matrix),
            // A covariance that cannot quantify a relevant schema declaration
            // (for example, `StochasticUnknown` without explicit enrichment)
            // is not usable uncertainty information for status calculation.
            has_information: complete && has_nonzero_entry,
            complete,
        });
    }

    let missing_runtime_covariance = specifications
        .filter(|(id, class)| {
            relevant_ids.contains(*id) && !matches!(class, DeclaredUncertaintyClass::Deterministic)
        })
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    if missing_runtime_covariance.is_empty() {
        // A block with only deterministic relevant quantities needs no runtime
        // covariance. Its exact zero covariance is a semantic consequence of
        // the schema, not a schema-derived approximation of posterior data.
        Ok(ResolvedCovariance {
            matrix: Some(vec![vec![0.0; dimension]; dimension]),
            has_information: false,
            complete: true,
        })
    } else {
        missing.extend(
            missing_runtime_covariance
                .into_iter()
                .map(|id| format!("{subject}:{id} runtime covariance missing")),
        );
        Ok(ResolvedCovariance {
            matrix: None,
            has_information: false,
            complete: false,
        })
    }
}

fn validate_covariance(
    covariance: &[Vec<f64>],
    expected: usize,
    subject: &'static str,
) -> Result<(), ModelError> {
    if covariance.len() != expected || covariance.iter().any(|row| row.len() != expected) {
        let actual = format!(
            "{} rows with widths {:?}",
            covariance.len(),
            covariance.iter().map(Vec::len).collect::<Vec<_>>()
        );
        return Err(ModelError::CovarianceDimension {
            subject,
            expected,
            actual,
        });
    }
    for (row, values) in covariance.iter().enumerate() {
        for (column, value) in values.iter().enumerate() {
            if !value.is_finite() {
                return Err(ModelError::NonFiniteCovariance {
                    subject,
                    row,
                    column,
                });
            }
            if (value - covariance[column][row]).abs() > COVARIANCE_SYMMETRY_TOLERANCE {
                return Err(ModelError::AsymmetricCovariance {
                    subject,
                    row,
                    column,
                });
            }
        }
    }
    validate_positive_semidefinite(covariance, subject)
}

/// Numerical tolerance for matrix symmetry validation only.
const COVARIANCE_SYMMETRY_TOLERANCE: f64 = 1.0e-12;
/// Numerical tolerance for positive-semidefinite matrix validation only.
const COVARIANCE_PSD_TOLERANCE: f64 = 1.0e-12;

/// A caller-supplied covariance quantifies the schema contract; it cannot
/// override it. Unknown declarations remain incomplete until an explicit
/// schema migration/enrichment changes the declaration itself.
fn validate_covariance_contract<'a>(
    covariance: &[Vec<f64>],
    specifications: impl Iterator<Item = (&'a str, DeclaredUncertaintyClass)>,
    relevant_ids: &BTreeSet<String>,
    subject: &'static str,
    missing: &mut Vec<String>,
) -> Result<(), ModelError> {
    for (index, (id, class)) in specifications.enumerate() {
        let diagonal = covariance[index][index];
        match class {
            DeclaredUncertaintyClass::Deterministic => {
                for (column, covariance_entry) in covariance[index].iter().enumerate() {
                    if *covariance_entry != 0.0 {
                        return Err(ModelError::NonzeroCovarianceForDeterministicQuantity {
                            quantity_id: id.into(),
                            covariance_entry: *covariance_entry,
                            row: index,
                            column,
                        });
                    }
                }
                for (row, values) in covariance.iter().enumerate() {
                    let covariance_entry = values[index];
                    if covariance_entry != 0.0 {
                        return Err(ModelError::NonzeroCovarianceForDeterministicQuantity {
                            quantity_id: id.into(),
                            covariance_entry,
                            row,
                            column: index,
                        });
                    }
                }
            }
            DeclaredUncertaintyClass::StochasticKnown if diagonal == 0.0 => {
                return Err(ModelError::ZeroCovarianceForStochasticQuantity {
                    quantity_id: id.into(),
                });
            }
            DeclaredUncertaintyClass::StochasticKnown if diagonal < 0.0 => {
                return Err(ModelError::CovarianceUncertaintyConflict {
                    quantity_id: id.into(),
                    declared_uncertainty: class,
                    covariance_diagonal: Some(diagonal),
                    reason:
                        "a declared stochastic quantity requires a strictly positive covariance diagonal"
                            .into(),
                });
            }
            DeclaredUncertaintyClass::StochasticUnknown if relevant_ids.contains(id) => {
                missing.push(format!(
                    "{subject}:{id} covariance remains unknown; explicit schema enrichment is required"
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// PSD validation permits a singular covariance but rejects negative modes.
fn validate_positive_semidefinite(
    covariance: &[Vec<f64>],
    subject: &'static str,
) -> Result<(), ModelError> {
    let dimension = covariance.len();
    let mut lower = vec![vec![0.0; dimension]; dimension];
    for row in 0..dimension {
        for column in 0..=row {
            let residual = covariance[row][column]
                - (0..column)
                    .map(|index| lower[row][index] * lower[column][index])
                    .sum::<f64>();
            if row == column {
                if residual < -COVARIANCE_PSD_TOLERANCE {
                    return Err(ModelError::NonPositiveSemidefiniteCovariance { subject });
                }
                lower[row][column] = residual.max(0.0).sqrt();
            } else if lower[column][column] > COVARIANCE_PSD_TOLERANCE {
                lower[row][column] = residual / lower[column][column];
            } else if residual.abs() > COVARIANCE_PSD_TOLERANCE {
                return Err(ModelError::NonPositiveSemidefiniteCovariance { subject });
            }
        }
    }
    Ok(())
}

fn derivative_is_required<'a>(
    id: &str,
    indices: &BTreeMap<String, usize>,
    specifications: impl Iterator<Item = (&'a str, DeclaredUncertaintyClass)>,
) -> bool {
    if !indices.contains_key(id) {
        return true;
    }
    specifications
        .filter(|(candidate, _)| *candidate == id)
        .any(|(_, class)| !matches!(class, DeclaredUncertaintyClass::Deterministic))
}

#[allow(clippy::too_many_arguments)]
fn record_relevant_missing_derivatives<'a>(
    jacobian: &AggregatedJacobian,
    indices: &BTreeMap<String, usize>,
    specifications: impl Iterator<Item = (&'a str, DeclaredUncertaintyClass)> + Clone,
    subject: &str,
    missing_sources: &mut Vec<String>,
) {
    for (component, id, message) in &jacobian.missing {
        if derivative_is_required(id, indices, specifications.clone()) {
            missing_sources.push(format!("{subject}:{id} {message} (component:{component})"));
        }
    }
}

fn resolve_observation_variance(
    supplied: Option<f64>,
    contributions: &[ComponentContribution],
    missing_sources: &mut Vec<String>,
) -> Result<Option<f64>, ModelError> {
    if let Some(value) = supplied {
        if value.is_finite() && value >= 0.0 {
            return Ok(Some(value));
        }
        return Err(ModelError::InvalidUncertainty {
            subject: "observation variance".into(),
        });
    }
    let values = contributions
        .iter()
        .filter_map(|item| item.variance_v2)
        .collect::<Vec<_>>();
    if values.is_empty() {
        missing_sources.push("observation variance missing".into());
        return Ok(None);
    }
    let total = values.into_iter().try_fold(0.0, |sum, value| {
        if value.is_finite() && value >= 0.0 {
            Ok(sum + value)
        } else {
            Err(ModelError::NonFinite {
                subject: "observation variance".into(),
            })
        }
    })?;
    Ok(Some(total))
}

fn quadratic_form(jacobian: &[f64], covariance: &[Vec<f64>]) -> Result<f64, ModelError> {
    if covariance.len() != jacobian.len()
        || covariance
            .iter()
            .any(|row| row.len() != jacobian.len() || row.iter().any(|value| !value.is_finite()))
    {
        return Err(ModelError::JacobianDimension {
            component: "prediction uncertainty covariance".into(),
        });
    }
    let result = jacobian
        .iter()
        .enumerate()
        .map(|(row, left)| {
            covariance[row]
                .iter()
                .enumerate()
                .map(|(column, covariance)| left * covariance * jacobian[column])
                .sum::<f64>()
        })
        .sum::<f64>();
    if result.is_finite() && result >= 0.0 {
        Ok(result)
    } else {
        Err(ModelError::NonFinite {
            subject: "propagated prediction variance".into(),
        })
    }
}

fn valid_matrix(matrix: &Jacobian, rows: usize, columns: usize) -> bool {
    matrix.len() == rows
        && matrix
            .iter()
            .all(|row| row.len() == columns && row.iter().all(|value| value.is_finite()))
}

fn multiply(left: &Jacobian, right: &Jacobian) -> Jacobian {
    let size = left.len();
    let mut result = vec![vec![0.0; size]; size];
    for row in 0..size {
        for column in 0..size {
            result[row][column] = (0..size)
                .map(|index| left[row][index] * right[index][column])
                .sum();
        }
    }
    result
}
