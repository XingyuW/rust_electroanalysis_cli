//! Nicolsky--Eisenman selectivity-aware potential model.

use super::error::CalibrationError;
use crate::potentiometry::calibration::nernst::{FARADAY_C_PER_MOL, GAS_CONSTANT_J_PER_MOL_K};

#[derive(Debug, Clone, PartialEq)]
pub struct InterferentModelInput {
    pub name: String,
    pub charge: i32,
    pub activity: f64,
    pub selectivity_coefficient: f64,
}

pub fn effective_activity(
    primary_activity: f64,
    primary_charge: i32,
    interferents: &[InterferentModelInput],
) -> Result<f64, CalibrationError> {
    if !primary_activity.is_finite() || primary_activity <= 0.0 || primary_charge == 0 {
        return Err(CalibrationError::NotIdentifiable(
            "primary activity must be positive and primary charge nonzero".to_string(),
        ));
    }
    let mut effective = primary_activity;
    for interferent in interferents {
        if interferent.charge == 0
            || !interferent.activity.is_finite()
            || interferent.activity <= 0.0
            || !interferent.selectivity_coefficient.is_finite()
            || interferent.selectivity_coefficient <= 0.0
        {
            return Err(CalibrationError::NotIdentifiable(format!(
                "interferent '{}' has invalid activity, charge, or positive selectivity coefficient",
                interferent.name
            )));
        }
        let contribution = interferent.selectivity_coefficient
            * interferent
                .activity
                .powf(f64::from(primary_charge) / f64::from(interferent.charge));
        effective += contribution;
    }
    if effective.is_finite() && effective > 0.0 {
        Ok(effective)
    } else {
        Err(CalibrationError::NotIdentifiable(
            "Nicolsky-Eisenman effective activity is nonpositive or non-finite".to_string(),
        ))
    }
}

pub fn evaluate_potential(
    standard_potential_v: f64,
    primary_activity: f64,
    primary_charge: i32,
    temperature_k: f64,
    interferents: &[InterferentModelInput],
) -> Result<f64, CalibrationError> {
    if !standard_potential_v.is_finite() || !temperature_k.is_finite() || temperature_k <= 0.0 {
        return Err(CalibrationError::InvalidObservation(
            "Nicolsky-Eisenman E0 and temperature must be finite and physical".to_string(),
        ));
    }
    let effective = effective_activity(primary_activity, primary_charge, interferents)?;
    let potential = standard_potential_v
        + GAS_CONSTANT_J_PER_MOL_K * temperature_k
            / (f64::from(primary_charge) * FARADAY_C_PER_MOL)
            * effective.ln();
    potential.is_finite().then_some(potential).ok_or_else(|| {
        CalibrationError::InvalidObservation(
            "Nicolsky-Eisenman potential was non-finite".to_string(),
        )
    })
}

pub fn primary_activity_from_potential(
    potential_v: f64,
    standard_potential_v: f64,
    primary_charge: i32,
    temperature_k: f64,
    interferents: &[InterferentModelInput],
) -> Result<f64, CalibrationError> {
    if primary_charge == 0
        || !potential_v.is_finite()
        || !standard_potential_v.is_finite()
        || !temperature_k.is_finite()
        || temperature_k <= 0.0
    {
        return Err(CalibrationError::InvalidPrediction(
            "Nicolsky-Eisenman inversion received invalid potential, charge, or temperature"
                .to_string(),
        ));
    }
    let effective =
        ((potential_v - standard_potential_v) * f64::from(primary_charge) * FARADAY_C_PER_MOL
            / (GAS_CONSTANT_J_PER_MOL_K * temperature_k))
            .exp();
    let interferent_sum = interferents.iter().try_fold(0.0, |sum, item| {
        if item.charge == 0 || item.activity <= 0.0 || item.selectivity_coefficient <= 0.0 {
            return Err(CalibrationError::NotIdentifiable(format!(
                "interferent '{}' is unavailable for inversion",
                item.name
            )));
        }
        Ok(sum
            + item.selectivity_coefficient
                * item
                    .activity
                    .powf(f64::from(primary_charge) / f64::from(item.charge)))
    })?;
    let primary = effective - interferent_sum;
    if primary.is_finite() && primary > 0.0 {
        Ok(primary)
    } else {
        Err(CalibrationError::NotIdentifiable(
            "Nicolsky-Eisenman inversion yielded nonpositive primary activity".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{InterferentModelInput, evaluate_potential, primary_activity_from_potential};

    #[test]
    fn positive_selectivity_coefficient_can_be_inverted() {
        let interferents = vec![InterferentModelInput {
            name: "K+".to_string(),
            charge: 1,
            activity: 0.01,
            selectivity_coefficient: 0.1,
        }];
        let potential = evaluate_potential(0.2, 0.1, 1, 298.15, &interferents).unwrap();
        let recovered =
            primary_activity_from_potential(potential, 0.2, 1, 298.15, &interferents).unwrap();
        assert!((recovered - 0.1).abs() < 1e-10);
    }
}
