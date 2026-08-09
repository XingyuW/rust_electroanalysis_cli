# A0 artifact-contract fixture manifest

These tracked fixtures are permanent compatibility evidence for the eight A0
repair-set artifact kinds. Schema-1 fixtures omit `artifact_kind`; schema-2
fixtures contain the correct `artifact_kind`.

The schema-1 payload shapes were checked against the historical result
definitions listed below and against the current reader's legacy
defaults/aliases. Representative values were produced by the existing
producer constructors in `mhi_t02f_producer_roundtrip` and committed as
immutable inputs.

| Artifact kind | Schema-1 fixture | Schema version | `artifact_kind` | Historical/source evidence | Expected public-reader result | Payload fields asserted |
|---|---|---:|---|---|---|---|
| `transient_analysis` | `schema1/transient_analysis.schema1.json` | 1 | missing | `bd88cc2:src/results/transient.rs`, `TransientAnalysisReport` | typed read succeeds | channel, event count, candidate fit count |
| `calibration_observations` | `schema1/calibration_observations.schema1.json` | 1 | missing | `97a6e8c:src/results/calibration.rs`, `CalibrationObservationSet` | typed read succeeds | analyte, observation count, potential |
| `calibration_model` | `schema1/calibration_model.schema1.json` | 1 | missing | `97a6e8c:src/results/calibration.rs`, `StoredCalibrationModel` | typed read succeeds | analyte, model parameter and slope |
| `calibration_analysis` | `schema1/calibration_analysis.schema1.json` | 1 | missing | `97a6e8c:src/results/calibration.rs`, `CalibrationAnalysisReport` | typed read succeeds | calibration id and analyte |
| `signal_analysis` | `schema1/signal_analysis.schema1.json` | 1 | missing | `8e9979d:src/results/signal.rs`, `SignalAnalysisReport` | typed read succeeds | channel, unit, timestamps |
| `mechanism_analysis` | `schema1/mechanism_analysis.schema1.json` | 1 | missing | `a009ed5:src/results/mechanism.rs`, `MechanismAnalysisReport` | typed read succeeds | analysis id and transient timescale |
| `health_assessment` | `schema1/health_assessment.schema1.json` | 1 | missing | `8e9979d:src/results/health.rs`, `SensorHealthAssessment` | typed read succeeds | experiment id, feature value |
| `health_trend` | `schema1/health_trend.schema1.json` | 1 | missing | `8e9979d:src/results/health.rs`, `HealthTrendReport` | typed read succeeds | analysis id and trend collection |

| Artifact kind | Schema-2 fixture | Schema version | `artifact_kind` | Expected public-reader result |
|---|---|---:|---|---|
| `transient_analysis` | `schema2/transient_analysis.schema2.json` | 2 | `transient_analysis` | typed read succeeds |
| `calibration_observations` | `schema2/calibration_observations.schema2.json` | 2 | `calibration_observations` | typed read succeeds |
| `calibration_model` | `schema2/calibration_model.schema2.json` | 2 | `calibration_model` | typed read succeeds |
| `calibration_analysis` | `schema2/calibration_analysis.schema2.json` | 2 | `calibration_analysis` | typed read succeeds |
| `signal_analysis` | `schema2/signal_analysis.schema2.json` | 2 | `signal_analysis` | typed read succeeds |
| `mechanism_analysis` | `schema2/mechanism_analysis.schema2.json` | 2 | `mechanism_analysis` | typed read succeeds |
| `health_assessment` | `schema2/health_assessment.schema2.json` | 2 | `health_assessment` | typed read succeeds |
| `health_trend` | `schema2/health_trend.schema2.json` | 2 | `health_trend` | typed read succeeds |

Schema-1 compatibility is tested by `mhi_t02d_legacy` and current JSON
acceptance by `mhi_t02a_current_correct_kind`, both in
`tests/a0_producer_roundtrip.rs`. Both call the public `read_artifact` path and
assert typed scientific payload fields.
