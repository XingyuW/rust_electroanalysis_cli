//! Human-readable reporting helpers for fitted circuits.
//!
//! These structures and formatters are used to render parameter breakdowns in
//! text reports and logs without leaking parser internals to callers.

use super::{CircuitNode, ElementType, parse_circuit_string};
use crate::domain::ReportingError;
use crate::results::CircuitFitResult;
use std::collections::BTreeMap;

/// One fitted numeric value attached to a specific circuit-element parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitElementParameterValue {
    pub name: String,
    pub unit: String,
    pub value: f64,
}

/// Per-element section in the rendered composition report.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitElementBreakdown {
    pub element_label: String,
    pub element_symbol: &'static str,
    pub element_name: &'static str,
    pub parameters: Vec<CircuitElementParameterValue>,
}

/// Aggregated count by element type.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitElementCount {
    pub element_symbol: &'static str,
    pub element_name: &'static str,
    pub count: usize,
}

/// Structured report model used by text/JSON-style output formatting.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitCompositionReport {
    pub circuit_string: String,
    pub element_counts: Vec<CircuitElementCount>,
    pub elements: Vec<CircuitElementBreakdown>,
}

pub fn describe_fitted_circuit(
    circuit_string: &str,
    fitted_parameters: &[f64],
) -> Result<CircuitCompositionReport, ReportingError> {
    let circuit = parse_circuit_string(circuit_string)?;
    let expected_param_count = circuit.count_total_params();

    if fitted_parameters.len() != expected_param_count {
        return Err(ReportingError::ParameterCountMismatch {
            circuit: circuit_string.to_string(),
            expected: expected_param_count,
            actual: fitted_parameters.len(),
        });
    }

    let mut element_nodes = Vec::new();
    collect_element_nodes(&circuit, &mut element_nodes);

    let mut counts_by_type: BTreeMap<&'static str, CircuitElementCount> = BTreeMap::new();
    let mut elements = Vec::with_capacity(element_nodes.len());

    for (element_type, parameter_start, label_suffix) in element_nodes {
        let parameter_end = parameter_start + element_type.param_count();
        let parameter_values = &fitted_parameters[parameter_start..parameter_end];
        let parameter_names = element_type.param_names();
        let parameter_units = element_type.param_units();

        let parameters = parameter_names
            .into_iter()
            .zip(parameter_units)
            .zip(parameter_values.iter())
            .map(|((name, unit), &value)| CircuitElementParameterValue {
                name: name.to_string(),
                unit: unit.to_string(),
                value,
            })
            .collect();

        counts_by_type
            .entry(element_type.code())
            .and_modify(|entry| entry.count += 1)
            .or_insert(CircuitElementCount {
                element_symbol: element_type.code(),
                element_name: element_type.display_name(),
                count: 1,
            });

        elements.push(CircuitElementBreakdown {
            element_label: format!("{}{}", element_type.code(), label_suffix),
            element_symbol: element_type.code(),
            element_name: element_type.display_name(),
            parameters,
        });
    }

    Ok(CircuitCompositionReport {
        circuit_string: circuit_string.to_string(),
        element_counts: counts_by_type.into_values().collect(),
        elements,
    })
}

pub fn format_circuit_composition_report(
    composition: &CircuitCompositionReport,
    indent: &str,
) -> String {
    let detail_indent = format!("{indent}  ");
    let parameter_indent = format!("{indent}    ");
    let element_counts = if composition.element_counts.is_empty() {
        "n/a".to_string()
    } else {
        composition
            .element_counts
            .iter()
            .map(|count| {
                format!(
                    "{} ({}) = {}",
                    count.element_symbol, count.element_name, count.count
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    };

    let mut lines = vec![
        format!("{indent}Circuit Topology: {}", composition.circuit_string),
        format!("{indent}Element Counts: {element_counts}"),
        format!("{indent}Element Breakdown:"),
    ];

    for element in &composition.elements {
        lines.push(format!(
            "{detail_indent}- {} [{}: {}]",
            element.element_label, element.element_symbol, element.element_name
        ));
        for parameter in &element.parameters {
            if parameter.unit.is_empty() {
                lines.push(format!(
                    "{parameter_indent}{} = {:.6e}",
                    parameter.name, parameter.value
                ));
            } else {
                lines.push(format!(
                    "{parameter_indent}{} = {:.6e} {}",
                    parameter.name, parameter.value, parameter.unit
                ));
            }
        }
    }

    lines.join("\n")
}

pub fn format_fitted_circuit_composition(
    circuit_string: &str,
    fitted_parameters: &[f64],
    indent: &str,
) -> Result<String, ReportingError> {
    let composition = describe_fitted_circuit(circuit_string, fitted_parameters)?;
    Ok(format_circuit_composition_report(&composition, indent))
}

/// Format the named output of a direct circuit fit for CLI/report consumers.
pub fn format_circuit_fit_report(circuit_model: &str, fit: &CircuitFitResult) -> String {
    let mut lines = vec![
        format!("Circuit Model: {circuit_model}"),
        "Parameters:".to_string(),
        "Name | Value | Unit".to_string(),
        "-----|-------|-----".to_string(),
    ];

    for ((name, unit), value) in fit
        .parameter_names
        .iter()
        .zip(fit.parameter_units.iter())
        .zip(fit.fitted_parameters.iter())
    {
        lines.push(format!("{name} | {value:.6e} | {unit}"));
    }

    lines.push(format!("Fitted points: {}", fit.fitted_z_re.len()));
    lines.push("Fitted real impedance: available".to_string());
    lines.push("Fitted imaginary impedance: available".to_string());
    lines.push("Fitted magnitude: available".to_string());
    lines.push("Fitted phase: available".to_string());
    lines.join("\n")
}

fn collect_element_nodes(node: &CircuitNode, elements: &mut Vec<(ElementType, usize, String)>) {
    match node {
        CircuitNode::Element(element_type, parameter_start, label_suffix) => {
            elements.push((*element_type, *parameter_start, label_suffix.clone()));
        }
        CircuitNode::Series(nodes) | CircuitNode::Parallel(nodes) => {
            for child in nodes {
                collect_element_nodes(child, elements);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::describe_fitted_circuit;

    #[test]
    fn describes_fitted_circuit_with_element_counts_and_parameters() {
        let report =
            describe_fitted_circuit("R0-p(CPE1,R1)-Gw2", &[5.0, 2.5e-5, 0.88, 125.0, 0.35, 0.62])
                .expect("describe fitted circuit");

        assert_eq!(report.element_counts.len(), 3);
        assert_eq!(report.elements.len(), 4);
        assert_eq!(report.elements[0].element_label, "R0");
        assert_eq!(report.elements[1].element_label, "CPE1");
        assert_eq!(report.elements[2].element_label, "R1");
        assert_eq!(report.elements[3].element_label, "Gw2");
        assert_eq!(report.elements[1].parameters[0].name, "Q");
        assert_eq!(report.elements[1].parameters[1].name, "alpha");
        assert_eq!(report.elements[3].parameters[0].name, "sigma");
    }
}
