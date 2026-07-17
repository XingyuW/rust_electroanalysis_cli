//! Stable result models returned by scientific workflows.

/// Complete output of a circuit fit.
///
/// The fields are deliberately named so callers do not need to rely on the
/// positional ordering of the former `FitCircuitResult` tuple alias.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitFitResult {
    /// Optimized physical parameter values.
    pub fitted_parameters: Vec<f64>,
    /// Names aligned with `fitted_parameters`.
    pub parameter_names: Vec<String>,
    /// Units aligned with `parameter_names`.
    pub parameter_units: Vec<String>,
    /// Fitted real impedance at each input frequency.
    pub fitted_z_re: Vec<f64>,
    /// Fitted imaginary impedance at each input frequency.
    pub fitted_z_im: Vec<f64>,
    /// Fitted impedance magnitude at each input frequency.
    pub fitted_magnitude: Vec<f64>,
    /// Fitted impedance phase in degrees at each input frequency.
    pub fitted_phase: Vec<f64>,
}

impl CircuitFitResult {
    /// Alias-style accessor using the longer semantic name.
    pub fn fitted_real_impedance(&self) -> &[f64] {
        &self.fitted_z_re
    }

    /// Alias-style accessor using the longer semantic name.
    pub fn fitted_imaginary_impedance(&self) -> &[f64] {
        &self.fitted_z_im
    }
}

#[cfg(test)]
mod tests {
    use super::CircuitFitResult;

    #[test]
    fn circuit_fit_result_keeps_all_named_fit_channels() {
        let result = CircuitFitResult {
            fitted_parameters: vec![1.0],
            parameter_names: vec!["R_0".to_string()],
            parameter_units: vec!["Ohm".to_string()],
            fitted_z_re: vec![2.0],
            fitted_z_im: vec![-3.0],
            fitted_magnitude: vec![3.605551275],
            fitted_phase: vec![-56.309932],
        };

        assert_eq!(result.fitted_parameters, vec![1.0]);
        assert_eq!(result.parameter_names, vec!["R_0"]);
        assert_eq!(result.parameter_units, vec!["Ohm"]);
        assert_eq!(result.fitted_real_impedance(), &[2.0]);
        assert_eq!(result.fitted_imaginary_impedance(), &[-3.0]);
        assert_eq!(result.fitted_magnitude.len(), 1);
        assert_eq!(result.fitted_phase.len(), 1);
    }
}
