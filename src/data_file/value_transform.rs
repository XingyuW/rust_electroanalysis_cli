#![allow(clippy::too_many_arguments)]

//! Configurable value transformations applied at the visualization layer.
//!
//! This module provides [`ValueTransform`] — a composable pipeline of
//! mathematical transformations applied to x and/or y values **just before
//! plotting**.  Raw data and intermediate processing remain unmodified; only
//! the values handed to the rendering engine are affected.
//!
//! # Supported transform types
//!
//! | Type | TOML key | Formula | Parameters |
//! |------|----------|---------|------------|
//! | Logarithmic | `"log"` | log_b(x) | `base` (default `10`) |
//! | Negative logarithmic | `"-log"` | -log_b(x) | `base` (default `10`) |
//! | Linear | `"linear"` | a·x + b | `a` (default 1.0), `b` (default 0.0) |
//!
//! # Numerical robustness
//!
//! * **Log of non-positive values** — silently replaced with `f64::NAN` and
//!   a warning is emitted via the returned [`TransformWarning`] list.  NaN
//!   points are skipped by the rendering engine, preventing panics without
//!   blocking the pipeline.
//! * **NaN / infinity propagation** — pre-existing NaN or infinite values
//!   pass through unchanged (IEEE 754 semantics).
//!
//! # Configuration
//!
//! Transforms are declared inside any style block (`[shared.style]`,
//! `[eis.style]`, `[generic_plot.individual_style]`, etc.) via optional
//! TOML fields:
//!
//! ```toml
//! [shared.style]
//! x_transform = "log"        # log base 10 on x by default
//! x_transform_base = 2.718   # override log base (optional; e.g. natural log)
//! y_transform = "-log"       # negative log base 10 on y by default
//!
//! x_transform = "linear"     # linear transform on x
//! x_transform_a = 1000.0     # multiplier
//! x_transform_b = -50.0      # offset
//! ```

use crate::DEFAULT_LOG_BASE;
use serde::{Deserialize, Serialize};
use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// TransformKind — the TOML-level enum
// ─────────────────────────────────────────────────────────────────────────────

/// The type of value transformation, as specified in TOML.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformKind {
    /// Logarithmic: `log_base(x)`.
    #[serde(rename = "log")]
    Log,
    /// Negative logarithmic: `-log_base(x)`.
    #[serde(rename = "-log", alias = "negative_log", alias = "neg_log")]
    NegLog,
    /// Linear: `a * x + b`.
    #[serde(rename = "linear")]
    Linear,
}

impl fmt::Display for TransformKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Log => write!(f, "log"),
            Self::NegLog => write!(f, "-log"),
            Self::Linear => write!(f, "linear"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ValueTransform — fully resolved transform specification
// ─────────────────────────────────────────────────────────────────────────────

/// A single resolved value transformation ready to apply.
#[derive(Debug, Clone)]
pub enum ValueTransform {
    /// Logarithmic transform: `log_base(x)`.
    Log {
        /// Logarithm base; must be > 1.0.  Default is `e`.
        base: f64,
    },
    /// Negative logarithmic transform: `-log_base(x)`.
    NegLog {
        /// Logarithm base; must be > 1.0.  Default is `e`.
        base: f64,
    },
    /// Linear transform: `a * x + b`.
    Linear {
        /// Multiplicative factor; default is 1.0.
        a: f64,
        /// Additive offset; default is 0.0.
        b: f64,
    },
}

impl ValueTransform {
    /// Apply this transform to a single value.
    ///
    /// Returns the transformed value plus an optional warning when the input
    /// falls outside the valid domain (e.g. non-positive for log / -log).
    #[inline]
    pub fn apply(&self, x: f64) -> (f64, Option<TransformWarning>) {
        match self {
            Self::Log { base } => {
                if x <= 0.0 {
                    (
                        f64::NAN,
                        Some(TransformWarning::NonPositiveLogInput { value: x }),
                    )
                } else {
                    (x.log(*base), None)
                }
            }
            Self::NegLog { base } => {
                if x <= 0.0 {
                    (
                        f64::NAN,
                        Some(TransformWarning::NonPositiveLogInput { value: x }),
                    )
                } else {
                    (-x.log(*base), None)
                }
            }
            Self::Linear { a, b } => (a * x + b, None),
        }
    }

    /// Apply this transform to a slice of values in-place, collecting any
    /// warnings.
    pub fn apply_vec(&self, values: &mut [f64]) -> Vec<TransformWarning> {
        let mut warnings = Vec::new();
        for v in values.iter_mut() {
            let (result, warn) = self.apply(*v);
            *v = result;
            if let Some(w) = warn {
                // Deduplicate: only keep the first warning of each kind.
                if warnings.is_empty()
                    || !warnings
                        .iter()
                        .any(|existing: &TransformWarning| existing.same_kind(&w))
                {
                    warnings.push(w);
                }
            }
        }
        warnings
    }

    /// Render this transform as an expression over a symbolic variable name.
    ///
    /// Examples: `log10(x)`, `ln(y)`, `2*x + 3`.
    pub fn expression(&self, variable: &str) -> String {
        match self {
            Self::Log { base } => format!("{}({variable})", format_log_function_name(*base)),
            Self::NegLog { base } => {
                format!("-{}({variable})", format_log_function_name(*base))
            }
            Self::Linear { a, b } => format_linear_expression(*a, *b, variable),
        }
    }
}

impl fmt::Display for ValueTransform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Log { base } => write!(f, "log_{base}(x)"),
            Self::NegLog { base } => write!(f, "-log_{base}(x)"),
            Self::Linear { a, b } => {
                if *b == 0.0 {
                    write!(f, "{a} * x")
                } else if *b > 0.0 {
                    write!(f, "{a} * x + {b}")
                } else {
                    write!(f, "{a} * x - {}", b.abs())
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TransformWarning
// ─────────────────────────────────────────────────────────────────────────────

/// Non-fatal warnings produced during transformation.
#[derive(Debug, Clone)]
pub enum TransformWarning {
    /// A non-positive value was encountered in a logarithmic transform;
    /// replaced with NaN.
    NonPositiveLogInput { value: f64 },
}

impl TransformWarning {
    fn same_kind(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (
                Self::NonPositiveLogInput { .. },
                Self::NonPositiveLogInput { .. }
            )
        )
    }
}

impl fmt::Display for TransformWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositiveLogInput { value } => write!(
                f,
                "logarithmic transform: non-positive value {value} replaced with NaN"
            ),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AxisTransforms — per-axis transform pair
// ─────────────────────────────────────────────────────────────────────────────

/// Optional transforms for x and y axes, resolved from configuration.
#[derive(Debug, Clone, Default)]
pub struct AxisTransforms {
    pub x: Option<ValueTransform>,
    pub y: Option<ValueTransform>,
}

impl AxisTransforms {
    /// Returns `true` when no transforms are configured.
    pub fn is_empty(&self) -> bool {
        self.x.is_none() && self.y.is_none()
    }
}

/// Return the symbolic regression term for an axis.
///
/// * `None` transform -> `variable`
/// * transformed axis -> expression of the configured transform
pub fn regression_axis_term(transform: Option<&ValueTransform>, variable: &str) -> String {
    transform
        .map(|value| value.expression(variable))
        .unwrap_or_else(|| variable.to_string())
}

fn format_log_function_name(base: f64) -> String {
    if (base - 10.0).abs() < 1e-12 {
        return "log10".to_string();
    }
    if (base - std::f64::consts::E).abs() < 1e-12 {
        return "ln".to_string();
    }
    if (base - 2.0).abs() < 1e-12 {
        return "log2".to_string();
    }
    format!("log{}", format_number(base))
}

fn format_linear_expression(a: f64, b: f64, variable: &str) -> String {
    let a_str = format_number(a);
    let b_str = format_number(b.abs());

    if b.abs() < 1e-12 {
        if (a - 1.0).abs() < 1e-12 {
            return variable.to_string();
        }
        return format!("{a_str}*{variable}");
    }

    if (a - 1.0).abs() < 1e-12 {
        if b > 0.0 {
            return format!("{variable} + {b_str}");
        }
        return format!("{variable} - {b_str}");
    }

    if b > 0.0 {
        format!("{a_str}*{variable} + {b_str}")
    } else {
        format!("{a_str}*{variable} - {b_str}")
    }
}

fn format_number(value: f64) -> String {
    let mut rendered = format!("{value:.6}");
    if rendered.contains('.') {
        while rendered.ends_with('0') {
            rendered.pop();
        }
        if rendered.ends_with('.') {
            rendered.pop();
        }
    }
    if rendered == "-0" {
        "0".to_string()
    } else {
        rendered
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Resolution helpers (config fields → ValueTransform)
// ─────────────────────────────────────────────────────────────────────────────

/// Resolve a [`ValueTransform`] from the TOML config fields for one axis.
///
/// # Parameters
///
/// * `kind` — the transform type (`"log"`, `"-log"`, or `"linear"`), or
///   `None` for no
///   transform.
/// * `base` — log base; only meaningful when `kind == Some(Log)`.
/// * `a` — linear multiplier; only meaningful when `kind == Some(Linear)`.
/// * `b` — linear offset; only meaningful when `kind == Some(Linear)`.
///
/// # Errors
///
/// Returns an error string when the parameters are invalid (e.g. log base
/// ≤ 1.0).
pub fn resolve_transform(
    kind: Option<TransformKind>,
    base: Option<f64>,
    a: Option<f64>,
    b: Option<f64>,
) -> Result<Option<ValueTransform>, String> {
    match kind {
        None => Ok(None),
        Some(TransformKind::Log) => {
            let base = base.unwrap_or(DEFAULT_LOG_BASE);
            if !base.is_finite() || base <= 1.0 {
                return Err(format!(
                    "log transform base must be a finite value greater than 1.0, got {base}"
                ));
            }
            Ok(Some(ValueTransform::Log { base }))
        }
        Some(TransformKind::NegLog) => {
            let base = base.unwrap_or(DEFAULT_LOG_BASE);
            if !base.is_finite() || base <= 1.0 {
                return Err(format!(
                    "-log transform base must be a finite value greater than 1.0, got {base}"
                ));
            }
            Ok(Some(ValueTransform::NegLog { base }))
        }
        Some(TransformKind::Linear) => {
            let a = a.unwrap_or(1.0);
            let b = b.unwrap_or(0.0);
            if !a.is_finite() {
                return Err(format!("linear transform 'a' must be finite, got {a}"));
            }
            if !b.is_finite() {
                return Err(format!("linear transform 'b' must be finite, got {b}"));
            }
            Ok(Some(ValueTransform::Linear { a, b }))
        }
    }
}

/// Resolve an [`AxisTransforms`] from the per-axis TOML config fields.
pub fn resolve_axis_transforms(
    x_kind: Option<TransformKind>,
    x_base: Option<f64>,
    x_a: Option<f64>,
    x_b: Option<f64>,
    y_kind: Option<TransformKind>,
    y_base: Option<f64>,
    y_a: Option<f64>,
    y_b: Option<f64>,
) -> Result<AxisTransforms, String> {
    let x = resolve_transform(x_kind, x_base, x_a, x_b)
        .map_err(|e| format!("x-axis transform: {e}"))?;
    let y = resolve_transform(y_kind, y_base, y_a, y_b)
        .map_err(|e| format!("y-axis transform: {e}"))?;
    Ok(AxisTransforms { x, y })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_transform_positive() {
        let t = ValueTransform::Log { base: 10.0 };
        let (result, warn) = t.apply(100.0);
        assert!((result - 2.0).abs() < 1e-12);
        assert!(warn.is_none());
    }

    #[test]
    fn log_transform_non_positive() {
        let t = ValueTransform::Log { base: 10.0 };
        let (result, warn) = t.apply(-5.0);
        assert!(result.is_nan());
        assert!(warn.is_some());
    }

    #[test]
    fn neg_log_transform_positive() {
        let t = ValueTransform::NegLog { base: 10.0 };
        let (result, warn) = t.apply(100.0);
        assert!((result + 2.0).abs() < 1e-12);
        assert!(warn.is_none());
    }

    #[test]
    fn neg_log_transform_non_positive() {
        let t = ValueTransform::NegLog { base: 10.0 };
        let (result, warn) = t.apply(-5.0);
        assert!(result.is_nan());
        assert!(warn.is_some());
    }

    #[test]
    fn linear_transform() {
        let t = ValueTransform::Linear { a: 2.0, b: 3.0 };
        let (result, warn) = t.apply(5.0);
        assert!((result - 13.0).abs() < 1e-12);
        assert!(warn.is_none());
    }

    #[test]
    fn resolve_none() {
        assert!(resolve_transform(None, None, None, None).unwrap().is_none());
    }

    #[test]
    fn resolve_log_default_base() {
        let t = resolve_transform(Some(TransformKind::Log), None, None, None)
            .unwrap()
            .unwrap();
        if let ValueTransform::Log { base } = t {
            assert!((base - DEFAULT_LOG_BASE).abs() < 1e-12);
        } else {
            panic!("expected Log");
        }
    }

    #[test]
    fn resolve_log_invalid_base() {
        assert!(resolve_transform(Some(TransformKind::Log), Some(0.5), None, None).is_err());
        assert!(resolve_transform(Some(TransformKind::Log), Some(1.0), None, None).is_err());
    }

    #[test]
    fn resolve_neg_log_default_base() {
        let t = resolve_transform(Some(TransformKind::NegLog), None, None, None)
            .unwrap()
            .unwrap();
        if let ValueTransform::NegLog { base } = t {
            assert!((base - DEFAULT_LOG_BASE).abs() < 1e-12);
        } else {
            panic!("expected NegLog");
        }
    }

    #[test]
    fn resolve_neg_log_invalid_base() {
        assert!(resolve_transform(Some(TransformKind::NegLog), Some(0.5), None, None).is_err());
        assert!(resolve_transform(Some(TransformKind::NegLog), Some(1.0), None, None).is_err());
    }

    #[test]
    fn resolve_linear_defaults() {
        let t = resolve_transform(Some(TransformKind::Linear), None, None, None)
            .unwrap()
            .unwrap();
        if let ValueTransform::Linear { a, b } = t {
            assert!((a - 1.0).abs() < 1e-12);
            assert!(b.abs() < 1e-12);
        } else {
            panic!("expected Linear");
        }
    }

    #[test]
    fn regression_axis_term_formats_log_base_ten() {
        let term = regression_axis_term(Some(&ValueTransform::Log { base: 10.0 }), "x");
        assert_eq!(term, "log10(x)");
    }

    #[test]
    fn regression_axis_term_formats_natural_log() {
        let term = regression_axis_term(
            Some(&ValueTransform::Log {
                base: std::f64::consts::E,
            }),
            "y",
        );
        assert_eq!(term, "ln(y)");
    }

    #[test]
    fn regression_axis_term_formats_linear_identity() {
        let term = regression_axis_term(Some(&ValueTransform::Linear { a: 1.0, b: 0.0 }), "x");
        assert_eq!(term, "x");
    }
}
