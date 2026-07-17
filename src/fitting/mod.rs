//! Public fitting boundary.
//!
//! Numerical implementation remains in `impedance/`; this façade gives
//! callers a stable fitting namespace and keeps command orchestration from
//! reaching into optimizer details.

use crate::domain::FittingError;
use crate::results::CircuitFitResult;

/// Fit a circuit expression to measured impedance data.
pub fn fit_circuit(
    circuit_str: &str,
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
) -> Result<CircuitFitResult, FittingError> {
    crate::impedance::fit_circuit(circuit_str, frequencies, z_real, z_imag, phase_deg)
}

pub use crate::domain::FittingError as Error;
pub use crate::impedance::{
    ImpedanceFitter, clamp_to_bounds, guess_parameters, lin_kk_solver, sanitize_physical_params,
    transform_backward, transform_forward,
};
