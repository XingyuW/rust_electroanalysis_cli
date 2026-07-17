use crate::{
    domain::MultiChannelMeasurement, estimation::error::EstimationError,
    potentiometry::units::Quantity, results::FeatureComparability,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuxiliaryObservationKind {
    KnownActivityStandard,
    ReferenceMeasurement,
    CalibrationCheck,
    EisConditionProxy,
    TransientResponseParameter,
    ReferenceElectrodeControl,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuxiliaryObservation {
    pub timestamp_s: f64,
    pub observation_type: AuxiliaryObservationKind,
    pub value: f64,
    pub variance: Option<f64>,
    #[serde(default)]
    pub variance_unit: Option<String>,
    pub unit: String,
    pub source: String,
    pub comparability: FeatureComparability,
}

#[derive(Debug, Clone, Copy)]
pub struct MeasurementObservation {
    pub timestamp_s: f64,
    pub potential_v: Option<f64>,
    pub observation_variance_v2: Option<f64>,
}

pub fn observations(
    measurement: &MultiChannelMeasurement,
    channel: &str,
) -> Result<Vec<MeasurementObservation>, EstimationError> {
    let c = measurement.channel(channel).ok_or_else(|| {
        EstimationError::invalid(format!("selected channel '{channel}' does not exist"))
    })?;
    let unit = c.unit.parse().map_err(|error| {
        EstimationError::invalid(format!(
            "selected channel '{}' has unsupported unit '{}': {error}",
            c.name, c.unit
        ))
    })?;
    let scale = Quantity::new(1.0, unit)
        .and_then(|quantity| quantity.to_potential_v())
        .map_err(|error| {
            EstimationError::invalid(format!(
                "selected channel '{}' is not a potential channel: {error}",
                c.name
            ))
        })?;
    if measurement.time.iter().any(|t| !t.is_finite()) {
        return Err(EstimationError::invalid(
            "measurement contains a nonfinite timestamp",
        ));
    }
    for pair in measurement.time.windows(2) {
        if pair[1].partial_cmp(&pair[0]) != Some(std::cmp::Ordering::Greater) {
            return Err(EstimationError::invalid(
                "timestamps must be strictly increasing; duplicate or non-monotonic timestamps were not resolved",
            ));
        }
    }
    Ok(measurement
        .time
        .iter()
        .copied()
        .zip(c.values.iter().copied())
        .zip(
            c.variance
                .as_deref()
                .unwrap_or(&[])
                .iter()
                .copied()
                .chain(std::iter::repeat(None)),
        )
        .map(
            |((timestamp_s, potential), variance)| MeasurementObservation {
                timestamp_s,
                potential_v: potential.map(|value| value * scale),
                observation_variance_v2: variance.map(|value| value * scale * scale),
            },
        )
        .collect())
}
