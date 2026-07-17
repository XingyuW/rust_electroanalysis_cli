//! Calibration regression and nonlinear model fitting.

use super::error::CalibrationError;
use super::nernst::{effective_temperature_k, response_slope, theoretical_slope_v_per_decade};
use super::nicolsky_eisenman::{InterferentModelInput, evaluate_potential};
use crate::calibration_config::ResolvedCalibrationConfig;
use crate::results::calibration::{
    ActivityModelKind, CalibrationDomain, CalibrationFitStatistics, CalibrationFitStatus,
    CalibrationModelKind, CalibrationModelResult, CalibrationParameter, CalibrationWarning,
    CalibrationWarningKind, NernstSlopeMode, SelectivityCoefficient, WeightingMode,
};
use nalgebra::{DMatrix, DVector};

#[derive(Debug, Clone)]
pub struct FitInput<'a> {
    pub observations: &'a [crate::results::calibration::CalibrationObservation],
    pub config: &'a ResolvedCalibrationConfig,
}

pub fn fit_model(
    observations: &[crate::results::calibration::CalibrationObservation],
    config: &ResolvedCalibrationConfig,
    model: CalibrationModelKind,
) -> Result<CalibrationModelResult, CalibrationError> {
    let rows = observations
        .iter()
        .filter(|observation| {
            observation.potential_v.is_finite()
                && observation.log10_activity().is_some()
                && (model != CalibrationModelKind::NicolskyEisenman
                    || config
                        .nicolsky_eisenman
                        .interferents
                        .iter()
                        .all(|interferent| {
                            observation
                                .interferent_activities
                                .get(&interferent.name)
                                .is_some_and(|value| value.is_finite() && *value > 0.0)
                        }))
        })
        .collect::<Vec<_>>();
    if rows.is_empty() {
        return Err(CalibrationError::FitFailure {
            model: model.to_string(),
            reason: "no finite observations with the required activity inputs".to_string(),
        });
    }
    match model {
        CalibrationModelKind::Nernst => fit_nernst(&rows, config),
        CalibrationModelKind::ConductivityEmpirical => fit_conductivity(&rows, config),
        CalibrationModelKind::NicolskyEisenman => fit_nicolsky(&rows, config),
    }
}

fn fit_nernst(
    rows: &[&crate::results::calibration::CalibrationObservation],
    config: &ResolvedCalibrationConfig,
) -> Result<CalibrationModelResult, CalibrationError> {
    let default_temperature_k = 273.15 + config.temperature.default_celsius;
    let reference_temperature_k = 273.15 + config.temperature.reference_celsius;
    let mut warnings = Vec::new();
    let fixed_slopes = rows
        .iter()
        .map(|row| {
            let temperature = effective_temperature_k(
                config.temperature.mode,
                row.temperature_k,
                default_temperature_k,
                reference_temperature_k,
            )?;
            let slope = theoretical_slope_v_per_decade(temperature, row.ion_charge)?;
            response_slope(slope, config.nernst.response_sign)
        })
        .collect::<Result<Vec<_>, CalibrationError>>()?;

    let x = rows
        .iter()
        .map(|row| row.log10_activity().expect("filtered above"))
        .collect::<Vec<_>>();
    let y = rows.iter().map(|row| row.potential_v).collect::<Vec<_>>();
    let weights = observation_weights(
        rows,
        config.weighting.mode,
        config.weighting.minimum_standard_error_v,
    );
    let (design, target, names, units, theoretical_slope) = match config.nernst.slope_mode {
        NernstSlopeMode::FixedTheoretical => {
            let adjusted = y
                .iter()
                .zip(x.iter())
                .zip(fixed_slopes.iter())
                .map(|((potential, log_activity), slope)| potential - slope * log_activity)
                .collect::<Vec<_>>();
            (
                matrix_from_columns(&[vec![1.0; rows.len()]]),
                adjusted,
                vec!["E0"],
                vec!["V"],
                Some(median(&fixed_slopes)),
            )
        }
        NernstSlopeMode::Free | NernstSlopeMode::PriorConstrained => {
            if config.nernst.slope_mode == NernstSlopeMode::PriorConstrained {
                warnings.push(CalibrationWarning::new(
                    CalibrationWarningKind::UnsupportedPriorSlope,
                    "prior-constrained slope uses a transparent Gaussian prior pseudo-observation",
                ));
            }
            (
                matrix_from_columns(&[vec![1.0; rows.len()], x.clone()]),
                y,
                vec!["E0", "slope"],
                vec!["V", "V/decade"],
                Some(median(&fixed_slopes)),
            )
        }
    };
    let fit = weighted_linear_fit(
        &design,
        &target,
        weights.as_deref(),
        if config.nernst.slope_mode == NernstSlopeMode::PriorConstrained {
            config
                .nernst
                .prior_slope_v_per_decade
                .zip(config.nernst.prior_standard_deviation_v_per_decade)
        } else {
            None
        },
    )?;
    let fitted = fit.parameters.clone();
    let predictions = rows
        .iter()
        .zip(fixed_slopes.iter())
        .map(|(row, theoretical)| {
            let log_activity = row.log10_activity().expect("filtered above");
            let value = match config.nernst.slope_mode {
                NernstSlopeMode::FixedTheoretical => fitted[0] + theoretical * log_activity,
                NernstSlopeMode::Free | NernstSlopeMode::PriorConstrained => {
                    fitted[0] + fitted[1] * log_activity
                }
            };
            if value.is_finite() {
                Ok(value)
            } else {
                Err(CalibrationError::FitFailure {
                    model: "nernst".to_string(),
                    reason: "prediction was non-finite".to_string(),
                })
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut statistics = statistics(
        rows,
        &predictions,
        &fitted,
        weights.as_deref(),
        fit.condition_number,
        true,
    );
    statistics.parameter_covariance = fit.covariance.clone();
    statistics.leverage = fit.leverage.clone();
    statistics.cooks_distance = fit.cooks_distance.clone();
    statistics.convergence_reason = Some("stable SVD linear least-squares solution".to_string());
    if rows.len() < 3 {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::InsufficientConcentrationLevels,
            "fewer than three finite calibration observations are available",
        ));
    }
    let activity_range = x.iter().copied().reduce(f64::max).unwrap_or(0.0)
        - x.iter().copied().reduce(f64::min).unwrap_or(0.0);
    if activity_range < 1.0 {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::LimitedActivityRange,
            format!("calibration spans only {activity_range:.3} log10 activity decades"),
        ));
    }
    add_regression_warnings(&statistics, &mut warnings);
    let fitted_slope = match config.nernst.slope_mode {
        NernstSlopeMode::FixedTheoretical => theoretical_slope,
        NernstSlopeMode::Free | NernstSlopeMode::PriorConstrained => fitted.get(1).copied(),
    };
    if let Some(slope) = fitted_slope
        && let Some(theoretical) = theoretical_slope
    {
        if slope.abs() < 1.0e-12 {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::NonNernstianSlope,
                "fitted slope is zero or too small for reliable inversion",
            ));
        }
        if (slope.abs() - theoretical.abs()).abs() > 0.2 * theoretical.abs() {
            warnings.push(CalibrationWarning::new(
                    CalibrationWarningKind::NonNernstianSlope,
                    format!("fitted slope {slope:.6} V/decade differs materially from the theoretical magnitude {theoretical:.6} V/decade"),
                ));
        }
        if slope.signum() != theoretical.signum() {
            warnings.push(CalibrationWarning::new(
                CalibrationWarningKind::SlopeSignInconsistent,
                "fitted slope sign differs from the configured signed-charge convention",
            ));
        }
    }
    let parameters = names
        .iter()
        .zip(units.iter())
        .zip(fitted.iter())
        .enumerate()
        .map(|(index, ((name, unit), value))| CalibrationParameter {
            name: (*name).to_string(),
            unit: (*unit).to_string(),
            value: *value,
            standard_error: fit.standard_errors.get(index).copied().flatten(),
            lower_bound: None,
            upper_bound: None,
            source: None,
        })
        .collect::<Vec<_>>();
    let valid_domain = domain(rows);
    Ok(CalibrationModelResult {
        model_kind: CalibrationModelKind::Nernst,
        status: CalibrationFitStatus::Converged,
        activity_model: config.activity.model,
        parameters,
        selectivity_coefficients: Vec::new(),
        equation: "E = E0 + S log10(activity); activity model is configured separately".to_string(),
        theoretical_slope_v_per_decade: theoretical_slope,
        fitted_slope_v_per_decade: fitted_slope,
        slope_efficiency: fitted_slope.zip(theoretical_slope).map(|(a, b)| a / b),
        statistics,
        predicted_potential_v: predictions.clone(),
        residuals_v: y_residuals(rows, &predictions),
        standardized_residuals: standardized_residuals(rows, &predictions, &fit),
        confidence_intervals: Vec::new(),
        valid_domain,
        warnings,
    })
}

fn fit_conductivity(
    rows: &[&crate::results::calibration::CalibrationObservation],
    config: &ResolvedCalibrationConfig,
) -> Result<CalibrationModelResult, CalibrationError> {
    let empirical = &config.activity.conductivity_empirical;
    let mut x = Vec::with_capacity(rows.len());
    let mut conductivity = Vec::with_capacity(rows.len());
    for row in rows {
        let concentration =
            row.molar_concentration_mol_l
                .ok_or_else(|| CalibrationError::FitFailure {
                    model: "conductivity-empirical".to_string(),
                    reason: "molar concentration is required".to_string(),
                })?;
        let kappa = row
            .conductivity
            .as_ref()
            .ok_or_else(|| CalibrationError::FitFailure {
                model: "conductivity-empirical".to_string(),
                reason: "conductivity is missing".to_string(),
            })?
            .to_conductivity_s_per_m()?;
        x.push(concentration.log10() + empirical.b0);
        conductivity.push(kappa);
    }
    let y = rows.iter().map(|row| row.potential_v).collect::<Vec<_>>();
    let mut columns = vec![vec![1.0; rows.len()], x.clone()];
    if empirical.fit_b1 {
        columns.push(conductivity.clone());
    } else {
        for (index, value) in x.iter_mut().enumerate() {
            *value += empirical.b1 * conductivity[index];
        }
        columns[1] = x.clone();
    }
    let design = matrix_from_columns(&columns);
    let weights = observation_weights(
        rows,
        config.weighting.mode,
        config.weighting.minimum_standard_error_v,
    );
    let fit = weighted_linear_fit(&design, &y, weights.as_deref(), None)?;
    let predictions = (0..rows.len())
        .map(|index| {
            let mut prediction =
                fit.parameters[0] + fit.parameters[1] * x.get(index).copied().unwrap_or(0.0);
            if empirical.fit_b1 {
                prediction += fit.parameters[2] * conductivity[index];
            }
            prediction
        })
        .collect::<Vec<_>>();
    let mut statistics = statistics(
        rows,
        &predictions,
        &fit.parameters,
        weights.as_deref(),
        fit.condition_number,
        true,
    );
    statistics.parameter_covariance = fit.covariance.clone();
    statistics.leverage = fit.leverage.clone();
    statistics.cooks_distance = fit.cooks_distance.clone();
    statistics.convergence_reason =
        Some("stable SVD empirical conductivity regression".to_string());
    let mut warnings = vec![CalibrationWarning::new(
        CalibrationWarningKind::EmpiricalConductivityCorrection,
        "conductivity-empirical calibration is an explicitly empirical correction, not a thermodynamic activity law",
    )];
    add_regression_warnings(&statistics, &mut warnings);
    let mut parameters = vec![
        CalibrationParameter {
            name: "E0".to_string(),
            unit: "V".to_string(),
            value: fit.parameters[0],
            standard_error: fit.standard_errors.first().copied().flatten(),
            lower_bound: None,
            upper_bound: None,
            source: None,
        },
        CalibrationParameter {
            name: "slope".to_string(),
            unit: "V/decade".to_string(),
            value: fit.parameters[1],
            standard_error: fit.standard_errors.get(1).copied().flatten(),
            lower_bound: None,
            upper_bound: None,
            source: Some("empirical conductivity-corrected activity".to_string()),
        },
    ];
    if empirical.fit_b1 {
        parameters.push(CalibrationParameter {
            name: "conductivity_coefficient".to_string(),
            unit: "V m/S".to_string(),
            value: fit.parameters[2],
            standard_error: fit.standard_errors.get(2).copied().flatten(),
            lower_bound: None,
            upper_bound: None,
            source: Some("empirical conductivity correction".to_string()),
        });
    }
    Ok(CalibrationModelResult {
        model_kind: CalibrationModelKind::ConductivityEmpirical,
        status: CalibrationFitStatus::Converged,
        activity_model: ActivityModelKind::ConductivityEmpirical,
        parameters,
        selectivity_coefficients: Vec::new(),
        equation: "E = E0 + S[log10(c) + b0 + b1 conductivity] (empirical correction)".to_string(),
        theoretical_slope_v_per_decade: None,
        fitted_slope_v_per_decade: Some(fit.parameters[1]),
        slope_efficiency: None,
        statistics,
        predicted_potential_v: predictions.clone(),
        residuals_v: y_residuals(rows, &predictions),
        standardized_residuals: standardized_residuals(rows, &predictions, &fit),
        confidence_intervals: Vec::new(),
        valid_domain: domain(rows),
        warnings,
    })
}

fn fit_nicolsky(
    rows: &[&crate::results::calibration::CalibrationObservation],
    config: &ResolvedCalibrationConfig,
) -> Result<CalibrationModelResult, CalibrationError> {
    if config.nicolsky_eisenman.interferents.is_empty() {
        return Err(CalibrationError::FitFailure {
            model: "nicolsky-eisenman".to_string(),
            reason: "at least one interferent is required".to_string(),
        });
    }
    let mut internal = vec![median(
        &rows.iter().map(|row| row.potential_v).collect::<Vec<_>>(),
    )];
    for interferent in &config.nicolsky_eisenman.interferents {
        internal.push(
            interferent
                .selectivity_coefficient
                .unwrap_or(0.01)
                .max(1e-12)
                .ln(),
        );
    }
    let fit_selectivity = config.nicolsky_eisenman.fit_selectivity_coefficients;
    if !fit_selectivity {
        internal.truncate(1);
    }
    let weights = observation_weights(
        rows,
        config.weighting.mode,
        config.weighting.minimum_standard_error_v,
    );
    for _ in 0..80 {
        let predictions = ne_predictions(rows, config, &internal)?;
        let residuals = rows
            .iter()
            .zip(predictions.iter())
            .map(|(row, predicted)| row.potential_v - predicted)
            .collect::<Vec<_>>();
        let mut jacobian = DMatrix::zeros(rows.len(), internal.len());
        for row_index in 0..rows.len() {
            jacobian[(row_index, 0)] = 1.0;
            if fit_selectivity {
                for parameter_index in 1..internal.len() {
                    let mut perturbed = internal.clone();
                    perturbed[parameter_index] += 1e-5;
                    let p = ne_predictions(&[rows[row_index]], config, &perturbed)?[0];
                    jacobian[(row_index, parameter_index)] = (p - predictions[row_index]) / 1e-5;
                }
            }
        }
        let weighted_jacobian = apply_weights(&jacobian, weights.as_deref());
        let weighted_residuals =
            apply_weights_vector(&DVector::from_vec(residuals), weights.as_deref());
        let delta = weighted_jacobian
            .svd(true, true)
            .solve(&weighted_residuals, 1e-12)
            .map_err(|_| CalibrationError::FitFailure {
                model: "nicolsky-eisenman".to_string(),
                reason: "singular nonlinear Jacobian".to_string(),
            })?;
        let norm = delta.norm();
        for (index, value) in delta.iter().enumerate() {
            internal[index] += *value;
        }
        if norm < 1e-11 {
            break;
        }
    }
    let predictions = ne_predictions(rows, config, &internal)?;
    let residuals = y_residuals(rows, &predictions);
    let mut statistics = statistics(
        rows,
        &predictions,
        &internal,
        weights.as_deref(),
        None,
        true,
    );
    statistics.convergence_reason =
        Some("deterministic Gauss-Newton with logarithmic selectivity parameters".to_string());
    let mut selectivity_coefficients = Vec::new();
    let mut parameters = vec![CalibrationParameter {
        name: "E0".to_string(),
        unit: "V".to_string(),
        value: internal[0],
        standard_error: None,
        lower_bound: None,
        upper_bound: None,
        source: None,
    }];
    for (index, interferent) in config.nicolsky_eisenman.interferents.iter().enumerate() {
        let value = if fit_selectivity {
            internal[index + 1].exp()
        } else {
            interferent.selectivity_coefficient.unwrap_or(0.01)
        };
        selectivity_coefficients.push(SelectivityCoefficient {
            primary_analyte: config.analyte.name.clone(),
            interferent: interferent.name.clone(),
            value,
            source: if fit_selectivity {
                "experimentally_fitted"
            } else {
                interferent.source.as_str()
            }
            .to_string(),
            standard_error: None,
            confidence_interval: None,
        });
        if fit_selectivity {
            parameters.push(CalibrationParameter {
                name: format!("log_Kpot_{}", interferent.name),
                unit: "dimensionless (ln)".to_string(),
                value: internal[index + 1],
                standard_error: None,
                lower_bound: None,
                upper_bound: None,
                source: Some("logarithmic positive parameterization".to_string()),
            });
        }
    }
    let mut warnings = Vec::new();
    if selectivity_coefficients
        .iter()
        .any(|coefficient| coefficient.value <= 1.0e-10)
    {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::SelectivityCoefficientAtBound,
            "a positive selectivity coefficient is close to its numerical lower bound",
        ));
    }
    Ok(CalibrationModelResult {
        model_kind: CalibrationModelKind::NicolskyEisenman,
        status: CalibrationFitStatus::Converged,
        activity_model: config.activity.model,
        parameters,
        selectivity_coefficients,
        equation:
            "E = E0 + RT/(zF) ln(a_primary + sum(Kpot a_interferent^(z_primary/z_interferent)))"
                .to_string(),
        theoretical_slope_v_per_decade: rows.first().and_then(|row| {
            row.temperature_k.and_then(|temperature| {
                theoretical_slope_v_per_decade(temperature, row.ion_charge).ok()
            })
        }),
        fitted_slope_v_per_decade: None,
        slope_efficiency: None,
        statistics,
        predicted_potential_v: predictions,
        residuals_v: residuals,
        standardized_residuals: vec![None; rows.len()],
        confidence_intervals: Vec::new(),
        valid_domain: domain(rows),
        warnings,
    })
}

fn ne_predictions(
    rows: &[&crate::results::calibration::CalibrationObservation],
    config: &ResolvedCalibrationConfig,
    internal: &[f64],
) -> Result<Vec<f64>, CalibrationError> {
    rows.iter()
        .map(|row| {
            let primary = row.activity.ok_or_else(|| CalibrationError::FitFailure {
                model: "nicolsky-eisenman".to_string(),
                reason: "primary activity missing".to_string(),
            })?;
            let interferents = config
                .nicolsky_eisenman
                .interferents
                .iter()
                .enumerate()
                .map(|(index, config_item)| {
                    Ok(InterferentModelInput {
                        name: config_item.name.clone(),
                        charge: config_item.charge,
                        activity: *row
                            .interferent_activities
                            .get(&config_item.name)
                            .ok_or_else(|| {
                                CalibrationError::NotIdentifiable(format!(
                                    "missing activity for interferent '{}',",
                                    config_item.name
                                ))
                            })?,
                        selectivity_coefficient: if internal.len() > 1 {
                            internal[index + 1].exp()
                        } else {
                            config_item.selectivity_coefficient.unwrap_or(0.01)
                        },
                    })
                })
                .collect::<Result<Vec<_>, CalibrationError>>()?;
            let temperature = row
                .temperature_k
                .unwrap_or(273.15 + config.temperature.default_celsius);
            evaluate_potential(
                internal[0],
                primary,
                row.ion_charge,
                temperature,
                &interferents,
            )
        })
        .collect()
}

#[derive(Debug)]
struct LinearFit {
    parameters: Vec<f64>,
    standard_errors: Vec<Option<f64>>,
    covariance: Option<Vec<Vec<f64>>>,
    condition_number: Option<f64>,
    leverage: Vec<f64>,
    cooks_distance: Vec<f64>,
}

fn weighted_linear_fit(
    design: &DMatrix<f64>,
    target: &[f64],
    weights: Option<&[f64]>,
    prior_slope: Option<(f64, f64)>,
) -> Result<LinearFit, CalibrationError> {
    let mut matrix = apply_weights(design, weights);
    let mut vector = apply_weights_vector(&DVector::from_column_slice(target), weights);
    if let Some((slope, standard_deviation)) = prior_slope
        && standard_deviation.is_finite()
        && standard_deviation > 0.0
        && matrix.ncols() >= 2
    {
        let mut augmented = DMatrix::zeros(matrix.nrows() + 1, matrix.ncols());
        let mut augmented_vector = DVector::zeros(vector.len() + 1);
        augmented.rows_mut(0, matrix.nrows()).copy_from(&matrix);
        augmented_vector
            .rows_mut(0, vector.len())
            .copy_from(&vector);
        augmented[(matrix.nrows(), 1)] = 1.0 / standard_deviation;
        augmented_vector[vector.len()] = slope / standard_deviation;
        matrix = augmented;
        vector = augmented_vector;
    }
    let svd = matrix.clone().svd(true, true);
    let solution = svd.solve(&vector, 1e-12).map_err(|_| {
        CalibrationError::NotIdentifiable("singular calibration design matrix".to_string())
    })?;
    let singular_values = svd.singular_values;
    let rank = singular_values
        .iter()
        .filter(|value| **value > 1e-12)
        .count();
    if rank < solution.len() {
        return Err(CalibrationError::NotIdentifiable(
            "singular calibration design matrix".to_string(),
        ));
    }
    let condition_number = singular_values
        .iter()
        .copied()
        .filter(|value| *value > 1e-12)
        .fold(None, |state: Option<(f64, f64)>, value| match state {
            Some((minimum, maximum)) => Some((minimum.min(value), maximum.max(value))),
            None => Some((value, value)),
        })
        .map(|(minimum, maximum)| maximum / minimum);
    let weighted_residuals = &vector - &matrix * &solution;
    let sigma_squared = if matrix.nrows() > solution.len() {
        weighted_residuals.norm_squared() / (matrix.nrows() - solution.len()) as f64
    } else {
        0.0
    };
    let covariance = covariance_from_design(&matrix, sigma_squared);
    let standard_errors = covariance
        .as_ref()
        .map(|matrix| {
            (0..solution.len())
                .map(|index| {
                    matrix
                        .get(index)
                        .and_then(|row| row.get(index))
                        .and_then(|value| (*value >= 0.0).then_some(value.sqrt()))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![None; solution.len()]);
    let leverage = leverage_diagonal(&matrix);
    let cooks_distance = weighted_residuals
        .iter()
        .zip(leverage.iter())
        .take(target.len())
        .map(|(residual, leverage)| {
            if sigma_squared > 0.0 && *leverage < 1.0 {
                residual.powi(2) * *leverage
                    / (solution.len().max(1) as f64 * sigma_squared * (1.0 - leverage).powi(2))
            } else {
                0.0
            }
        })
        .collect();
    Ok(LinearFit {
        parameters: solution.iter().copied().collect(),
        standard_errors,
        covariance,
        condition_number,
        leverage,
        cooks_distance,
    })
}

fn leverage_diagonal(matrix: &DMatrix<f64>) -> Vec<f64> {
    let svd = matrix.clone().svd(true, false);
    let Some(u) = svd.u else {
        return vec![0.0; matrix.nrows()];
    };
    (0..matrix.nrows())
        .map(|row| {
            svd.singular_values
                .iter()
                .enumerate()
                .filter(|(_, value)| **value > 1e-12)
                .map(|(column, _)| u[(row, column)].powi(2))
                .sum::<f64>()
                .clamp(0.0, 1.0)
        })
        .collect()
}

fn statistics(
    rows: &[&crate::results::calibration::CalibrationObservation],
    predictions: &[f64],
    parameters: &[f64],
    weights: Option<&[f64]>,
    condition_number: Option<f64>,
    ordered: bool,
) -> CalibrationFitStatistics {
    let residuals = y_residuals(rows, predictions);
    let n = residuals.len();
    let k = parameters.len();
    let rss = residuals.iter().map(|value| value * value).sum::<f64>();
    let weighted_rss = weights.map(|weights| {
        residuals
            .iter()
            .zip(weights)
            .map(|(residual, weight)| weight * residual * residual)
            .sum()
    });
    let mean_y = rows.iter().map(|row| row.potential_v).sum::<f64>() / n.max(1) as f64;
    let total = rows
        .iter()
        .map(|row| (row.potential_v - mean_y).powi(2))
        .sum::<f64>();
    let r_squared = (total > 0.0).then_some(1.0 - rss / total);
    let adjusted = (n > k + 1 && total > 0.0)
        .then_some(1.0 - (1.0 - r_squared.unwrap_or(0.0)) * (n - 1) as f64 / (n - k - 1) as f64);
    let safe_rss = rss.max(f64::EPSILON);
    let n_f = n as f64;
    let aic = (n > 0).then_some(n_f * (safe_rss / n_f).ln() + 2.0 * k as f64);
    let aicc = (n > k + 1)
        .then_some(aic.unwrap_or(0.0) + 2.0 * k as f64 * (k as f64 + 1.0) / (n - k - 1) as f64);
    let bic = (n > 0).then_some(n_f * (safe_rss / n_f).ln() + k as f64 * n_f.ln());
    CalibrationFitStatistics {
        observations: n,
        fitted_parameters: k,
        rss: finite_option(rss),
        weighted_rss: weighted_rss.and_then(finite_option),
        rmse_v: finite_option((rss / n.max(1) as f64).sqrt()),
        mae_v: finite_option(
            residuals.iter().map(|value| value.abs()).sum::<f64>() / n.max(1) as f64,
        ),
        r_squared: r_squared.and_then(finite_option),
        adjusted_r_squared: adjusted.and_then(finite_option),
        aic,
        aicc,
        bic,
        criterion_delta: None,
        model_weight: None,
        parameter_covariance: None,
        condition_number,
        durbin_watson: ordered.then(|| durbin_watson(&residuals)),
        leverage: Vec::new(),
        cooks_distance: Vec::new(),
        convergence_reason: None,
    }
}

fn domain(rows: &[&crate::results::calibration::CalibrationObservation]) -> CalibrationDomain {
    let activities = rows
        .iter()
        .filter_map(|row| row.log10_activity())
        .collect::<Vec<_>>();
    let concentrations = rows
        .iter()
        .filter_map(|row| {
            row.molar_concentration_mol_l
                .filter(|value| value.is_finite() && *value > 0.0)
        })
        .collect::<Vec<_>>();
    let temperatures = rows
        .iter()
        .filter_map(|row| {
            row.temperature_k
                .filter(|value| value.is_finite() && *value > 0.0)
        })
        .collect::<Vec<_>>();
    let conductivities = rows
        .iter()
        .filter_map(|row| {
            row.conductivity
                .as_ref()
                .and_then(|value| value.to_conductivity_s_per_m().ok())
        })
        .collect::<Vec<_>>();
    CalibrationDomain {
        log10_activity_min: min_max(&activities).map(|value| value.0),
        log10_activity_max: min_max(&activities).map(|value| value.1),
        molar_concentration_min: min_max(&concentrations).map(|value| value.0),
        molar_concentration_max: min_max(&concentrations).map(|value| value.1),
        temperature_min_k: min_max(&temperatures).map(|value| value.0),
        temperature_max_k: min_max(&temperatures).map(|value| value.1),
        conductivity_min_s_per_m: min_max(&conductivities).map(|value| value.0),
        conductivity_max_s_per_m: min_max(&conductivities).map(|value| value.1),
    }
}

fn y_residuals(
    rows: &[&crate::results::calibration::CalibrationObservation],
    predictions: &[f64],
) -> Vec<f64> {
    rows.iter()
        .zip(predictions)
        .map(|(row, predicted)| row.potential_v - predicted)
        .collect()
}

fn standardized_residuals(
    rows: &[&crate::results::calibration::CalibrationObservation],
    predictions: &[f64],
    fit: &LinearFit,
) -> Vec<Option<f64>> {
    let _ = fit;
    let residuals = y_residuals(rows, predictions);
    let degrees_of_freedom = rows.len().saturating_sub(2);
    let sigma = (degrees_of_freedom > 0)
        .then(|| {
            (residuals.iter().map(|value| value * value).sum::<f64>() / degrees_of_freedom as f64)
                .sqrt()
        })
        .filter(|value| value.is_finite() && *value > 0.0);
    residuals
        .into_iter()
        .map(|value| sigma.map(|sigma| value / sigma))
        .collect()
}

fn observation_weights(
    rows: &[&crate::results::calibration::CalibrationObservation],
    mode: WeightingMode,
    minimum_standard_error_v: f64,
) -> Option<Vec<f64>> {
    (mode == WeightingMode::PotentialStandardError).then(|| {
        rows.iter()
            .map(|row| {
                row.potential_standard_error_v
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .map(|value| 1.0 / value.max(minimum_standard_error_v).powi(2))
                    .unwrap_or(1.0)
            })
            .collect()
    })
}

fn matrix_from_columns(columns: &[Vec<f64>]) -> DMatrix<f64> {
    let rows = columns.first().map_or(0, Vec::len);
    let mut matrix = DMatrix::zeros(rows, columns.len());
    for (column, values) in columns.iter().enumerate() {
        for (row, value) in values.iter().enumerate() {
            matrix[(row, column)] = *value;
        }
    }
    matrix
}

fn apply_weights(matrix: &DMatrix<f64>, weights: Option<&[f64]>) -> DMatrix<f64> {
    let mut result = matrix.clone();
    if let Some(weights) = weights {
        for row in 0..result.nrows() {
            let factor = weights.get(row).copied().unwrap_or(1.0).sqrt();
            for column in 0..result.ncols() {
                result[(row, column)] *= factor;
            }
        }
    }
    result
}

fn apply_weights_vector(vector: &DVector<f64>, weights: Option<&[f64]>) -> DVector<f64> {
    let mut result = vector.clone();
    if let Some(weights) = weights {
        for row in 0..result.len() {
            result[row] *= weights.get(row).copied().unwrap_or(1.0).sqrt();
        }
    }
    result
}

fn covariance_from_design(design: &DMatrix<f64>, sigma_squared: f64) -> Option<Vec<Vec<f64>>> {
    let parameters = design.ncols();
    if design.nrows() <= parameters || !sigma_squared.is_finite() {
        return None;
    }
    let svd = design.clone().svd(true, true);
    let u = svd.u?;
    let v_t = svd.v_t?;
    let threshold = 1e-12;
    let mut covariance = DMatrix::zeros(parameters, parameters);
    for index in 0..svd.singular_values.len() {
        let singular = svd.singular_values[index];
        if singular <= threshold {
            return None;
        }
        let v = v_t.row(index).transpose();
        covariance += (&v * v.transpose()) / singular.powi(2);
    }
    let _ = u;
    let matrix = covariance.iter().copied().collect::<Vec<_>>();
    Some(
        (0..parameters)
            .map(|row| {
                (0..parameters)
                    .map(|column| matrix[row * parameters + column] * sigma_squared)
                    .collect()
            })
            .collect(),
    )
}

fn durbin_watson(residuals: &[f64]) -> f64 {
    let numerator = residuals
        .windows(2)
        .map(|window| (window[1] - window[0]).powi(2))
        .sum::<f64>();
    let denominator = residuals.iter().map(|value| value * value).sum::<f64>();
    if denominator > 0.0 {
        numerator / denominator
    } else {
        0.0
    }
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut values = values.to_vec();
    values.sort_by(f64::total_cmp);
    if values.len().is_multiple_of(2) {
        (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
    } else {
        values[values.len() / 2]
    }
}

fn min_max(values: &[f64]) -> Option<(f64, f64)> {
    values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(None, |state, value| match state {
            Some((minimum, maximum)) => Some((minimum.min(value), maximum.max(value))),
            None => Some((value, value)),
        })
}

fn finite_option(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

fn add_regression_warnings(
    statistics: &CalibrationFitStatistics,
    warnings: &mut Vec<CalibrationWarning>,
) {
    if statistics
        .condition_number
        .is_some_and(|condition| condition > 1.0e8)
    {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::PoorConditionNumber,
            "calibration design matrix has a large condition number",
        ));
    }
    if statistics.parameter_covariance.is_none() {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::SingularCovariance,
            "parameter covariance is unavailable or singular",
        ));
    }
    let n = statistics.observations.max(1) as f64;
    let leverage_threshold = 2.0 * statistics.fitted_parameters.max(1) as f64 / n;
    if statistics
        .leverage
        .iter()
        .any(|leverage| *leverage > leverage_threshold)
    {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::HighLeverage,
            "at least one calibration observation has high leverage",
        ));
    }
    if statistics
        .cooks_distance
        .iter()
        .any(|distance| *distance > 4.0 / n)
    {
        warnings.push(CalibrationWarning::new(
            CalibrationWarningKind::InfluentialObservation,
            "at least one calibration observation is influential by Cook's distance",
        ));
    }
}

impl std::fmt::Display for CalibrationModelKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Nernst => "nernst",
            Self::NicolskyEisenman => "nicolsky-eisenman",
            Self::ConductivityEmpirical => "conductivity-empirical",
        })
    }
}
