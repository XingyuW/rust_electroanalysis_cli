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
