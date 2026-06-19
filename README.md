# rust_plots (CLI-only)

`rust_plots` is a CLI application for electrochemical data parsing, plotting, fitting, and equivalent-circuit search.

## Workspace layout

The CLI now uses TOML configuration under `workspace/config/`:

```text
workspace/
├── config/
│   ├── app.toml
│   ├── plotting.toml
│   ├── analysis.toml
│   └── parsing.toml
├── data/
├── output/
└── logs/
```

- `config/app.toml`: app-level state (schema version, logging, last run).
- `config/plotting.toml`: plotting workflow configuration.
- `config/analysis.toml`: ECM search/evolution configuration.
- `config/parsing.toml`: circuit model resolver configuration.

Legacy root config files (`plot_config.toml`, `ecm_search.toml`, `circuit_models.toml`) are migrated/fallback-compatible.

## CLI usage

```bash
cargo run -- --help
cargo run -- --plot all
cargo run -- --plot eis
cargo run -- --plot regular-plot
cargo run -- --plot generic
cargo run -- --search-eis data/
```

Optional overrides:

- `--plot-config <path>`: override plotting config file.
- `--search-config <path>`: override analysis config file.
- `--search-output <path>`: override report/export destination.
- `--search-top <n>`: override ranked candidate count.

## Validation commands

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```
