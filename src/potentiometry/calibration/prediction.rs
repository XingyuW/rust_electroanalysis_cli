//! Prediction and inversion from a stored calibration artifact.

use super::error::CalibrationError;
use super::nernst::{
    activity_from_potential, effective_temperature_k, evaluate_nernst, response_slope,
    theoretical_slope_v_per_decade,
};
use super::nicolsky_eisenman::{
    InterferentModelInput, evaluate_potential, primary_activity_from_potential,
};
use crate::results::calibration::{
    CalibrationModelKind, CalibrationPrediction, CalibrationWarning, CalibrationWarningKind,
    ResponseSign, StoredCalibrationModel,
};

pub fn predict_potential_from_activity(
    model: &StoredCalibrationModel,
    activity: f64,
    temperature_k: Option<f64>,
    interferent_activities: &std::collections::BTreeMap<String, f64>,
) -> Result<CalibrationPrediction, CalibrationError> {
    if !activity.is_finite() || activity <= 0.0 {
        return Err(CalibrationError::InvalidPrediction(
            "activity must be finite and positive".to_string(),
        ));
    }
    let temperature = prediction_temperature(model, temperature_k)?;
    let potential = match model.model_kind {
        CalibrationModelKind::Nernst => {
            let e0 = parameter(model, "E0")?;
            let slope = model
                .parameters
                .iter()
                .find(|parameter| parameter.name == "slope")
                .map(|parameter| parameter.value)
                .unwrap_or(theoretical_slope(model, temperature)?);
            e0 + slope * activity.log10()
        }
        CalibrationModelKind::ConductivityEmpirical => {
            let e0 = parameter(model, "E0")?;
            let slope = parameter(model, "slope")?;
            e0 + slope * activity.log10()
        }
        CalibrationModelKind::NicolskyEisenman => {
            let e0 = parameter(model, "E0")?;
            let interferents = model
                .selectivity_coefficients
                .iter()
                .map(|coefficient| {
                    Ok(InterferentModelInput {
                        name: coefficient.interferent.clone(),
                        charge: model
                            .configuration
                            .nicolsky_eisenman
                            .interferents
                            .iter()
                            .find(|item| item.name == coefficient.interferent)
                            .map(|item| item.charge)
                            .unwrap_or(1),
                        activity: *interferent_activities
                            .get(&coefficient.interferent)
                            .ok_or_else(|| {
                                CalibrationError::InvalidPrediction(format!(
                                    "missing activity for interferent '{}'",
                                    coefficient.interferent
                                ))
                            })?,
                        selectivity_coefficient: coefficient.value,
                    })
                })
                .collect::<Result<Vec<_>, CalibrationError>>()?;
            evaluate_potential(e0, activity, model.ion_charge, temperature, &interferents)?
        }
    };
    if !potential.is_finite() {
        return Err(CalibrationError::InvalidPrediction(
            "predicted potential is non-finite".to_string(),
        ));
    }
    let (extrapolated, distance) = domain_distance(model, activity.log10());
    let warnings = extrapolation_warning(extrapolated, distance);
    let potential_standard_error_v = linear_prediction_standard_error(model, activity.log10())
        .or(model.training_statistics.rmse_v);
    Ok(CalibrationPrediction {
        potential_v: Some(potential),
        potential_standard_error_v,
        predicted_activity: Some(activity),
        predicted_activity_lower: None,
        predicted_activity_upper: None,
        predicted_molar_concentration_mol_l: None,
        predicted_concentration_unit: None,
        temperature_k: Some(temperature),
        extrapolated,
        distance_from_domain_log10_activity: distance,
        warnings,
    })
}

pub fn predict_activity_from_potential(
    model: &StoredCalibrationModel,
    potential_v: f64,
    temperature_k: Option<f64>,
    interferent_activities: &std::collections::BTreeMap<String, f64>,
) -> Result<CalibrationPrediction, CalibrationError> {
    if !potential_v.is_finite() {
        return Err(CalibrationError::InvalidPrediction(
            "potential must be finite".to_string(),
        ));
    }
    let temperature = prediction_temperature(model, temperature_k)?;
    let activity = match model.model_kind {
        CalibrationModelKind::Nernst | CalibrationModelKind::ConductivityEmpirical => {
            let e0 = parameter(model, "E0")?;
            let slope = model
                .parameters
                .iter()
                .find(|parameter| parameter.name == "slope")
                .map(|parameter| parameter.value)
                .unwrap_or(theoretical_slope(model, temperature)?);
            activity_from_potential(potential_v, e0, slope)?
        }
        CalibrationModelKind::NicolskyEisenman => {
            let e0 = parameter(model, "E0")?;
            let interferents = model
                .selectivity_coefficients
                .iter()
                .map(|coefficient| {
                    Ok(InterferentModelInput {
                        name: coefficient.interferent.clone(),
                        charge: model
                            .configuration
                            .nicolsky_eisenman
                            .interferents
                            .iter()
                            .find(|item| item.name == coefficient.interferent)
                            .map(|item| item.charge)
                            .unwrap_or(1),
                        activity: *interferent_activities
                            .get(&coefficient.interferent)
                            .ok_or_else(|| {
                                CalibrationError::InvalidPrediction(format!(
                                    "missing activity for interferent '{}'",
                                    coefficient.interferent
                                ))
                            })?,
                        selectivity_coefficient: coefficient.value,
                    })
                })
                .collect::<Result<Vec<_>, CalibrationError>>()?;
            primary_activity_from_potential(
                potential_v,
                e0,
                model.ion_charge,
                temperature,
                &interferents,
            )?
        }
    };
    let (extrapolated, distance) = domain_distance(model, activity.log10());
    let mut warnings = extrapolation_warning(extrapolated, distance);
    let potential_standard_error_v = model.training_statistics.rmse_v;
    let (predicted_activity_lower, predicted_activity_upper) =
        activity_interval(model, potential_v, potential_standard_error_v);
    let concentration = (model.activity_model
        == crate::results::calibration::ActivityModelKind::Ideal)
        .then_some(activity)
        .filter(|_| model.configuration.analyte.molar_mass_g_per_mol.is_some());
    let missing_input_warning = match model.activity_model {
        crate::results::calibration::ActivityModelKind::ConductivityEmpirical => {
            CalibrationWarningKind::MissingConductivity
        }
        _ => CalibrationWarningKind::MissingIonicStrength,
    };
    if !matches!(
        model.activity_model,
        crate::results::calibration::ActivityModelKind::Ideal
    ) {
        warnings.push(CalibrationWarning::new(
            missing_input_warning,
            "concentration prediction is unavailable without an explicit activity-model inversion; the reported quantity is activity",
        ));
    }
    Ok(CalibrationPrediction {
        potential_v: Some(potential_v),
        potential_standard_error_v,
        predicted_activity: Some(activity),
        predicted_activity_lower,
        predicted_activity_upper,
        predicted_molar_concentration_mol_l: concentration,
        predicted_concentration_unit: concentration
            .map(|_| crate::potentiometry::units::QuantityUnit::MolPerL),
        temperature_k: Some(temperature),
        extrapolated,
        distance_from_domain_log10_activity: distance,
        warnings,
    })
}

fn linear_prediction_standard_error(
    model: &StoredCalibrationModel,
    log_activity: f64,
) -> Option<f64> {
    let covariance = model.training_statistics.parameter_covariance.as_ref()?;
    let variance = if covariance.len() >= 2 && covariance[0].len() >= 2 {
        covariance[0][0]
            + 2.0 * log_activity * covariance[0][1]
            + log_activity.powi(2) * covariance[1][1]
    } else {
        covariance.first()?.first().copied()?
    };
    variance.is_finite().then_some(variance.max(0.0).sqrt())
}

fn activity_interval(
    model: &StoredCalibrationModel,
    potential_v: f64,
    potential_standard_error_v: Option<f64>,
) -> (Option<f64>, Option<f64>) {
    let slope = model
        .parameters
        .iter()
        .find(|parameter| parameter.name == "slope")
        .map(|parameter| parameter.value);
    let Some(slope) = slope.filter(|value| value.is_finite() && value.abs() > 1e-15) else {
        return (None, None);
    };
    let e0 = model
        .parameters
        .iter()
        .find(|parameter| parameter.name == "E0")
        .map(|parameter| parameter.value);
    let Some(e0) = e0 else {
        return (None, None);
    };
    let log_activity = (potential_v - e0) / slope;
    let mut variance = potential_standard_error_v
        .filter(|value| value.is_finite() && *value >= 0.0)
        .map(|value| value.powi(2) / slope.powi(2))
        .unwrap_or(0.0);
    if let Some(covariance) = &model.training_statistics.parameter_covariance
        && covariance.len() >= 2
        && covariance[0].len() >= 2
        && covariance[1].len() >= 2
    {
        let gradient_e0 = -1.0 / slope;
        let gradient_slope = -log_activity / slope;
        variance += gradient_e0.powi(2) * covariance[0][0]
            + 2.0 * gradient_e0 * gradient_slope * covariance[0][1]
            + gradient_slope.powi(2) * covariance[1][1];
    }
    if !variance.is_finite() || variance <= 0.0 {
        return (None, None);
    }
    let z = confidence_z(model.configuration.uncertainty.confidence_level);
    let standard_error = variance.sqrt();
    let lower = 10_f64.powf(log_activity - z * standard_error);
    let upper = 10_f64.powf(log_activity + z * standard_error);
    (
        lower.is_finite().then_some(lower),
        upper.is_finite().then_some(upper),
    )
}

fn confidence_z(confidence_level: f64) -> f64 {
    if confidence_level >= 0.99 {
        2.576
    } else if confidence_level >= 0.975 {
        2.241
    } else if confidence_level >= 0.90 {
        1.645
    } else {
        1.96
    }
}

fn parameter(model: &StoredCalibrationModel, name: &str) -> Result<f64, CalibrationError> {
    model
        .parameters
        .iter()
        .find(|parameter| parameter.name == name)
        .map(|parameter| parameter.value)
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            CalibrationError::InvalidPrediction(format!(
                "stored calibration model lacks finite parameter '{name}'"
            ))
        })
}

fn prediction_temperature(
    model: &StoredCalibrationModel,
    temperature_k: Option<f64>,
) -> Result<f64, CalibrationError> {
    let default =
        QuantityTemperature::from_celsius(model.configuration.temperature.default_celsius)?;
    effective_temperature_k(
        model.temperature_mode,
        temperature_k,
        default,
        QuantityTemperature::from_celsius(model.configuration.temperature.reference_celsius)?,
    )
}

fn theoretical_slope(
    model: &StoredCalibrationModel,
    temperature_k: f64,
) -> Result<f64, CalibrationError> {
    response_slope(
        theoretical_slope_v_per_decade(temperature_k, model.ion_charge)?,
        model.response_sign,
    )
}

fn domain_distance(model: &StoredCalibrationModel, log_activity: f64) -> (bool, Option<f64>) {
    let Some((minimum, maximum)) = model
        .valid_domain
        .log10_activity_min
        .zip(model.valid_domain.log10_activity_max)
    else {
        return (false, None);
    };
    if log_activity < minimum {
        (true, Some(minimum - log_activity))
    } else if log_activity > maximum {
        (true, Some(log_activity - maximum))
    } else {
        (false, Some(0.0))
    }
}

fn extrapolation_warning(extrapolated: bool, distance: Option<f64>) -> Vec<CalibrationWarning> {
    extrapolated
        .then(|| {
            CalibrationWarning::new(
                CalibrationWarningKind::PredictionExtrapolation,
                format!(
                    "prediction is outside the calibration domain by {:.6} log10 activity",
                    distance.unwrap_or_default()
                ),
            )
        })
        .into_iter()
        .collect()
}

struct QuantityTemperature;
impl QuantityTemperature {
    fn from_celsius(value: f64) -> Result<f64, CalibrationError> {
        let kelvin = value + 273.15;
        (kelvin.is_finite() && kelvin > 0.0)
            .then_some(kelvin)
            .ok_or_else(|| {
                CalibrationError::InvalidPrediction(
                    "configured temperature is not physical".to_string(),
                )
            })
    }
}

#[allow(dead_code)]
fn _evaluate_nernst_for_api(
    e0: f64,
    activity: f64,
    temperature_k: f64,
    charge: i32,
    sign: ResponseSign,
) -> Result<f64, CalibrationError> {
    evaluate_nernst(e0, activity, temperature_k, charge, sign)
}
