# Phase B mechanism evidence traceability

| Contract area | Implementation owner | Executable evidence |
|---|---|---|
| A1-neutral source preparation | `src/mechanism/preparation.rs` | `prepare_phase_b_evidence` |
| Structural binding and eligibility | `src/mechanism/evidence.rs` | `bind_hypothesis_evidence`, `evaluate_hypothesis_evidence_eligibility` |
| Temporal joins | `src/mechanism/temporal.rs` | `evaluate_temporal_join` |
| Scientific gates | `src/mechanism/{timescale,amplitude,repeatability,identifiability,validation}.rs` | gate evaluators |
| Promotion and components | `src/mechanism/promotion.rs` | `assess_hypothesis`, `assess_components` |
| Semantic history identities | `src/mechanism/history.rs` | JCS/SHA-256 functions |
| Schema-4 report and CLI route | `src/results/mechanism.rs`, `src/runners/mechanism.rs`, `src/cli.rs` | `mechanism compare --mechanism-evidence-config` |
