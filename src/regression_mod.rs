//! Regression models for optional data-fit overlays in plots.
//!
//! # Overview
//!
//! This module provides a clean, stand-alone interface for computing
//! regression fits given raw x/y data vectors.  It is intentionally
//! decoupled from the plotting layer: it only produces **fitted curve
//! points** (and associated statistics) that the caller can then pass into
//! the standard [`crate::plottings::PlotSeries`] machinery.
//!
//! # Supported models
//!
//! | `RegressionKind` | Description |
//! |-----------------|-------------|
//! | `linear`        | Ordinary least-squares (OLS) linear fit: y = m·x + b |
//!
//! Additional variants (polynomial, exponential, power-law …) can be added
//! by extending [`RegressionKind`] and implementing the corresponding branch
//! inside [`compute_regression`] without changing any call-site code.
//!
//! # Configuration
//!
//! Regression is activated from `plot_config.toml` by adding a `regression`
//! field inside any `style`, `individual_style`, or `combined_style` block:
//!
//! ```toml
//! [[generic_plot]]
//! input_dir = "data/my_data"
//!
//! [generic_plot.style]
//! regression = "linear"   # scatter + OLS line overlay
//! ```
//!
//! Omitting `regression` (the default) preserves existing line-rendering
//! behaviour unchanged.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// Regression model variant.
///
/// Controls which mathematical model is fitted to the data and overlaid as
/// a continuous curve on the plot.
///
/// New variants can be added in the future (e.g. `Polynomial`, `Exponential`)
/// by extending this enum and adding the corresponding branch in
/// [`compute_regression`]; no other code changes are required.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegressionKind {
    /// Ordinary least-squares linear regression: y = slope·x + intercept.
    ///
    /// Requires at least 2 data points with non-identical x values.
    Linear,
}

/// Statistics and parameters returned by an OLS linear fit.
///
/// Use [`fit_linear`] to produce a `LinearFit`, then [`linear_curve_points`]
/// to generate a dense curve for rendering.
#[derive(Clone, Debug)]
pub struct LinearFit {
    /// Slope coefficient m in y = m·x + b.
    pub slope: f64,
    /// Intercept b in y = m·x + b.
    pub intercept: f64,
    /// Coefficient of determination R² ∈ [0, 1].
    ///
    /// R² = 1 means the linear model explains all variance in y.
    /// R² = 0 means the model explains none of the variance (no better than ȳ).
    pub r_squared: f64,
    /// Number of samples used for the fit.
    pub sample_count: usize,
    /// Root mean squared error in the fitted space.
    pub rmse: f64,
    /// Mean absolute error in the fitted space.
    pub mae: f64,
    /// Pearson correlation coefficient between x and y in the fitted space.
    pub correlation_coefficient: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// Core computation
// ─────────────────────────────────────────────────────────────────────────────

/// Fit a linear model y = m·x + b using ordinary least squares.
///
/// # Errors
///
/// Returns an error string when:
/// * `x` and `y` have different lengths.
/// * Fewer than 2 data points are provided.
/// * All x values are identical (denominator = 0 → indeterminate system).
pub fn fit_linear(x: &[f64], y: &[f64]) -> Result<LinearFit, String> {
    let n = x.len();
    if n < 2 {
        return Err(format!(
            "linear regression requires at least 2 data points, got {n}"
        ));
    }
    if n != y.len() {
        return Err(format!(
            "x and y must have equal length ({n} vs {})",
            y.len()
        ));
    }

    let n_f = n as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xx: f64 = x.iter().map(|xi| xi * xi).sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();

    let denom = n_f * sum_xx - sum_x * sum_x;
    if denom.abs() < f64::EPSILON {
        return Err("linear regression is indeterminate: all x values are identical".to_string());
    }

    let slope = (n_f * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n_f;

    // R² = 1 − SS_res / SS_tot
    let mean_y = sum_y / n_f;
    let ss_tot: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();
    let ss_res: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| {
            let predicted = slope * xi + intercept;
            (yi - predicted).powi(2)
        })
        .sum();
    let r_squared = if ss_tot < f64::EPSILON {
        // All y values are the same — perfect fit by convention.
        1.0
    } else {
        (1.0 - ss_res / ss_tot).clamp(0.0, 1.0)
    };

    let rmse = (ss_res / n_f).sqrt();
    let mae = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| {
            let predicted = slope * xi + intercept;
            (yi - predicted).abs()
        })
        .sum::<f64>()
        / n_f;

    let mean_x = sum_x / n_f;
    let cov_xy: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum::<f64>()
        / n_f;
    let var_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum::<f64>() / n_f;
    let var_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum::<f64>() / n_f;
    let correlation_coefficient = {
        let denom = (var_x * var_y).sqrt();
        if denom < f64::EPSILON {
            0.0
        } else {
            (cov_xy / denom).clamp(-1.0, 1.0)
        }
    };

    Ok(LinearFit {
        slope,
        intercept,
        r_squared,
        sample_count: n,
        rmse,
        mae,
        correlation_coefficient,
    })
}

/// Generate a dense set of curve points for a [`LinearFit`] over a given
/// x range.
///
/// Returns `n_points` evenly-spaced (x, y) pairs from `x_min` to `x_max`
/// (inclusive), suitable for rendering as a continuous line.
///
/// `n_points` is silently clamped to a minimum of 2.
pub fn linear_curve_points(
    fit: &LinearFit,
    x_min: f64,
    x_max: f64,
    n_points: usize,
) -> Vec<(f64, f64)> {
    let n = n_points.max(2);
    (0..n)
        .map(|i| {
            let t = i as f64 / (n - 1) as f64;
            let x = x_min + t * (x_max - x_min);
            let y = fit.slope * x + fit.intercept;
            (x, y)
        })
        .collect()
}

/// Compute a regression curve and return both the dense fitted curve points
/// and the underlying [`LinearFit`] statistics, ready for rendering and
/// optional on-plot annotation.
///
/// This is a richer variant of [`compute_regression`] that exposes the fit
/// parameters (slope, intercept, R²) alongside the curve so the caller can
/// display regression info on the figure when configured.
///
/// # Errors
///
/// Propagates any error from the underlying fit (insufficient data,
/// degenerate x values, etc.) as a human-readable string.
pub fn compute_regression_with_fit(
    x: &[f64],
    y: &[f64],
    kind: RegressionKind,
) -> Result<(Vec<(f64, f64)>, LinearFit), String> {
    match kind {
        RegressionKind::Linear => {
            let fit = fit_linear(x, y)?;
            let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
            let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let points = linear_curve_points(&fit, x_min, x_max, 200);
            Ok((points, fit))
        }
    }
}

/// Compute a regression curve for the given data vectors and return the
/// dense fitted curve points ready for rendering.
///
/// This is the primary entry point used by the plotting layer.  It
/// dispatches to the model-specific implementation selected by `kind`.
///
/// The returned points span the x-range `[min(x), max(x)]` at a fixed
/// resolution of 200 evenly-spaced samples — enough for a smooth curve on
/// any typical plot.
///
/// # Errors
///
/// Propagates any error from the underlying fit (insufficient data,
/// degenerate x values, etc.) as a human-readable string.
pub fn compute_regression(
    x: &[f64],
    y: &[f64],
    kind: RegressionKind,
) -> Result<Vec<(f64, f64)>, String> {
    match kind {
        RegressionKind::Linear => {
            let fit = fit_linear(x, y)?;
            let x_min = x.iter().cloned().fold(f64::INFINITY, f64::min);
            let x_max = x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            Ok(linear_curve_points(&fit, x_min, x_max, 200))
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    #[test]
    fn linear_fit_perfect_line() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 3.0, 5.0, 7.0, 9.0]; // slope=2, intercept=1
        let fit = fit_linear(&x, &y).unwrap();
        assert!(approx_eq(fit.slope, 2.0, 1e-10));
        assert!(approx_eq(fit.intercept, 1.0, 1e-10));
        assert!(approx_eq(fit.r_squared, 1.0, 1e-10));
        assert_eq!(fit.sample_count, 5);
        assert!(approx_eq(fit.rmse, 0.0, 1e-12));
        assert!(approx_eq(fit.mae, 0.0, 1e-12));
        assert!(approx_eq(fit.correlation_coefficient, 1.0, 1e-12));
    }

    #[test]
    fn linear_fit_r_squared_less_than_one() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 2.0, 3.0, 4.0, 4.0]; // imperfect fit
        let fit = fit_linear(&x, &y).unwrap();
        assert!(fit.r_squared < 1.0);
        assert!(fit.r_squared > 0.0);
        assert_eq!(fit.sample_count, 5);
        assert!(fit.rmse > 0.0);
        assert!(fit.mae > 0.0);
        assert!(fit.correlation_coefficient.is_finite());
        assert!(fit.correlation_coefficient.abs() <= 1.0);
    }

    #[test]
    fn linear_fit_error_on_too_few_points() {
        let err = fit_linear(&[1.0], &[2.0]).unwrap_err();
        assert!(err.contains("at least 2"));
    }

    #[test]
    fn linear_fit_error_on_identical_x() {
        let x = vec![5.0, 5.0, 5.0];
        let y = vec![1.0, 2.0, 3.0];
        let err = fit_linear(&x, &y).unwrap_err();
        assert!(err.contains("indeterminate"));
    }

    #[test]
    fn linear_fit_error_on_length_mismatch() {
        let err = fit_linear(&[1.0, 2.0], &[3.0]).unwrap_err();
        assert!(err.contains("equal length"));
    }

    #[test]
    fn compute_regression_returns_200_points() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|xi| 2.0 * xi + 1.0).collect();
        let pts = compute_regression(&x, &y, RegressionKind::Linear).unwrap();
        assert_eq!(pts.len(), 200);
        assert!(approx_eq(pts[0].0, 0.0, 1e-10));
        assert!(approx_eq(pts[199].0, 9.0, 1e-10));
    }
}
