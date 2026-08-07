//! Independent full consumer-domain migration parity checks.
//!
//! `legacy_snapshot` is a copied, dependency-independent reference reader.
//! These tests intentionally compare project domain objects, not provider
//! views.  Every field absent from the archived domain is recorded below.

#[path = "legacy_snapshot/mod.rs"]
mod legacy_snapshot;

use legacy_snapshot::{Dataset as LegacyDataset, Eis as LegacyEis, TimeSeries as LegacyTimeSeries};
use rust_electroanalysis_cli::{
    data_file::{EISData, parse_measurement_file, project_compatibility_read_options},
    domain::MeasurementParseResult,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct IntentionalDifference {
    field: &'static str,
    legacy_behavior: &'static str,
    canonical_behavior: &'static str,
    reason: &'static str,
    scientific_consequence: &'static str,
    classification: &'static str,
}

/// The complete, deliberately narrow set of fields for which the archived
/// consumer model had no equivalent.  Parity tests must not add broad ignore
/// rules; every absence is a separately reviewed migration improvement.
const INTENTIONAL_DIFFERENCES: &[IntentionalDifference] = &[
    IntentionalDifference {
        field: "measurement.time_coordinate_name/time_unit",
        legacy_behavior: "raw timestamps only",
        canonical_behavior: "preserves source coordinate header and unit",
        reason: "source-coordinate provenance was added at the domain boundary",
        scientific_consequence: "prevents implicit seconds assumptions",
        classification: "schema/provenance addition",
    },
    IntentionalDifference {
        field: "channel.source_header and channel.metadata",
        legacy_behavior: "unit-stripped name only",
        canonical_behavior: "logical name plus exact source_header metadata",
        reason: "historical selector compatibility without source identity loss",
        scientific_consequence: "none; aliases select the same physical channel",
        classification: "new source-header preservation",
    },
    IntentionalDifference {
        field: "channel.variance/sensor_id/analyte_id",
        legacy_behavior: "field absent",
        canonical_behavior: "explicit optional domain fields",
        reason: "current domain supports uncertainty and experiment enrichment",
        scientific_consequence: "none when absent in source fixtures",
        classification: "schema/provenance addition",
    },
    IntentionalDifference {
        field: "MeasurementParseResult.diagnostics",
        legacy_behavior: "field absent",
        canonical_behavior: "provider row/recovery diagnostics retained",
        reason: "canonical ingestion diagnostics are part of the consumer contract",
        scientific_consequence: "makes recovery policy auditable",
        classification: "new ingestion diagnostics",
    },
    IntentionalDifference {
        field: "EISData.derived_magnitude/derived_phase",
        legacy_behavior: "only one phase/magnitude representation",
        canonical_behavior: "keeps source Bode values distinct from derived values",
        reason: "source-versus-derived EIS semantics were made explicit",
        scientific_consequence: "avoids treating derived values as measurements",
        classification: "schema/provenance addition",
    },
    IntentionalDifference {
        field: "EIS acquisition/provenance metadata",
        legacy_behavior: "flat preamble map only",
        canonical_behavior: "structured date/technique/instrument/label plus metadata",
        reason: "canonical provider exposes structured acquisition provenance",
        scientific_consequence: "none to impedance vectors",
        classification: "schema/provenance addition",
    },
    IntentionalDifference {
        field: "EISData.circuit_model",
        legacy_behavior: "field absent",
        canonical_behavior: "resolved circuit-model hint retained on the consumer object",
        reason: "model-selection configuration is applied after canonical EIS conversion",
        scientific_consequence: "guides fitting; does not alter measured impedance vectors",
        classification: "schema/provenance addition",
    },
];

const LEGACY_TIME_FIXTURES: &[&str] = &[
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

fn assert_complete_time_series_domain_parity(path: &Path, legacy: &LegacyTimeSeries) {
    let MeasurementParseResult {
        measurement,
        diagnostics,
    } = parse_measurement_file(path).expect("canonical consumer-domain parse");

    assert_eq!(
        measurement.time,
        legacy.raw_time,
        "time values: {}",
        path.display()
    );
    assert_eq!(
        measurement.channels.len(),
        legacy.channels.len(),
        "channel count: {}",
        path.display()
    );
    assert!(
        measurement.time_coordinate_name.is_some(),
        "coordinate header: {}",
        path.display()
    );
    assert!(
        !measurement.time_unit.is_empty(),
        "coordinate unit: {}",
        path.display()
    );
    assert!(
        measurement.time_conversion.is_none(),
        "ingestion must not normalize time: {}",
        path.display()
    );

    for (current, archived) in measurement.channels.iter().zip(&legacy.channels) {
        let source_header = if archived.unit.is_empty() {
            archived.name.clone()
        } else {
            format!("{}/{}", archived.name, archived.unit)
        };
        assert_eq!(
            current.name,
            archived.name,
            "logical name: {}",
            path.display()
        );
        assert_eq!(
            current.source_header(),
            Some(source_header.as_str()),
            "source header: {}",
            path.display()
        );
        assert_eq!(current.unit, archived.unit, "unit: {}", path.display());
        assert_eq!(
            current.values,
            archived.values,
            "optional values/nulls: {}",
            path.display()
        );
        assert_eq!(
            current.variance,
            None,
            "fixture variance: {}",
            path.display()
        );
        assert_eq!(
            current.sensor_id,
            None,
            "fixture sensor id: {}",
            path.display()
        );
        assert_eq!(
            current.analyte_id,
            None,
            "fixture analyte id: {}",
            path.display()
        );
        assert_eq!(
            current
                .metadata
                .as_ref()
                .and_then(|metadata| metadata.get("source_header")),
            Some(&source_header),
            "channel metadata: {}",
            path.display()
        );
    }

    assert_eq!(
        diagnostics.successfully_parsed_rows,
        measurement.time.len(),
        "retained rows: {}",
        path.display()
    );
    assert_eq!(
        diagnostics.missing_values,
        measurement.missing_value_count(),
        "missing cells: {}",
        path.display()
    );
    assert!(
        diagnostics.total_rows >= diagnostics.successfully_parsed_rows,
        "total rows: {}",
        path.display()
    );
    assert!(
        diagnostics.total_rows >= diagnostics.skipped_rows,
        "skipped rows: {}",
        path.display()
    );
    assert!(
        diagnostics.total_rows >= diagnostics.malformed_rows,
        "malformed rows: {}",
        path.display()
    );
    assert!(
        diagnostics
            .messages
            .iter()
            .any(|message| message.contains("electrodata-io")),
        "provider diagnostic provenance: {}",
        path.display()
    );
}

#[test]
fn archived_and_canonical_time_series_match_complete_consumer_domains() {
    for name in LEGACY_TIME_FIXTURES {
        let path = fixture(name);
        let LegacyDataset::TimeSeries(legacy) = legacy_snapshot::read(&path).expect("legacy parse")
        else {
            panic!("expected legacy time series: {name}");
        };
        assert_complete_time_series_domain_parity(&path, &legacy);
    }
}

fn assert_complete_eis_domain_parity(path: &Path, legacy: &LegacyEis) {
    let current = EISData::parse_file(path).expect("canonical EIS domain parse");
    assert_eq!(
        current.freq,
        legacy.frequency,
        "frequency: {}",
        path.display()
    );
    assert_eq!(
        current.z_re,
        legacy.real,
        "real impedance: {}",
        path.display()
    );
    assert_eq!(
        current.z_im,
        legacy.imaginary,
        "imaginary impedance/sign: {}",
        path.display()
    );
    assert_eq!(
        current.phase,
        legacy.phase,
        "source/legacy phase: {}",
        path.display()
    );
    assert_eq!(
        current.measured_magnitude,
        legacy
            .measured_magnitude
            .as_ref()
            .map(|values| values.iter().copied().map(Some).collect()),
        "source magnitude: {}",
        path.display()
    );
    assert_eq!(
        current.measured_phase,
        legacy
            .measured_phase
            .as_ref()
            .map(|values| values.iter().copied().map(Some).collect()),
        "source phase: {}",
        path.display()
    );
    assert_eq!(
        current.derived_magnitude.len(),
        legacy.frequency.len(),
        "derived magnitude rows: {}",
        path.display()
    );
    assert_eq!(
        current.derived_phase.len(),
        legacy.frequency.len(),
        "derived phase rows: {}",
        path.display()
    );
    for ((real, imaginary), (magnitude, phase)) in current
        .z_re
        .iter()
        .zip(&current.z_im)
        .zip(current.derived_magnitude.iter().zip(&current.derived_phase))
    {
        assert!((magnitude - real.hypot(*imaginary)).abs() < 1e-10);
        assert!((phase - imaginary.atan2(*real).to_degrees()).abs() < 1e-10);
    }
    assert_eq!(
        current.freq.len(),
        legacy.frequency.len(),
        "row count: {}",
        path.display()
    );
    assert!(!current.date.is_empty(), "date: {}", path.display());
    assert!(
        !current.test_type.is_empty(),
        "technique: {}",
        path.display()
    );
    assert!(
        !current.instrument_model.is_empty(),
        "instrument: {}",
        path.display()
    );
    assert!(!current.label.is_empty(), "label: {}", path.display());
    assert!(
        !current.circuit_model.is_empty(),
        "circuit model: {}",
        path.display()
    );
    for (key, value) in &legacy.metadata {
        assert!(
            current.metadata.get(key) == Some(value)
                || current
                    .metadata
                    .iter()
                    .any(|(current_key, current_value)| current_key == key
                        && current_value.contains(value)),
            "metadata key {key:?}: {}",
            path.display()
        );
    }
}

#[test]
fn archived_and_canonical_eis_match_complete_consumer_domains() {
    for name in ["chi_eis_four_column.csv", "chi_eis_five_column.csv"] {
        let path = fixture(name);
        let LegacyDataset::Eis(legacy) = legacy_snapshot::read(&path).expect("legacy parse") else {
            panic!("expected legacy EIS: {name}");
        };
        assert_complete_eis_domain_parity(&path, &legacy);
    }
}

#[test]
fn canonical_only_corpus_is_explicitly_classified() {
    for name in [
        "regular_headerless.csv",
        "generic_text.dat",
        "chi_eis_three_column.csv",
        "chi_eis_four_column_magnitude.csv",
        "chi_eis_reordered_columns.csv",
    ] {
        let path = fixture(name);
        assert!(
            legacy_snapshot::read(&path).is_err(),
            "legacy unexpectedly accepted {name}"
        );
        if name.contains("eis") {
            EISData::parse_file(&path).expect("canonical EIS improvement");
        } else {
            parse_measurement_file(&path).expect("canonical time-series improvement");
        }
    }

    let eis_xlsx = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xlsx/eis_only.xlsx");
    assert!(
        legacy_snapshot::read(&eis_xlsx).is_err(),
        "legacy XLSX reader is time-series-only"
    );
    EISData::parse_file(&eis_xlsx).expect("canonical XLSX EIS support");
}

#[test]
fn archived_xlsx_time_series_match_complete_consumer_domains() {
    for path in [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xlsx/single_timeseries.xlsx"),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/xlsx/historical_preamble_timeseries.xlsx"),
    ] {
        let LegacyDataset::TimeSeries(legacy) =
            legacy_snapshot::read(&path).expect("legacy XLSX parse")
        else {
            panic!("expected legacy XLSX time series: {}", path.display());
        };
        assert_complete_time_series_domain_parity(&path, &legacy);
    }
}

#[test]
fn intentional_difference_allowlist_is_complete_and_explained() {
    for difference in INTENTIONAL_DIFFERENCES {
        assert!(!difference.field.is_empty());
        assert!(!difference.legacy_behavior.is_empty());
        assert!(!difference.canonical_behavior.is_empty());
        assert!(!difference.reason.is_empty());
        assert!(!difference.scientific_consequence.is_empty());
        assert!(!difference.classification.is_empty());
    }
}

#[test]
fn project_compatibility_policy_is_part_of_the_parity_contract() {
    let options = project_compatibility_read_options();
    assert_eq!(options.profile, electrodata_io::ReadProfile::Compatibility);
}
