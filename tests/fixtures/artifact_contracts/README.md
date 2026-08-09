# Preserved A0 compatibility fixtures

These tracked inputs preserve the pre-A0 compatibility contract for the two
non-repair artifacts whose current `artifact_kind` remains optional. The
missing-kind inputs are historical compatibility cases; the correct-kind and
wrong-kind inputs are fixed matrix cases derived from the same payload.

| Fixture | Kind/version | Kind state | Source | Public-reader result | Payload asserted |
|---|---|---|---|---|---|
| `eis_fit_schema2_missing_kind.json` | `eis_fit` / 2 | missing | Existing tracked A0 fixture | accepted | fit id, source and fitted impedance payload are deserialized |
| `eis_fit_schema2_correct_kind.json` | `eis_fit` / 2 | correct | Existing tracked fixture plus fixed contract header | accepted | same fit payload |
| `eis_fit_schema2_wrong_kind.json` | `eis_fit` / 2 | `signal_analysis` | Existing tracked fixture plus fixed wrong header | `IncompatibleKind` | rejection preserves typed contract |
| `health_baseline_schema2_missing_kind.json` | `health_baseline` / 2 | missing | Existing tracked A0 fixture | accepted | baseline id and baseline collections are deserialized |
| `health_baseline_schema2_correct_kind.json` | `health_baseline` / 2 | correct | Existing tracked fixture plus fixed contract header | accepted | same baseline payload |
| `health_baseline_schema2_wrong_kind.json` | `health_baseline` / 2 | `signal_analysis` | Existing tracked fixture plus fixed wrong header | `IncompatibleKind` | rejection preserves typed contract |

The preserved matrix is exercised by
`a0_ac_compat_01_preserves_eis_fit_and_health_baseline_matrices` in
`tests/artifact_contract.rs`. No test writes to this directory.

# A0 repair-set compatibility fixtures

The following permanent fixtures cover all eight A0 repair-set artifact kinds.
Schema-1 fixtures omit `artifact_kind`, as required by the legacy contract;
schema-2 fixtures contain the correct `artifact_kind`. The schema-1 payload
shapes were checked against the historical result definitions listed below and
against the current reader's legacy defaults/aliases. Representative values
were produced by the existing producer constructors in
`mhi_t02f_producer_roundtrip` and committed as immutable inputs.

| Artifact kind | Schema-1 fixture | Historical source evidence | Schema-2 fixture | Representative payload asserted |
|---|---|---|---|---|
| `transient_analysis` | `schema1/transient_analysis.schema1.json` | `bd88cc2:src/results/transient.rs` (`TransientAnalysisReport`) | `schema2/transient_analysis.schema2.json` | channel, event count, candidate fit count |
| `calibration_observations` | `schema1/calibration_observations.schema1.json` | `97a6e8c:src/results/calibration.rs` (`CalibrationObservationSet`) | `schema2/calibration_observations.schema2.json` | analyte, observation count, potential |
| `calibration_model` | `schema1/calibration_model.schema1.json` | `97a6e8c:src/results/calibration.rs` (`StoredCalibrationModel`) | `schema2/calibration_model.schema2.json` | analyte, model parameter and slope |
| `calibration_analysis` | `schema1/calibration_analysis.schema1.json` | `97a6e8c:src/results/calibration.rs` (`CalibrationAnalysisReport`) | `schema2/calibration_analysis.schema2.json` | calibration id and analyte |
| `signal_analysis` | `schema1/signal_analysis.schema1.json` | `8e9979d:src/results/signal.rs` (`SignalAnalysisReport`) | `schema2/signal_analysis.schema2.json` | channel, unit, timestamps |
| `mechanism_analysis` | `schema1/mechanism_analysis.schema1.json` | `a009ed5:src/results/mechanism.rs` (`MechanismAnalysisReport`) | `schema2/mechanism_analysis.schema2.json` | analysis id and transient timescale |
| `health_assessment` | `schema1/health_assessment.schema1.json` | `8e9979d:src/results/health.rs` (`SensorHealthAssessment`) | `schema2/health_assessment.schema2.json` | experiment id, feature value |
| `health_trend` | `schema1/health_trend.schema1.json` | `8e9979d:src/results/health.rs` (`HealthTrendReport`) | `schema2/health_trend.schema2.json` | analysis id and trend collection |

Schema-1 compatibility is tested by `mhi_t02d_legacy` in
`tests/a0_producer_roundtrip.rs`; current JSON acceptance is tested by
`mhi_t02a_current_correct_kind` in the same file. Both tests call the public
`read_artifact` path and assert typed scientific payload fields.
