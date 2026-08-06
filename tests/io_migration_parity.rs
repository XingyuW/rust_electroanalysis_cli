//! Independent migration parity checks.
//!
//! The legacy side is `tests/legacy_snapshot`, copied from cc6f283. It does
//! not depend on or register `electrodata-io` formats.

#[path = "legacy_snapshot/mod.rs"]
mod legacy_snapshot;

use legacy_snapshot::{Dataset as LegacyDataset, Eis as LegacyEis, TimeSeries as LegacyTimeSeries};
use rust_electroanalysis_cli::data_file::{project_compatibility_read_options, read_dataset};
use std::path::{Path, PathBuf};

const EXACT_TIME_FIXTURES: &[&str] = &[
    "regular_two_column.csv",
    "regular_multichannel.csv",
    "regular_missing_cells.csv",
    "regular_malformed_timestamp.csv",
    "regular_invalid_numeric.csv",
    "regular_ragged_rows.csv",
    "chi_ocpt.csv",
    "chi_multichannel_ocpt.csv",
];

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/io_migration")
        .join(name)
}

fn unit_name(unit: Option<&electrodata_io::Unit>) -> String {
    match unit {
        Some(electrodata_io::Unit::Second) => "s".to_string(),
        Some(electrodata_io::Unit::Volt) => "V".to_string(),
        Some(electrodata_io::Unit::Ampere) => "A".to_string(),
        Some(electrodata_io::Unit::Hertz) => "Hz".to_string(),
        Some(electrodata_io::Unit::Ohm) => "ohm".to_string(),
        Some(electrodata_io::Unit::Degree) => "deg".to_string(),
        Some(electrodata_io::Unit::Other(value)) => value.clone(),
        _ => String::new(),
    }
}

fn assert_complete_time_series_parity(path: &Path, legacy: &LegacyTimeSeries) {
    let canonical = read_dataset(path).expect("canonical reader must accept fixture");
    let view = canonical.time_series_view().expect("time-series view");
    assert_eq!(
        view.time.values.len(),
        legacy.raw_time.len(),
        "row count: {}",
        path.display()
    );
    assert_eq!(
        view.time.values,
        legacy
            .raw_time
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<_>>(),
        "raw timestamps: {}",
        path.display()
    );
    assert_eq!(
        view.time_seconds.as_ref(),
        Some(
            &legacy
                .raw_time
                .iter()
                .copied()
                .map(Some)
                .collect::<Vec<_>>()
        ),
        "time seconds: {}",
        path.display()
    );
    assert_eq!(
        unit_name(view.time.unit.as_ref()),
        "s",
        "time source unit: {}",
        path.display()
    );
    assert_eq!(
        view.measurements.len(),
        legacy.channels.len(),
        "channel count: {}",
        path.display()
    );

    for (canonical_channel, legacy_channel) in view.measurements.iter().zip(&legacy.channels) {
        // cc6f283 removed the unit from its domain channel name. The canonical
        // reader intentionally retains the complete source header instead.
        let expected_source_name = if legacy_channel.unit.is_empty() {
            legacy_channel.name.clone()
        } else {
            format!("{}/{}", legacy_channel.name, legacy_channel.unit)
        };
        assert_eq!(
            canonical_channel.original_name.as_deref(),
            Some(expected_source_name.as_str()),
            "source channel name: {}",
            path.display()
        );
        if !legacy_channel.unit.is_empty() {
            assert_ne!(
                canonical_channel.original_name.as_deref(),
                Some(legacy_channel.name.as_str()),
                "canonical source name must not reproduce the legacy unit-stripped name: {}",
                path.display()
            );
        }
        assert_eq!(
            unit_name(canonical_channel.unit.as_ref()),
            legacy_channel.unit,
            "channel unit: {}",
            path.display()
        );
        assert_eq!(
            canonical_channel.values,
            legacy_channel.values,
            "all optional values/null positions: {}",
            path.display()
        );
    }
}

#[test]
fn archived_legacy_and_canonical_time_series_match_all_numeric_domain_values() {
    for name in EXACT_TIME_FIXTURES {
        let path = fixture(name);
        let LegacyDataset::TimeSeries(legacy) =
            legacy_snapshot::read(&path).expect("archived legacy reader")
        else {
            panic!("expected legacy time series: {name}");
        };
        assert_complete_time_series_parity(&path, &legacy);
    }
}

fn assert_complete_eis_parity(path: &Path, legacy: &LegacyEis) {
    let canonical = read_dataset(path).expect("canonical reader must accept EIS");
    let view = canonical.eis_view().expect("EIS view");
    assert_eq!(
        view.frequency.values,
        legacy
            .frequency
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<_>>(),
        "frequency: {}",
        path.display()
    );
    assert_eq!(
        view.real.values,
        legacy.real.iter().copied().map(Some).collect::<Vec<_>>(),
        "real impedance: {}",
        path.display()
    );
    assert_eq!(
        view.imaginary.values,
        legacy
            .imaginary
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<_>>(),
        "imaginary impedance/sign: {}",
        path.display()
    );
    assert_eq!(
        view.phase_values(),
        legacy.phase.iter().copied().map(Some).collect::<Vec<_>>(),
        "derived phase: {}",
        path.display()
    );
    assert_eq!(
        view.magnitude_values(),
        legacy.measured_magnitude.as_ref().map_or_else(
            || legacy
                .real
                .iter()
                .zip(&legacy.imaginary)
                .map(|(real, imaginary)| Some(real.hypot(*imaginary)))
                .collect::<Vec<_>>(),
            |values| values.iter().copied().map(Some).collect::<Vec<_>>()
        ),
        "derived magnitude: {}",
        path.display()
    );
    assert_eq!(
        view.measured_magnitude.map(|column| column.values.clone()),
        legacy
            .measured_magnitude
            .as_ref()
            .map(|values| values.iter().copied().map(Some).collect()),
        "source measured magnitude: {}",
        path.display()
    );
    assert_eq!(
        view.measured_phase.map(|column| column.values.clone()),
        legacy
            .measured_phase
            .as_ref()
            .map(|values| values.iter().copied().map(Some).collect()),
        "source measured phase: {}",
        path.display()
    );
    assert_eq!(
        view.frequency.values.len(),
        legacy.frequency.len(),
        "row count: {}",
        path.display()
    );
    assert_eq!(
        unit_name(view.frequency.unit.as_ref()),
        "Hz",
        "frequency unit: {}",
        path.display()
    );
    assert_eq!(
        unit_name(view.real.unit.as_ref()),
        "ohm",
        "real unit: {}",
        path.display()
    );
    assert_eq!(
        unit_name(view.imaginary.unit.as_ref()),
        "ohm",
        "imaginary unit: {}",
        path.display()
    );
}

#[test]
fn archived_legacy_and_canonical_eis_compare_every_required_field() {
    for name in ["chi_eis_four_column.csv", "chi_eis_five_column.csv"] {
        let path = fixture(name);
        let LegacyDataset::Eis(legacy) =
            legacy_snapshot::read(&path).expect("archived legacy reader")
        else {
            panic!("expected legacy EIS: {name}");
        };
        assert_complete_eis_parity(&path, &legacy);
    }
}

#[test]
fn intentional_canonical_improvements_are_explicitly_classified() {
    for name in [
        "regular_headerless.csv",
        "generic_text.dat",
        "chi_eis_three_column.csv",
        "chi_eis_reordered_columns.csv",
    ] {
        let path = fixture(name);
        assert!(
            legacy_snapshot::read(&path).is_err(),
            "cc6f283 unexpectedly accepted {name}"
        );
        read_dataset(&path).expect("canonical reader must accept intentional improvement");
    }
}

#[test]
fn archived_legacy_xlsx_reader_matches_simple_and_historical_preamble_workbooks() {
    for path in [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xlsx/single_timeseries.xlsx"),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/xlsx/historical_preamble_timeseries.xlsx"),
    ] {
        let LegacyDataset::TimeSeries(legacy) =
            legacy_snapshot::read(&path).expect("archived xlsx reader")
        else {
            panic!("expected legacy xlsx time series: {}", path.display());
        };
        assert_complete_time_series_parity(&path, &legacy);
    }
}

#[test]
fn project_compatibility_policy_is_part_of_the_parity_contract() {
    let options = project_compatibility_read_options();
    assert_eq!(options.profile, electrodata_io::ReadProfile::Compatibility);
}
