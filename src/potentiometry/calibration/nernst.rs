//! Nernst equations and temperature/response-sign conventions.

use super::error::CalibrationError;
use crate::results::calibration::{ResponseSign, TemperatureMode};

pub const GAS_CONSTANT_J_PER_MOL_K: f64 = 8.314_462_618_153_24;
pub const FARADAY_C_PER_MOL: f64 = 96_485.332_12;

pub fn theoretical_slope_v_per_decade(
    temperature_k: f64,
    charge: i32,
) -> Result<f64, CalibrationError> {
    if !temperature_k.is_finite() || temperature_k <= 0.0 {
        return Err(CalibrationError::InvalidObservation(
            "Nernst temperature must be finite and positive in kelvin".to_string(),
        ));
    }
    if charge == 0 {
        return Err(CalibrationError::InvalidObservation(
            "Nernst ion charge must be nonzero".to_string(),
        ));
    }
    Ok(
        std::f64::consts::LN_10 * GAS_CONSTANT_J_PER_MOL_K * temperature_k
            / (f64::from(charge) * FARADAY_C_PER_MOL),
    )
}

pub fn effective_temperature_k(
    mode: TemperatureMode,
    observation_temperature_k: Option<f64>,
    default_temperature_k: f64,
    reference_temperature_k: f64,
) -> Result<f64, CalibrationError> {
    let temperature = match mode {
        TemperatureMode::Constant => default_temperature_k,
        TemperatureMode::ReferenceNormalized => reference_temperature_k,
        TemperatureMode::ObservationSpecific => {
            observation_temperature_k.unwrap_or(default_temperature_k)
        }
    };
    if !temperature.is_finite() || temperature <= 0.0 {
        return Err(CalibrationError::InvalidObservation(
            "temperature must be positive in kelvin".to_string(),
        ));
    }
    if !reference_temperature_k.is_finite() || reference_temperature_k <= 0.0 {
        return Err(CalibrationError::InvalidConfiguration(
            "reference temperature must be positive in kelvin".to_string(),
        ));
    }
    Ok(temperature)
}

pub fn response_slope(
    theoretical_slope: f64,
    response_sign: ResponseSign,
) -> Result<f64, CalibrationError> {
    if !theoretical_slope.is_finite() || theoretical_slope == 0.0 {
        return Err(CalibrationError::InvalidObservation(
            "theoretical Nernst slope is invalid".to_string(),
        ));
    }
    Ok(match response_sign {
        ResponseSign::Auto => theoretical_slope,
        ResponseSign::Positive => theoretical_slope.abs(),
        ResponseSign::Negative => -theoretical_slope.abs(),
    })
}

pub fn evaluate_nernst(
    standard_potential_v: f64,
    activity: f64,
    temperature_k: f64,
    charge: i32,
    response_sign: ResponseSign,
) -> Result<f64, CalibrationError> {
    if !standard_potential_v.is_finite() || !activity.is_finite() || activity <= 0.0 {
        return Err(CalibrationError::InvalidObservation(
            "Nernst evaluation requires finite E0 and positive activity".to_string(),
        ));
    }
    let slope = response_slope(
        theoretical_slope_v_per_decade(temperature_k, charge)?,
        response_sign,
    )?;
    let result = standard_potential_v + slope * activity.log10();
    result.is_finite().then_some(result).ok_or_else(|| {
        CalibrationError::InvalidObservation("Nernst prediction was non-finite".to_string())
    })
}

pub fn activity_from_potential(
    potential_v: f64,
    standard_potential_v: f64,
    slope_v_per_decade: f64,
) -> Result<f64, CalibrationError> {
    if !potential_v.is_finite()
        || !standard_potential_v.is_finite()
        || !slope_v_per_decade.is_finite()
        || slope_v_per_decade.abs() < 1e-15
    {
        return Err(CalibrationError::InvalidPrediction(
            "potential inversion requires a finite, nonzero slope".to_string(),
        ));
    }
    let activity = 10_f64.powf((potential_v - standard_potential_v) / slope_v_per_decade);
    if activity.is_finite() && activity > 0.0 {
        Ok(activity)
    } else {
        Err(CalibrationError::InvalidPrediction(
            "inverted activity is non-finite or nonpositive".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate_nernst, theoretical_slope_v_per_decade};
    use crate::results::calibration::ResponseSign;

    #[test]
    fn monovalent_slope_at_room_temperature_is_about_59_mv_per_decade() {
        let slope = theoretical_slope_v_per_decade(298.15, 1).unwrap();
        assert!((slope - 0.05916).abs() < 0.0002);
        let potential = evaluate_nernst(0.2, 10.0, 298.15, 1, ResponseSign::Auto).unwrap();
        assert!((potential - (0.2 + slope)).abs() < 1e-12);
    }

    #[test]
    fn signed_charge_controls_slope_sign() {
        let positive = theoretical_slope_v_per_decade(298.15, 1).unwrap();
        let negative = theoretical_slope_v_per_decade(298.15, -1).unwrap();
        assert!((positive + negative).abs() < 1e-15);
    }
}
