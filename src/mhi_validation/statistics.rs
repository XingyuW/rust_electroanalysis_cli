//! Exact registered Phase-E summary statistics.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MetricValueV1 {
    Available {
        numerator: u64,
        denominator: u64,
        point_estimate: f64,
        lower_confidence_bound: f64,
        upper_confidence_bound: f64,
    },
    Unavailable {
        numerator: u64,
        denominator: u64,
        reason: String,
    },
}

pub const MAX_EXACT_F64_COUNT: u64 = 9_007_199_254_740_992;

/// The registered Wilson calculation with the exact Phase-E operation order.
/// Counts are checked before conversion so binary64 never silently rounds a
/// declared cohort or subset cardinality.
pub fn wilson_95_checked(numerator: u64, denominator: u64) -> Result<MetricValueV1, &'static str> {
    if numerator > denominator {
        return Err("InvalidBinomialCount");
    }
    if numerator > MAX_EXACT_F64_COUNT || denominator > MAX_EXACT_F64_COUNT {
        return Err("CountExceedsExactF64Range");
    }
    if denominator == 0 {
        return Ok(MetricValueV1::Unavailable {
            numerator,
            denominator,
            reason: "denominator_zero".into(),
        });
    }
    let n = denominator as f64;
    let p = numerator as f64 / n;
    let z = 1.959_963_984_540_054_f64;
    let z2 = z * z;
    let denominator_f64 = 1.0 + z2 / n;
    let centre = (p + z2 / (2.0 * n)) / denominator_f64;
    let radicand = p * (1.0 - p) / n + z2 / (4.0 * n * n);
    let half_width = z / denominator_f64 * radicand.sqrt();
    Ok(MetricValueV1::Available {
        numerator,
        denominator,
        point_estimate: p,
        lower_confidence_bound: (centre - half_width).max(0.0),
        upper_confidence_bound: (centre + half_width).min(1.0),
    })
}

/// Compatibility helper for the early Phase-E library surface.  Evaluation
/// uses [`wilson_95_checked`] so invalid cohort counts are hard errors rather
/// than scientific values.
pub fn wilson_95(numerator: u64, denominator: u64) -> MetricValueV1 {
    wilson_95_checked(numerator, denominator).unwrap_or(MetricValueV1::Unavailable {
        numerator,
        denominator,
        reason: "invalid_binomial_count".into(),
    })
}

pub fn balanced_accuracy(tp: u64, tn: u64, fp: u64, r#fn: u64) -> Result<f64, &'static str> {
    let positive = tp + r#fn;
    let negative = tn + fp;
    if positive == 0 {
        return Err("positive_class_denominator_zero");
    }
    if negative == 0 {
        return Err("negative_class_denominator_zero");
    }
    Ok(((tp as f64 / positive as f64) + (tn as f64 / negative as f64)) / 2.0)
}
