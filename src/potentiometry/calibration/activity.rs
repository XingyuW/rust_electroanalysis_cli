//! Activity models used by potentiometric calibration.
//!
//! The functions in this module keep molar concentration, activity, and an
//! empirical conductivity correction distinct.  Conductivity is never used as
//! a thermodynamic activity coefficient unless the caller explicitly selects
//! the empirical model.

use super::error::CalibrationError;
use crate::calibration_config::ActivityConfig;
use crate::potentiometry::units::Quantity;
use crate::results::calibration::{ActivityModelKind, CalibrationWarning, CalibrationWarningKind};

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityEvaluation {
    pub activity: f64,
    pub activity_coefficient: Option<f64>,
    pub log10_activity: f64,
    pub empirical: bool,
    pub warnings: Vec<CalibrationWarning>,
}

/// Evaluate the configured activity model for one observation.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_activity(
    concentration: Option<&Quantity>,
    molar_mass_g_per_mol: Option<f64>,
    explicit_activity: Option<f64>,
    explicit_activity_coefficient: Option<f64>,
    charge: i32,
    ionic_strength_mol_l: Option<f64>,
    conductivity_s_per_m: Option<f64>,
    config: &ActivityConfig,
) -> Result<ActivityEvaluation, CalibrationError> {
    if charge == 0 {
        return Err(CalibrationError::ActivityModel(
            "target ion charge must be nonzero".to_string(),
        ));
    }

    if let Some(gamma) = explicit_activity_coefficient
        && (!gamma.is_finite() || gamma <= 0.0)
    {
        return Err(CalibrationError::ActivityModel(
            "explicit activity coefficient must be finite and positive".to_string(),
        ));
    }

    let concentration_molar = match concentration {
        Some(quantity) if quantity.unit == crate::potentiometry::units::QuantityUnit::MolPerKg => {
            None
        }
        Some(quantity) => Some(quantity.to_molar_concentration(molar_mass_g_per_mol)?),
        None => None,
    };
    let concentration_for_activity = concentration.map(|quantity| {
        if quantity.unit == crate::potentiometry::units::QuantityUnit::MolPerKg {
            quantity.value
        } else {
            concentration_molar.unwrap_or_default()
        }
    });
    if concentration_for_activity.is_some_and(|value| !value.is_finite() || value <= 0.0)
        && explicit_activity.is_none()
    {
        return Err(CalibrationError::ActivityModel(
            "concentration must be finite and positive".to_string(),
        ));
    }

    if let Some(activity) = explicit_activity {
        if !activity.is_finite() || activity <= 0.0 {
            return Err(CalibrationError::ActivityModel(
                "explicit activity must be finite and positive".to_string(),
            ));
        }
        return Ok(ActivityEvaluation {
            activity,
            activity_coefficient: explicit_activity_coefficient,
            log10_activity: activity.log10(),
            empirical: false,
            warnings: Vec::new(),
        });
    }

    let concentration_for_activity = concentration_for_activity.ok_or_else(|| {
        CalibrationError::ActivityModel(
            "concentration or explicit activity is required".to_string(),
        )
    })?;
    if let Some(gamma) = explicit_activity_coefficient {
        return finite_activity(
            concentration_for_activity * gamma,
            Some(gamma),
            false,
            Vec::new(),
        );
    }
    let mut warnings = Vec::new();

    let (gamma, empirical) = match config.model {
        ActivityModelKind::Ideal => (1.0, false),
        ActivityModelKind::UserProvided => {
            return Err(CalibrationError::ActivityModel(
                "user-provided activity model requires an explicit activity".to_string(),
            ));
        }
        ActivityModelKind::Davies => {
            let ionic_strength = ionic_strength_mol_l.ok_or_else(|| {
                CalibrationError::ActivityModel(
                    "Davies activity requires ionic strength".to_string(),
                )
            })?;
            validate_ionic_strength(ionic_strength)?;
            if ionic_strength > config.davies.maximum_ionic_strength_mol_l {
                warnings.push(CalibrationWarning::new(
                    CalibrationWarningKind::ActivityValidityExceeded,
                    format!(
                        "Davies ionic strength {:.6} mol/L exceeds configured validity range {:.6} mol/L",
                        ionic_strength, config.davies.maximum_ionic_strength_mol_l
                    ),
                ));
            }
            let root = ionic_strength.sqrt();
            let log_gamma = -config.davies.a_constant
                * f64::from(charge).powi(2)
                * (root / (1.0 + root) - 0.3 * ionic_strength);
            (10_f64.powf(log_gamma), false)
        }
        ActivityModelKind::ExtendedDebyeHuckel => {
            let ionic_strength = ionic_strength_mol_l.ok_or_else(|| {
                CalibrationError::ActivityModel(
                    "extended Debye-Huckel activity requires ionic strength".to_string(),
                )
            })?;
            validate_ionic_strength(ionic_strength)?;
            let ion_size = config
                .extended_debye_huckel
                .ion_size_parameter
                .ok_or_else(|| {
                    CalibrationError::ActivityModel(
                        "extended Debye-Huckel requires an ion-size parameter".to_string(),
                    )
                })?;
            if !ion_size.is_finite() || ion_size <= 0.0 {
                return Err(CalibrationError::ActivityModel(
                    "extended Debye-Huckel ion-size parameter must be positive".to_string(),
                ));
            }
            let ion_size_angstrom = match config
                .extended_debye_huckel
                .ion_size_unit
                .trim()
                .to_ascii_lowercase()
                .as_str()
            {
                "angstrom" | "å" | "a" => ion_size,
                "nm" | "nanometer" | "nanometers" => ion_size * 10.0,
                other => {
                    return Err(CalibrationError::ActivityModel(format!(
                        "unsupported extended Debye-Huckel ion-size unit '{other}'"
                    )));
                }
            };
            if ionic_strength > config.extended_debye_huckel.maximum_ionic_strength_mol_l {
                warnings.push(CalibrationWarning::new(
                    CalibrationWarningKind::ActivityValidityExceeded,
                    format!(
                        "extended Debye-Huckel ionic strength {:.6} mol/L exceeds configured validity range {:.6} mol/L",
                        ionic_strength, config.extended_debye_huckel.maximum_ionic_strength_mol_l
                    ),
                ));
            }
            let root = ionic_strength.sqrt();
            let log_gamma =
                -config.extended_debye_huckel.a_constant * f64::from(charge).powi(2) * root
                    / (1.0 + config.extended_debye_huckel.b_constant * ion_size_angstrom * root);
            (10_f64.powf(log_gamma), false)
        }
        ActivityModelKind::ConductivityEmpirical => {
            let conductivity = conductivity_s_per_m.ok_or_else(|| {
                CalibrationError::ActivityModel(
                    "empirical conductivity correction requires conductivity".to_string(),
                )
            })?;
            if !conductivity.is_finite() || conductivity < 0.0 {
                return Err(CalibrationError::ActivityModel(
                    "conductivity must be finite and nonnegative".to_string(),
                ));
            }
            let empirical = &config.conductivity_empirical;
            if empirical
                .minimum_conductivity_s_per_m
                .is_some_and(|minimum| conductivity < minimum)
                || empirical
                    .maximum_conductivity_s_per_m
                    .is_some_and(|maximum| conductivity > maximum)
            {
                warnings.push(CalibrationWarning::new(
                    CalibrationWarningKind::ConductivityExtrapolation,
                    "empirical conductivity correction is outside its configured training range",
                ));
            }
            let log_activity =
                concentration_for_activity.log10() + empirical.b0 + empirical.b1 * conductivity;
            let activity = 10_f64.powf(log_activity);
            return finite_activity(activity, None, true, warnings);
        }
    };

    if !gamma.is_finite() || gamma <= 0.0 {
        return Err(CalibrationError::ActivityModel(
            "activity coefficient calculation was non-finite or nonpositive".to_string(),
        ));
    }
    finite_activity(
        concentration_for_activity * gamma,
        Some(gamma),
        empirical,
        warnings,
    )
}

fn finite_activity(
    activity: f64,
    activity_coefficient: Option<f64>,
    empirical: bool,
    warnings: Vec<CalibrationWarning>,
) -> Result<ActivityEvaluation, CalibrationError> {
    if !activity.is_finite() || activity <= 0.0 {
        return Err(CalibrationError::ActivityModel(
            "computed activity was non-finite or nonpositive".to_string(),
        ));
    }
    Ok(ActivityEvaluation {
        activity,
        activity_coefficient,
        log10_activity: activity.log10(),
        empirical,
        warnings,
    })
}

fn validate_ionic_strength(value: f64) -> Result<(), CalibrationError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(CalibrationError::ActivityModel(
            "ionic strength must be finite and nonnegative".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_activity;
    use crate::calibration_config::ActivityConfig;
    use crate::potentiometry::units::{Quantity, QuantityUnit};
    use crate::results::calibration::ActivityModelKind;

    #[test]
    fn ideal_activity_is_molar_concentration() {
        let value = evaluate_activity(
            Some(&Quantity::new(2.0, QuantityUnit::MmolPerL).unwrap()),
            None,
            None,
            None,
            1,
            None,
            None,
            &ActivityConfig::default(),
        )
        .unwrap();
        assert!((value.activity - 0.002).abs() < 1e-12);
    }

    #[test]
    fn davies_activity_warns_outside_range() {
        let config = ActivityConfig {
            model: ActivityModelKind::Davies,
            ..ActivityConfig::default()
        };
        let value = evaluate_activity(
            Some(&Quantity::new(0.1, QuantityUnit::MolPerL).unwrap()),
            None,
            None,
            None,
            1,
            Some(0.6),
            None,
            &config,
        )
        .unwrap();
        assert!(value.activity < 0.1);
        assert!(!value.warnings.is_empty());
    }
}
