//! Small deterministic uncertainty helpers shared by timescale calculations.

pub fn delta_variance(gradient: &[f64], covariance: Option<&[Vec<f64>]>) -> Option<f64> {
    let covariance = covariance?;
    if covariance.len() != gradient.len()
        || covariance.iter().any(|row| row.len() != gradient.len())
    {
        return None;
    }
    let mut variance = 0.0;
    for i in 0..gradient.len() {
        for j in 0..gradient.len() {
            variance += gradient[i] * covariance[i][j] * gradient[j];
        }
    }
    (variance.is_finite() && variance >= 0.0).then_some(variance)
}

pub fn confidence_interval(
    value: f64,
    standard_error: Option<f64>,
    level: f64,
) -> Option<(f64, f64)> {
    let se = standard_error?.abs();
    if !value.is_finite() || !se.is_finite() || !(0.0 < level && level < 1.0) {
        return None;
    }
    let z = if level >= 0.99 {
        2.576
    } else if level >= 0.95 {
        1.96
    } else if level >= 0.90 {
        1.645
    } else {
        1.0
    };
    Some(((value - z * se).max(f64::MIN_POSITIVE), value + z * se))
}
