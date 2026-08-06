# Phase 06 Implementation Plan — Model Workflows

| File | Change |
|---|---|
| `src/cli.rs`, `src/main.rs` | Add `model validate`, `simulate`, `decompose`, and `report` command dispatch. |
| `src/runners/model.rs` | Compile/evaluate model definitions and write durable JSON/CSV/TXT artifacts. |
| `src/results/model.rs` | Add finite-only model-analysis artifact schema. |
| `src/plottings/model_plot.rs` | Generate deterministic model-analysis plots. |
| `src/workspace.rs`, `config/model.toml` | Register and provide workspace model configuration. |
| `tests/phase06_model_workflow.rs` | Cover parsing, validation, simulation, decomposition, schema, output, and estimate compatibility. |

The workflow is an outer adapter. `src/model/` remains independent of runners,
CLI, plotting, health, mechanism, and estimation.
