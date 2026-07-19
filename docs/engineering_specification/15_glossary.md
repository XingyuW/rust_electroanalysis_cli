# 15 — Glossary

**Identifier:** `DOC-15`  
**Status:** Verified from repository inspection  
**Last Updated:** 2026-07-19

---

## Software Engineering Terms

| Term | Definition |
|------|------------|
| **AST (Abstract Syntax Tree)** | A tree representation of a parsed expression. In this project, circuit strings like `R0-p(CPE1,R1)` are parsed into `CircuitNode` trees. |
| **CLI (Command-Line Interface)** | A text-based interface where users type commands. This project is entirely CLI-based. |
| **CI (Continuous Integration)** | Automated build and test runs triggered by code changes (GitHub Actions in this project). |
| **Crate** | A Rust package. This project is one crate named `rust_electroanalysis_cli`. |
| **DAG (Directed Acyclic Graph)** | A graph with directed edges and no cycles. Used to describe module dependencies. |
| **Enum** | A type that can be one of several named variants. Example: `ExperimentEventKind` can be `ConcentrationStep`, `FlowChange`, etc. |
| **Facade** | A simplified interface that hides complexity behind it. Example: `src/fitting/mod.rs` is a thin wrapper around `src/impedance/`. |
| **LTO (Link-Time Optimization)** | Compiler optimization applied when linking the final binary. Used in release builds. |
| **Mermaid** | A text-based diagramming language used throughout this specification. |
| **Module** | A Rust source file or directory that groups related code. |
| **Provenance** | A record of where data came from and how it was processed (SHA-256 hashes + timestamps in this project). |
| **Runner** | A module that orchestrates one complete workflow: parsing → analysis → reporting → output. |
| **Schema version** | A module-specific format version embedded in config/result artifacts to track evolution over time (for example, estimation config is currently v3 while several outputs remain v1 or v2). |
| **Serde** | The Rust serialization framework. Converts Rust structs to/from JSON, TOML, etc. |
| **Struct** | A composite data type with named fields. Example: `CircuitFitResult { fitted_parameters, parameter_names, ... }`. |
| **TOML** | A configuration file format (`[sections]` with `key = value`). All config files in this project use TOML. |
| **Trait** | A Rust interface defining shared behaviour. Example: `Impedance` trait requires `calculate()` and `param_count()`. |
| **thiserror** | A Rust library that simplifies creating custom error types with `#[derive(Error)]`. |
| **unwrap() / expect()** | Rust methods that extract a value from an `Option` or `Result`, but **panic** (crash) if there is no value. Used cautiously in this project. |

## Rust-Specific Terms

| Term | Definition |
|------|------------|
| **Cargo** | Rust's build system and package manager. |
| **clap** | The CLI argument-parsing library used in this project (version 4, derive mode). |
| **clippy** | Rust's official linter — catches common mistakes and style issues. |
| **rustfmt** | Rust's official code formatter. |
| **f64** | 64-bit floating-point number (IEEE 754 double precision). |
| **Option\<T\>** | A type that is either `Some(value)` or `None`. Used for missing/optional data. |
| **Result\<T, E\>** | A type that is either `Ok(value)` or `Err(error)`. Used for fallible operations. |
| **Vec\<T\>** | A growable array (list) of type `T`. |
| **PathBuf** | An owned, mutable file path (vs `&Path`, a borrowed reference). |

## Electrochemistry Terms

| Term | Definition |
|------|------------|
| **Activity (a)** | Effective concentration of an ion, accounting for non-ideal behaviour (a = γ·c). Dimensionless or in mol/L depending on convention. |
| **Allan Variance** | A statistical measure of signal stability over different averaging times; used to characterize sensor noise. |
| **Bode Plot** | A plot of log|Z| vs log(f) and phase vs log(f) for EIS data. |
| **Calibration Curve** | A plot of measured potential vs log₁₀(activity) used to determine sensor sensitivity (slope) and standard potential (intercept). |
| **CHI (CH Instruments)** | A manufacturer of potentiostats. Their data files have a recognizable header format. |
| **CPE (Constant Phase Element)** | A non-ideal capacitor with impedance Z = 1/(Q·(jω)^α), where α < 1 for real electrodes. |
| **EIS (Electrochemical Impedance Spectroscopy)** | A technique measuring impedance Z(ω) over a range of frequencies to characterize electrochemical systems. |
| **Equivalent Circuit Model (ECM)** | An electrical circuit (resistors, capacitors, CPEs, Warburg elements) whose impedance matches measured EIS data, providing physical interpretation. |
| **Gerischer Element** | An impedance element modeling coupled chemical reaction and diffusion. |
| **ISE (Ion-Selective Electrode)** | A sensor whose potential responds selectively to a specific ion concentration. |
| **Levenberg-Marquardt** | An algorithm for nonlinear least-squares optimization; used to fit circuit models to EIS data. |
| **Nernst Equation** | E = E⁰ + (RT/zF)·ln(a) — relates electrode potential to ion activity. |
| **Nicolsky-Eisenman Equation** | Extension of the Nernst equation accounting for interfering ions via selectivity coefficients. |
| **Nyquist Plot** | A plot of −Z″ vs Z′ (imaginary vs real impedance) for EIS data. |
| **OCPT (Open-Circuit Potential vs Time)** | A measurement of electrode potential over time at zero current. |
| **PSD (Power Spectral Density)** | Frequency-domain representation of signal power; used to identify noise sources. |
| **Warburg Element** | An impedance element representing semi-infinite linear diffusion: Z = σ(1−j)/√ω. |
| **Welch Method** | A PSD estimation method using windowed, overlapping FFT segments to reduce variance. |

## Project-Specific Terms

| Term | Definition |
|------|------------|
| **AnalysisProvenance** | A struct recording software version, input/config SHA-256 hashes, and generation timestamp for reproducibility. |
| **Circuit string** | A compact text representation of an equivalent circuit, e.g., `R0-p(CPE1,R1)` meaning R₀ in series with the parallel combination of CPE₁ and R₁. |
| **CommandSpec** | A normalized enum representing any valid CLI command, regardless of whether it came from structured subcommands or legacy flat flags. |
| **ExperimentMetadataDocument** | A TOML file describing an experiment (sensor info, events, environmental data) independently of measurement data. |
| **LastRunMode** | An enum recording which workflow was last executed, persisted in `config/app.toml`. |
| **Workspace** | The project root directory containing `config/`, `data/`, `output/`, and `logs/` subdirectories. |
| **EKF (Extended Kalman Filter)** | A nonlinear state estimator using first-order Taylor linearization of the process/measurement models. |
| **UKF (Unscented Kalman Filter)** | A nonlinear state estimator using deterministic sigma-point sampling, typically more accurate than EKF for strongly nonlinear systems. |
