#![allow(clippy::collapsible_if)]

use crate::{
    estimation::{
        environment::AlignedEnvironment,
        error::EstimationError,
        state::{CalibrationDomainStatus, EstimationWarning, EstimationWarningKind},
    },
    potentiometry::calibration::{
        nernst::{effective_temperature_k, response_slope, theoretical_slope_v_per_decade},
        nicolsky_eisenman::{InterferentModelInput, evaluate_potential},
    },
    results::calibration::{ActivityModelKind, CalibrationModelKind, StoredCalibrationModel},
};

pub trait CalibrationObservationModel {
    fn predict_potential(
        &self,
        log10_activity: f64,
        environment: &AlignedEnvironment,
    ) -> Result<f64, EstimationError>;
    fn jacobian_log10_activity(
        &self,
        log10_activity: f64,
        environment: &AlignedEnvironment,
    ) -> Result<f64, EstimationError>;
    fn valid_domain_check(
        &self,
        log10_activity: f64,
        environment: &AlignedEnvironment,
    ) -> CalibrationDomainStatus;
    fn domain_distance(
        &self,
        log10_activity: f64,
        _environment: &AlignedEnvironment,
    ) -> Option<f64> {
        let _ = log10_activity;
        None
    }
    fn near_boundary(
        &self,
        log10_activity: f64,
        _environment: &AlignedEnvironment,
        fraction: f64,
    ) -> bool {
        self.domain_distance(log10_activity, _environment)
            .is_some_and(|distance| distance <= fraction)
    }
    /// Return voltage prediction variance from calibration-parameter
    /// uncertainty.  This is deliberately separate from residual scatter.
    fn prediction_uncertainty_v2(
        &self,
        _log10_activity: f64,
        _environment: &AlignedEnvironment,
    ) -> Option<f64> {
        None
    }
    fn activity_model_is_ideal(&self) -> bool {
        false
    }
    fn inverse_log10_activity(
        &self,
        potential_v: f64,
        environment: &AlignedEnvironment,
    ) -> Result<f64, EstimationError> {
        if !potential_v.is_finite() {
            return Err(EstimationError::Calibration(
                "potential is nonfinite".into(),
            ));
        }
        let mut lo = -20.0;
        let mut hi = 10.0;
        let increasing =
            self.predict_potential(hi, environment)? > self.predict_potential(lo, environment)?;
        for _ in 0..100 {
            let mid = (lo + hi) / 2.0;
            let value = self.predict_potential(mid, environment)?;
            if (value - potential_v).abs() < 1e-12 {
                return Ok(mid);
            }
            if (value < potential_v) == increasing {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        Ok((lo + hi) / 2.0)
    }
    fn warnings(&self, _environment: &AlignedEnvironment) -> Vec<EstimationWarning> {
        Vec::new()
    }
}

#[derive(Debug, Clone)]
pub struct StoredCalibrationObservationModel {
    pub model: StoredCalibrationModel,
}
impl StoredCalibrationObservationModel {
    pub fn new(model: StoredCalibrationModel) -> Result<Self, EstimationError> {
        if model.parameters.iter().any(|p| !p.value.is_finite()) {
            return Err(EstimationError::Calibration(
                "stored calibration contains nonfinite parameters".into(),
            ));
        }
        Ok(Self { model })
    }
    fn param(&self, name: &str) -> Result<f64, EstimationError> {
        self.model
            .parameters
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.value)
            .filter(|v| v.is_finite())
            .ok_or_else(|| {
                EstimationError::Calibration(format!("stored calibration lacks parameter '{name}'"))
            })
    }
    fn temperature(&self, e: &AlignedEnvironment) -> Result<f64, EstimationError> {
        let default = self.model.configuration.temperature.default_celsius + 273.15;
        let reference = self.model.configuration.temperature.reference_celsius + 273.15;
        effective_temperature_k(
            self.model.temperature_mode,
            e.temperature_k,
            default,
            reference,
        )
        .map_err(|x| EstimationError::Calibration(x.to_string()))
    }
    fn slope(&self, e: &AlignedEnvironment) -> Result<f64, EstimationError> {
        if let Some(v) = self
            .model
            .parameters
            .iter()
            .find(|p| p.name == "slope")
            .map(|p| p.value)
        {
            if v.is_finite() && v.abs() > 1e-15 {
                return Ok(v);
            }
        }
        let t = self.temperature(e)?;
        response_slope(
            theoretical_slope_v_per_decade(t, self.model.ion_charge)
                .map_err(|x| EstimationError::Calibration(x.to_string()))?,
            self.model.response_sign,
        )
        .map_err(|x| EstimationError::Calibration(x.to_string()))
    }
    fn interferents(
        &self,
        e: &AlignedEnvironment,
    ) -> Result<Vec<InterferentModelInput>, EstimationError> {
        self.model
            .selectivity_coefficients
            .iter()
            .map(|c| {
                let activity = e
                    .interferent_activities
                    .get(&c.interferent)
                    .copied()
                    .ok_or_else(|| {
                        EstimationError::Calibration(format!(
                            "missing activity for interferent '{}'",
                            c.interferent
                        ))
                    })?;
                let charge = self
                    .model
                    .configuration
                    .nicolsky_eisenman
                    .interferents
                    .iter()
                    .find(|i| i.name == c.interferent)
                    .map(|i| i.charge)
                    .unwrap_or(1);
                Ok(InterferentModelInput {
                    name: c.interferent.clone(),
                    charge,
                    activity,
                    selectivity_coefficient: c.value,
                })
            })
            .collect()
    }
}

impl CalibrationObservationModel for StoredCalibrationObservationModel {
    fn activity_model_is_ideal(&self) -> bool {
        self.model.activity_model == ActivityModelKind::Ideal
    }
    fn domain_distance(
        &self,
        log10_activity: f64,
        _environment: &AlignedEnvironment,
    ) -> Option<f64> {
        let (lo, hi) = self
            .model
            .valid_domain
            .log10_activity_min
            .zip(self.model.valid_domain.log10_activity_max)?;
        Some(if log10_activity < lo {
            lo - log10_activity
        } else if log10_activity > hi {
            log10_activity - hi
        } else {
            (log10_activity - lo).min(hi - log10_activity)
        })
    }
    fn near_boundary(
        &self,
        log10_activity: f64,
        environment: &AlignedEnvironment,
        fraction: f64,
    ) -> bool {
        let Some((lo, hi)) = self
            .model
            .valid_domain
            .log10_activity_min
            .zip(self.model.valid_domain.log10_activity_max)
        else {
            return false;
        };
        if log10_activity < lo || log10_activity > hi {
            return false;
        }
        let span = (hi - lo).abs().max(f64::EPSILON);
        let distance = (log10_activity - lo).min(hi - log10_activity);
        let conductivity_near =
            if self.model.model_kind == CalibrationModelKind::ConductivityEmpirical {
                self.model
                    .valid_domain
                    .conductivity_min_s_per_m
                    .zip(self.model.valid_domain.conductivity_max_s_per_m)
                    .zip(environment.conductivity_s_per_m)
                    .is_some_and(|((low, high), value)| {
                        let conductivity_span = (high - low).abs().max(f64::EPSILON);
                        (value - low).min(high - value) / conductivity_span <= fraction
                    })
            } else {
                false
            };
        distance / span <= fraction || conductivity_near
    }
    fn prediction_uncertainty_v2(
        &self,
        log10_activity: f64,
        environment: &AlignedEnvironment,
    ) -> Option<f64> {
        if !log10_activity.is_finite() {
            return None;
        }
        let covariance = self.model.training_statistics.parameter_covariance.as_ref();
        let parameter_count = self.model.parameters.len();
        let mut gradient = vec![0.0; parameter_count];
        for (index, parameter) in self.model.parameters.iter().enumerate() {
            gradient[index] = match parameter.name.as_str() {
                "E0" => 1.0,
                "slope" => log10_activity,
                "conductivity_coefficient" => environment.conductivity_s_per_m.unwrap_or(0.0),
                _ => 0.0,
            };
        }
        let variance = covariance.and_then(|matrix| {
            if matrix.len() != parameter_count
                || matrix.iter().any(|row| row.len() != parameter_count)
            {
                return None;
            }
            let value = gradient
                .iter()
                .enumerate()
                .flat_map(|(i, gi)| {
                    gradient
                        .iter()
                        .enumerate()
                        .map(move |(j, gj)| gi * matrix[i][j] * gj)
                })
                .sum::<f64>();
            value.is_finite().then_some(value)
        });
        variance
            .or_else(|| {
                let value = self
                    .model
                    .parameters
                    .iter()
                    .zip(&gradient)
                    .filter_map(|(parameter, derivative)| {
                        parameter
                            .standard_error
                            .filter(|se| se.is_finite() && *se >= 0.0)
                            .map(|se| derivative.powi(2) * se.powi(2))
                    })
                    .sum::<f64>();
                (value > 0.0 && value.is_finite()).then_some(value)
            })
            .filter(|value| *value > 0.0 && value.is_finite())
    }
    fn predict_potential(
        &self,
        log10_activity: f64,
        e: &AlignedEnvironment,
    ) -> Result<f64, EstimationError> {
        if !log10_activity.is_finite() {
            return Err(EstimationError::Calibration(
                "log10 activity is nonfinite".into(),
            ));
        }
        let a = 10_f64.powf(log10_activity);
        if !a.is_finite() || a <= 0.0 {
            return Err(EstimationError::Calibration(
                "activity is outside representable range".into(),
            ));
        }
        let t = self.temperature(e)?;
        let e0 = self.param("E0")?;
        let value = match self.model.model_kind {
            CalibrationModelKind::Nernst => e0 + self.slope(e)? * log10_activity,
            CalibrationModelKind::ConductivityEmpirical => {
                let slope = self.slope(e)?;
                let b0 = self.model.configuration.activity.conductivity_empirical.b0;
                let k = e.conductivity_s_per_m.ok_or_else(|| {
                    EstimationError::Calibration(
                        "conductivity is required by the empirical calibration".into(),
                    )
                })?;
                if let Some(c) = self
                    .model
                    .parameters
                    .iter()
                    .find(|p| p.name == "conductivity_coefficient")
                    .map(|p| p.value)
                {
                    e0 + slope * (log10_activity + b0) + c * k
                } else {
                    e0 + slope
                        * (log10_activity
                            + b0
                            + self.model.configuration.activity.conductivity_empirical.b1 * k)
                }
            }
            CalibrationModelKind::NicolskyEisenman => {
                evaluate_potential(e0, a, self.model.ion_charge, t, &self.interferents(e)?)
                    .map_err(|x| EstimationError::Calibration(x.to_string()))?
            }
        };
        value.is_finite().then_some(value).ok_or_else(|| {
            EstimationError::Calibration("calibration prediction is nonfinite".into())
        })
    }
    fn jacobian_log10_activity(
        &self,
        log10_activity: f64,
        e: &AlignedEnvironment,
    ) -> Result<f64, EstimationError> {
        match self.model.model_kind {
            CalibrationModelKind::Nernst => self.slope(e),
            CalibrationModelKind::ConductivityEmpirical => self.slope(e),
            CalibrationModelKind::NicolskyEisenman => {
                let step = (f64::EPSILON.sqrt() * (1.0 + log10_activity.abs())).max(1e-7);
                let y1 = self.predict_potential(log10_activity - step, e)?;
                let y2 = self.predict_potential(log10_activity + step, e)?;
                Ok((y2 - y1) / (2.0 * step))
            }
        }
    }
    fn valid_domain_check(
        &self,
        log10_activity: f64,
        e: &AlignedEnvironment,
    ) -> CalibrationDomainStatus {
        let d = &self.model.valid_domain;
        let (mut outside, mut distance) =
            if let (Some(lo), Some(hi)) = (d.log10_activity_min, d.log10_activity_max) {
                if log10_activity < lo {
                    (true, lo - log10_activity)
                } else if log10_activity > hi {
                    (true, log10_activity - hi)
                } else {
                    (false, (log10_activity - lo).min(hi - log10_activity))
                }
            } else {
                return CalibrationDomainStatus::Missing;
            };
        if self.model.model_kind == CalibrationModelKind::ConductivityEmpirical {
            if let Some(k) = e.conductivity_s_per_m {
                if let (Some(lo), Some(hi)) =
                    (d.conductivity_min_s_per_m, d.conductivity_max_s_per_m)
                    && (k < lo || k > hi)
                {
                    outside = true;
                    distance = distance.max(if k < lo { lo - k } else { k - hi });
                }
            }
        }
        if outside {
            CalibrationDomainStatus::Outside
        } else if distance < 0.05 {
            CalibrationDomainStatus::NearBoundary
        } else {
            CalibrationDomainStatus::Inside
        }
    }
    fn warnings(&self, e: &AlignedEnvironment) -> Vec<EstimationWarning> {
        let mut w = e.warnings.clone();
        if matches!(
            self.model.activity_model,
            ActivityModelKind::Davies | ActivityModelKind::ExtendedDebyeHuckel
        ) && e.ionic_strength_mol_l.is_none()
        {
            w.push(EstimationWarning::new(
                EstimationWarningKind::MissingIonicStrength,
                "stored nonideal activity context has no ionic-strength observation",
            ));
        }
        w
    }
}
