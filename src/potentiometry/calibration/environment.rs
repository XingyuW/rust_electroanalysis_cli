//! Time alignment of environmental series without mutating source data.

use super::error::CalibrationError;
use crate::domain::EnvironmentalSeries;
use crate::results::calibration::EnvironmentalAlignment;

#[derive(Debug, Clone, PartialEq)]
pub struct AlignedEnvironmentalValue {
    pub value: f64,
    pub source_series: String,
    pub alignment: EnvironmentalAlignment,
    pub source_timestamps: Vec<f64>,
    pub interpolated: bool,
    pub time_gap_s: f64,
}

pub fn align_environmental_series(
    series: &EnvironmentalSeries,
    timestamp: f64,
    alignment: EnvironmentalAlignment,
    maximum_gap_s: f64,
    window_half_width_s: f64,
) -> Result<AlignedEnvironmentalValue, CalibrationError> {
    if !timestamp.is_finite() || !maximum_gap_s.is_finite() || maximum_gap_s <= 0.0 {
        return Err(CalibrationError::InvalidObservation(
            "environmental alignment received an invalid timestamp or gap".to_string(),
        ));
    }
    let mut pairs = series
        .time
        .iter()
        .copied()
        .zip(series.values.iter().copied())
        .filter_map(|(time, value)| {
            (time.is_finite() && value.is_some_and(f64::is_finite)).then_some((time, value?))
        })
        .collect::<Vec<_>>();
    pairs.sort_by(|left, right| left.0.total_cmp(&right.0));
    if pairs.is_empty() {
        return Err(CalibrationError::InvalidObservation(format!(
            "environmental series '{}' has no finite values",
            series.name
        )));
    }
    let nearest = pairs
        .iter()
        .min_by(|left, right| {
            (left.0 - timestamp)
                .abs()
                .total_cmp(&(right.0 - timestamp).abs())
        })
        .copied();
    let nearest_gap = nearest
        .map(|(time, _)| (time - timestamp).abs())
        .unwrap_or(f64::INFINITY);
    if nearest_gap > maximum_gap_s {
        return Err(CalibrationError::InvalidObservation(format!(
            "environmental series '{}' has no value within {:.6} s",
            series.name, maximum_gap_s
        )));
    }

    let (value, source_timestamps, interpolated) = match alignment {
        EnvironmentalAlignment::Nearest => {
            let (time, value) = nearest.expect("pairs was checked nonempty");
            (value, vec![time], false)
        }
        EnvironmentalAlignment::LinearInterpolation => {
            if let Some((left, right)) = pairs.windows(2).find_map(|window| {
                (window[0].0 <= timestamp && timestamp <= window[1].0)
                    .then_some((window[0], window[1]))
            }) {
                let span = right.0 - left.0;
                if span <= 0.0 {
                    (left.1, vec![left.0], false)
                } else {
                    let fraction = (timestamp - left.0) / span;
                    (
                        left.1 + fraction * (right.1 - left.1),
                        vec![left.0, right.0],
                        true,
                    )
                }
            } else {
                let (time, value) = nearest.expect("pairs was checked nonempty");
                (value, vec![time], false)
            }
        }
        EnvironmentalAlignment::WindowMean | EnvironmentalAlignment::WindowMedian => {
            let half_width = window_half_width_s.max(0.0);
            let mut values = pairs
                .iter()
                .filter(|(time, _)| (*time - timestamp).abs() <= half_width)
                .map(|(_, value)| *value)
                .collect::<Vec<_>>();
            if values.is_empty() {
                return Err(CalibrationError::InvalidObservation(format!(
                    "environmental window for '{}' is empty",
                    series.name
                )));
            }
            let value = if alignment == EnvironmentalAlignment::WindowMean {
                values.iter().sum::<f64>() / values.len() as f64
            } else {
                values.sort_by(f64::total_cmp);
                if values.len() % 2 == 0 {
                    (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
                } else {
                    values[values.len() / 2]
                }
            };
            (value, vec![timestamp], false)
        }
    };
    Ok(AlignedEnvironmentalValue {
        value,
        source_series: series.name.clone(),
        alignment,
        source_timestamps,
        interpolated,
        time_gap_s: nearest_gap,
    })
}

#[cfg(test)]
mod tests {
    use super::align_environmental_series;
    use crate::domain::EnvironmentalSeries;
    use crate::results::calibration::EnvironmentalAlignment;

    #[test]
    fn linearly_interpolates_environmental_values_with_gap_guard() {
        let series = EnvironmentalSeries {
            name: "temperature".to_string(),
            unit: "C".to_string(),
            time: vec![0.0, 10.0],
            values: vec![Some(20.0), Some(30.0)],
            metadata: None,
        };
        let aligned = align_environmental_series(
            &series,
            5.0,
            EnvironmentalAlignment::LinearInterpolation,
            6.0,
            1.0,
        )
        .unwrap();
        assert_eq!(aligned.value, 25.0);
        assert!(aligned.interpolated);
    }
}
