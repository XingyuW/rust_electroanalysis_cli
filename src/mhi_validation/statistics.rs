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
        reason: String,
    },
}

pub fn wilson_95(numerator: u64, denominator: u64) -> MetricValueV1 {
    if denominator == 0 {
        return MetricValueV1::Unavailable {
            reason: "denominator_zero".into(),
        };
    }
    // The fixed V1 form uses z=1.959963984540054 and performs no continuity
    // correction or adjustment.  Clamp eliminates only roundoff beyond [0,1].
    let n = denominator as f64;
    let p = numerator as f64 / n;
    let z = 1.959_963_984_540_054_f64;
    let z2 = z * z;
    let centre = (p + z2 / (2.0 * n)) / (1.0 + z2 / n);
    let radius = z * ((p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt()) / (1.0 + z2 / n);
    MetricValueV1::Available {
        numerator,
        denominator,
        point_estimate: p,
        lower_confidence_bound: (centre - radius).clamp(0.0, 1.0),
        upper_confidence_bound: (centre + radius).clamp(0.0, 1.0),
    }
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
