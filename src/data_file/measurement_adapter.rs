//! Adapters from scientific measurements into the existing PlotData type.

use crate::data_file::data_op::PlotData;
use crate::domain::{MeasurementChannel, MultiChannelMeasurement};

/// Convert every measurement channel into an independent PlotData series.
/// Missing channel values are omitted only in this plotting projection; the
/// source measurement retains them as `None` and its diagnostics remain intact.
pub fn measurement_to_plot_data(measurement: &MultiChannelMeasurement) -> Vec<PlotData> {
    measurement
        .channels
        .iter()
        .map(|channel| channel_to_plot_data(&measurement.time, channel))
        .collect()
}

/// Alias with an imperative name for callers building plotting datasets.
pub fn to_plot_data(measurement: &MultiChannelMeasurement) -> Vec<PlotData> {
    measurement_to_plot_data(measurement)
}

pub fn channel_to_plot_data(time: &[f64], channel: &MeasurementChannel) -> PlotData {
    let (x_values, y_values): (Vec<_>, Vec<_>) = time
        .iter()
        .copied()
        .zip(channel.values.iter().copied())
        .filter_map(|(time, value)| value.map(|value| (time, value)))
        .unzip();

    let label = if channel.unit.trim().is_empty() {
        channel.name.clone()
    } else {
        format!("{} [{}]", channel.name, channel.unit)
    };

    PlotData::new(x_values, y_values).with_label(label)
}

#[cfg(test)]
mod tests {
    use super::measurement_to_plot_data;
    use crate::domain::{MeasurementChannel, MultiChannelMeasurement};

    #[test]
    fn converts_channels_to_plot_data_without_touching_missing_values() {
        let measurement = MultiChannelMeasurement::new(
            vec![0.0, 1.0, 2.0],
            vec![MeasurementChannel::new(
                "potential",
                "V",
                vec![Some(0.1), None, Some(0.3)],
            )],
        )
        .expect("valid measurement");
        let plots = measurement_to_plot_data(&measurement);

        assert_eq!(plots[0].x_values, vec![0.0, 2.0]);
        assert_eq!(plots[0].y_values, vec![0.1, 0.3]);
        assert_eq!(measurement.missing_value_count(), 1);
    }
}
