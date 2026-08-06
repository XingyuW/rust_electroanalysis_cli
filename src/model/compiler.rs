use super::{
    component::{ComponentBindings, ComponentDescriptor, IsmComponent, Jacobian, identity},
    definition::ModelDefinition,
    error::ModelError,
    graph::dependency_order,
    identifiability::IdentifiabilityReport,
    input::{ModelInput, units_compatible, validate_unit},
    output::{ComponentContribution, ObservationPrediction},
    parameter::{CompiledParameterSpec, ParameterValues},
    registry::ComponentRegistry,
    state::{CompiledStateSpec, ModelState},
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
            if let Some(voltage_v) = component.observation_voltage(state, parameters, input)? {
                if !voltage_v.is_finite() {
                    return Err(ModelError::NonFiniteContribution {
                        component: component.descriptor().id.clone(),
                    });
                }
                let descriptor = component.descriptor();
                let Some(owner) = &descriptor.voltage_contribution_owner else {
                    return Err(ModelError::UndeclaredVoltageContribution {
                        component: descriptor.id.clone(),
                    });
                };
                contributions.push(ComponentContribution {
                    component_id: descriptor.id.clone(),
                    owner: owner.clone(),
                    role: descriptor.role,
                    voltage_v,
                    source: descriptor.source.clone(),
                    validity_domain: descriptor.validity_domain.clone(),
                });
            }
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
        ObservationPrediction::new(
            self.component_contributions(state, parameters, input)?,
            observed_voltage_v,
        )
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
        let dimension = self.state_definitions.len();
        let mut result = vec![0.0; dimension];
        for component in &self.components {
            let jacobian = component.observation_jacobian(dimension, state, parameters, input)?;
            if jacobian.len() != dimension || jacobian.iter().any(|value| !value.is_finite()) {
                return Err(ModelError::JacobianDimension {
                    component: component.descriptor().id.clone(),
                });
            }
            for (result_value, component_value) in result.iter_mut().zip(jacobian) {
                *result_value += component_value;
            }
        }
        Ok(result)
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
        if component.voltage_contribution_owner.is_some()
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
