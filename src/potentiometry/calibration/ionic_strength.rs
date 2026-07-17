//! Ionic-strength calculations from explicit solution composition.

use super::error::CalibrationError;
use crate::calibration_config::SolutionComponentConfig;

#[derive(Debug, Clone, PartialEq)]
pub struct SolutionComponent {
    pub name: String,
    pub concentration_mol_l: f64,
    pub charge: i32,
}

pub fn ionic_strength(components: &[SolutionComponent]) -> Result<f64, CalibrationError> {
    if components.is_empty() {
        return Err(CalibrationError::ActivityModel(
            "solution composition is empty".to_string(),
        ));
    }
    let mut sum = 0.0;
    for component in components {
        if !component.concentration_mol_l.is_finite()
            || component.concentration_mol_l < 0.0
            || component.charge == 0
        {
            return Err(CalibrationError::ActivityModel(format!(
                "invalid solution component '{}'",
                component.name
            )));
        }
        sum += component.concentration_mol_l * f64::from(component.charge).powi(2);
    }
    let result = 0.5 * sum;
    result.is_finite().then_some(result).ok_or_else(|| {
        CalibrationError::ActivityModel("ionic strength calculation was non-finite".to_string())
    })
}

pub fn ionic_strength_from_config(
    components: &[SolutionComponentConfig],
) -> Result<Option<f64>, CalibrationError> {
    if components.is_empty() {
        return Ok(None);
    }
    let converted = components
        .iter()
        .map(|component| SolutionComponent {
            name: component.name.clone(),
            concentration_mol_l: component.concentration_mol_l,
            charge: component.charge,
        })
        .collect::<Vec<_>>();
    ionic_strength(&converted).map(Some)
}

#[cfg(test)]
mod tests {
    use super::{SolutionComponent, ionic_strength};

    #[test]
    fn calculates_explicit_solution_ionic_strength() {
        let value = ionic_strength(&[
            SolutionComponent {
                name: "Na+".to_string(),
                concentration_mol_l: 0.1,
                charge: 1,
            },
            SolutionComponent {
                name: "Cl-".to_string(),
                concentration_mol_l: 0.1,
                charge: -1,
            },
        ])
        .unwrap();
        assert!((value - 0.1).abs() < 1e-12);
    }
}
