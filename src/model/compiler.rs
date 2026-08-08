use super::{
    component::{
        ComponentBindings, ComponentDescriptor, ContributionSemantics, IsmComponent, Jacobian,
        JacobianMethod, JacobianStatus, ParameterJacobian, StateJacobianStatus, identity,
    },
    definition::ModelDefinition,
    error::ModelError,
    graph::dependency_order,
    identifiability::IdentifiabilityReport,
    input::{ModelInput, units_compatible, validate_unit},
    output::{
        ComponentContribution, DEFAULT_POTENTIAL_RECONSTRUCTION_TOLERANCE_V, ObservationPrediction,
        PredictionUncertainty, PredictionUncertaintyInput, UncertaintyStatus,
    },
    parameter::{CompiledParameterSpec, ParameterValues},
    registry::ComponentRegistry,
    state::{CompiledStateSpec, ModelState, UncertaintySpec},
    validity::ValidityReport,
};
use std::collections::{BTreeMap, BTreeSet};

/// Deterministically validated and factory-resolved ISM model graph.
pub struct CompiledIsmModel {
    definition: ModelDefinition,
    state_definitions: Vec<CompiledStateSpec>,
    parameter_definitions: Vec<CompiledParameterSpec>,
    state_indices: BTreeMap<String, usize>,
    parameter_indices: BTreeMap<String, usize>,
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

    pub fn default_parameters(&self) -> ParameterValues {
        ParameterValues::new(
            self.parameter_definitions
                .iter()
                .map(|parameter| parameter.spec.default_value)
                .collect(),
        )
    }

    pub fn initialize(&self, parameters: &ParameterValues) -> Result<ModelState, ModelError> {
        self.validate_parameters(parameters)?;
        let mut state = ModelState::new(
            self.state_definitions
                .iter()
                .map(|state| state.spec.initial_value)
                .collect(),
        );
        for component in &self.components {
            component.initialize(&mut state, parameters)?;
        }
        self.validate_state(&state)?;
        Ok(state)
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
        let contributions = self.component_contributions(state, parameters, input)?;
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
        let mut assumptions = Vec::new();
        let state_jacobian = self.aggregate_state_jacobian(state, parameters, input)?;
        let parameter_jacobian = self.aggregate_parameter_jacobian(state, parameters, input)?;

        let resolved_state_covariance = resolve_covariance(
            supplied.state_covariance,
            self.state_definitions.len(),
            &state_jacobian.relevant_ids,
            self.state_definitions.iter().map(|item| {
                (
                    item.spec.id.as_str(),
                    &item.spec.initial_uncertainty,
                    item.spec.unit.as_str(),
                )
            }),
            "state",
            &mut missing_sources,
            &mut assumptions,
        )?;
        let resolved_parameter_covariance = resolve_covariance(
            supplied.parameter_covariance,
            self.parameter_definitions.len(),
            &parameter_jacobian.relevant_ids,
            self.parameter_definitions.iter().map(|item| {
                (
                    item.spec.id.as_str(),
                    &item.spec.uncertainty,
                    item.spec.unit.as_str(),
                )
            }),
            "parameter",
            &mut missing_sources,
            &mut assumptions,
        )?;
        let state_covariance = resolved_state_covariance.matrix;
        let state_has_information = resolved_state_covariance.has_information;
        let parameter_covariance = resolved_parameter_covariance.matrix;
        let parameter_has_information = resolved_parameter_covariance.has_information;

        record_relevant_missing_derivatives(
            &state_jacobian,
            state_covariance.as_deref(),
            &self.state_indices,
            self.state_definitions
                .iter()
                .map(|item| (item.spec.id.as_str(), &item.spec.initial_uncertainty)),
            "state",
            &mut missing_sources,
        );
        record_relevant_missing_derivatives(
            &parameter_jacobian,
            parameter_covariance.as_deref(),
            &self.parameter_indices,
            self.parameter_definitions
                .iter()
                .map(|item| (item.spec.id.as_str(), &item.spec.uncertainty)),
            "parameter",
            &mut missing_sources,
        );

        let state_derivatives_complete = !state_jacobian.missing_ids.iter().any(|id| {
            derivative_is_required(
                id,
                state_covariance.as_deref(),
                &self.state_indices,
                self.state_definitions
                    .iter()
                    .map(|item| (item.spec.id.as_str(), &item.spec.initial_uncertainty)),
            )
        });
        let parameter_derivatives_complete = !parameter_jacobian.missing_ids.iter().any(|id| {
            derivative_is_required(
                id,
                parameter_covariance.as_deref(),
                &self.parameter_indices,
                self.parameter_definitions
                    .iter()
                    .map(|item| (item.spec.id.as_str(), &item.spec.uncertainty)),
            )
        });

        let state_variance_v2 = if state_derivatives_complete {
            state_covariance
                .as_ref()
                .map(|matrix| quadratic_form(&state_jacobian.values, matrix))
                .transpose()?
        } else {
            None
        };
        let parameter_variance_v2 = if parameter_derivatives_complete {
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

    pub fn identifiability_report(&self) -> IdentifiabilityReport {
        IdentifiabilityReport::not_assessed(
            self.parameter_definitions
                .iter()
                .map(|parameter| parameter.spec.id.clone())
                .collect(),
        )
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

    fn validate_state(&self, state: &ModelState) -> Result<(), ModelError> {
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
        if component.descriptor().id != descriptor.id
            || component.descriptor().kind != descriptor.kind
        {
            return Err(ModelError::FactoryDescriptorMismatch {
                component: descriptor.id.clone(),
            });
        }
        component.bind(&ComponentBindings {
            state_indices: state_indices.clone(),
            parameter_indices: parameter_indices.clone(),
        })?;
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
    specifications: impl Iterator<Item = (&'a str, &'a UncertaintySpec, &'a str)>,
    subject: &'static str,
    missing: &mut Vec<String>,
    assumptions: &mut Vec<String>,
) -> Result<ResolvedCovariance, ModelError> {
    if let Some(matrix) = supplied {
        validate_covariance(&matrix, dimension, subject)?;
        let has_information = matrix.iter().flatten().any(|value| *value != 0.0);
        return Ok(ResolvedCovariance {
            matrix: Some(matrix),
            has_information,
        });
    }

    let mut diagonal = vec![0.0; dimension];
    let mut complete = true;
    let mut has_information = false;
    for (index, (id, uncertainty, unit)) in specifications.enumerate() {
        if !relevant_ids.contains(id) {
            continue;
        }
        match uncertainty.variance_in(unit) {
            Ok(Some(value)) => {
                diagonal[index] = value;
                has_information |= value > 0.0;
            }
            Ok(None) => {
                complete = false;
                missing.push(format!(
                    "{subject}:{id} covariance missing: {}",
                    uncertainty
                        .missing_reason()
                        .unwrap_or_else(|| "unknown source".into())
                ));
            }
            Err(_) => {
                complete = false;
                missing.push(format!("{subject}:{id} covariance is invalid"));
            }
        }
    }
    if complete {
        assumptions.push(format!(
            "{subject} uncertainties were propagated as an independent diagonal covariance"
        ));
        let matrix = (0..dimension)
            .map(|row| {
                (0..dimension)
                    .map(|column| if row == column { diagonal[row] } else { 0.0 })
                    .collect()
            })
            .collect();
        Ok(ResolvedCovariance {
            matrix: Some(matrix),
            has_information,
        })
    } else {
        Ok(ResolvedCovariance {
            matrix: None,
            has_information,
        })
    }
}

fn validate_covariance(
    covariance: &[Vec<f64>],
    expected: usize,
    subject: &'static str,
) -> Result<(), ModelError> {
    if covariance.len() != expected
        || covariance
            .iter()
            .any(|row| row.len() != expected || row.iter().any(|value| !value.is_finite()))
    {
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
    Ok(())
}

fn derivative_is_required<'a>(
    id: &str,
    covariance: Option<&[Vec<f64>]>,
    indices: &BTreeMap<String, usize>,
    specifications: impl Iterator<Item = (&'a str, &'a UncertaintySpec)>,
) -> bool {
    let Some(index) = indices.get(id).copied() else {
        return true;
    };
    if let Some(matrix) = covariance {
        return matrix[index].iter().any(|value| *value != 0.0)
            || matrix.iter().any(|row| row[index] != 0.0);
    }
    specifications
        .filter(|(candidate, _)| *candidate == id)
        .any(|(_, uncertainty)| !matches!(uncertainty, UncertaintySpec::Deterministic))
}

#[allow(clippy::too_many_arguments)]
fn record_relevant_missing_derivatives<'a>(
    jacobian: &AggregatedJacobian,
    covariance: Option<&[Vec<f64>]>,
    indices: &BTreeMap<String, usize>,
    specifications: impl Iterator<Item = (&'a str, &'a UncertaintySpec)> + Clone,
    subject: &str,
    missing_sources: &mut Vec<String>,
) {
    for (component, id, message) in &jacobian.missing {
        if derivative_is_required(id, covariance, indices, specifications.clone()) {
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
