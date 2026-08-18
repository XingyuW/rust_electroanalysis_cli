# Phase C sensor-health evidence traceability

This implementation follows the integrated Phase C contract. The executable
coverage is maintained beside its public artifact and CLI routes; schema-3
legacy compatibility and schema-4 Phase C mode are explicitly separated.

| Requirement ID | Acceptance Criterion | Implementation Symbol | Production Execution Path | Result |
|---|---|---|---|---|
| PC-SCHEMA-01 | Legacy reports remain readable without Phase C data | `SensorHealthAssessment`, `read_artifact` | public reader | implemented |
| PC-SCHEMA-02..04 | Schema 4 requires a complete, non-null report | `validate_phase_c`, `validate_value` | reader/writer | implemented |
| PC-LSW-01 | No-config health assessment remains schema 3 | `assess_legacy`, `write_legacy_sensor_health_assessment_v3` | CLI → runner | implemented |
| PC-LSW-02 | Configured health assessment writes schema 4 | `assess_phase_c`, `write_artifact` | CLI → runner | implemented |
| PC-LSW-03..10 | Writer boundary, route selection, reader compatibility, and config-only flags are deterministic | `assess`, legacy writer, CLI normalization | CLI → reader/writer | implemented |
| PC-DQ-01..03 | Absence is not healthy and quality is visible | `evaluate_data_quality`, `evaluate_dimension` | Phase C evaluator | implemented |
| PC-HEALTH-01..09 | Nine typed dimensions preserve category/causal separation | `health::phase_c` | Phase C evaluator | implemented |
