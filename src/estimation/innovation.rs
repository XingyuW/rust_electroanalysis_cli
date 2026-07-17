use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnovationRecord {
    pub timestamp_s: f64,
    pub innovation_v: f64,
    pub innovation_variance_v2: f64,
    pub standardized_innovation: f64,
    pub normalized_innovation_squared: f64,
    pub accepted: bool,
    pub gating_threshold: f64,
    pub kalman_gain: Vec<f64>,
    pub predicted_measurement_v: f64,
    pub measurement_residual_v: f64,
    pub log_likelihood: Option<f64>,
    #[serde(default)]
    pub measurement_variance_v2: f64,
    #[serde(default)]
    pub uninflated_measurement_variance_v2: f64,
    #[serde(default)]
    pub measurement_variance_source: String,
    #[serde(default)]
    pub variance_inflation_factor: f64,
    #[serde(default)]
    pub variance_inflation_reason: Option<String>,
}

pub fn gating_threshold(probability: f64) -> f64 {
    if probability >= 0.999 {
        10.828
    } else if probability >= 0.997 {
        8.807
    } else if probability >= 0.99 {
        6.635
    } else if probability >= 0.95 {
        3.841
    } else {
        2.706
    }
}
pub fn residual_autocorrelation(values: &[f64]) -> Option<f64> {
    if values.len() < 3 {
        return None;
    };
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let den = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>();
    (den > 0.0).then_some(
        values
            .windows(2)
            .map(|w| (w[0] - mean) * (w[1] - mean))
            .sum::<f64>()
            / den,
    )
}

pub fn autocorrelation_confidence_bound(sample_count: usize, confidence_level: f64) -> Option<f64> {
    if sample_count < 4 {
        return None;
    }
    // Bartlett's large-sample white-noise bound.  The normal quantile helper
    // is intentionally local to keep diagnostics independent of a statistics
    // crate; this is a diagnostic interval, not a hypothesis-test decision.
    let p = 0.5 + confidence_level.clamp(1e-6, 1.0 - 1e-6) / 2.0;
    Some(normal_quantile(p) / (sample_count as f64).sqrt())
}

/// Approximate confidence interval for the sample mean of scalar NIS values.
/// The interval is reported with its sample count in diagnostics; it is not a
/// replacement for a distributional check on individual innovations.
pub fn nis_consistency_interval(sample_count: usize, confidence_level: f64) -> Option<(f64, f64)> {
    if sample_count < 2 {
        return None;
    }
    let z = normal_quantile(0.5 + confidence_level.clamp(1e-6, 1.0 - 1e-6) / 2.0);
    let half_width = z * (2.0 / sample_count as f64).sqrt();
    Some((0.0_f64.max(1.0 - half_width), 1.0 + half_width))
}

fn normal_quantile(p: f64) -> f64 {
    let p = p.clamp(1e-15, 1.0 - 1e-15);
    let a: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    let b: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    let c: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    let d: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;
    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}
