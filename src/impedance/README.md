# impedance subsystem

`impedance` is the scientific equivalent-circuit modeling core.

## Files

- `elements.rs`: element equations, parameter names/units, constraints, and bounds.
- `circuits.rs`: parser and AST for circuit strings (series/parallel tree).
- `fitting.rs`: nonlinear least-squares fitting primitives and parameter transforms.
- `lib.rs`: façade exports and fit pipeline composition.
- `circuit_models.rs`: model selection rules/configuration and resolver logic.
- `ecm_candidate.rs`: genetic encoding/decoding and seeded candidate families.
- `ecm_evolution.rs`: evolutionary search loop and mutation/crossover operators.
- `ecm_scoring.rs`: scalar-residual RSS, Gaussian BIC, explicit legacy score,
  and weighted RMSE.
- `ecm_search.rs`: report assembly for ranked candidate outputs.
- `reporting.rs`: fitted-circuit composition summaries.
- `pinn_optimizer.rs`: PINN-based optimizer utilities used by advanced fitting paths.

## Runtime relationship

`search_runner.rs` and `chi_file.rs` call into this subsystem for:

- direct model fitting
- candidate ranking
- evolutionary topology search

All report outputs (`*_ecm_search.txt` and `*_ecm_search.csv`) derive from structures generated here.

ECM BIC assumes independent Gaussian real and imaginary residuals and uses two
scalar observations per complex frequency point. The former modulus-normalized
objective is named `legacy_penalized_score`; it is not a variance-calibrated
chi-square.

The direct fit façade returns the named `CircuitFitResult` from
`src/results/`. Its fields expose fitted parameters, parameter metadata, and
the fitted real/imaginary/magnitude/phase channels without positional tuple
access. Circuit parsing, fitting, and ECM search failures use
`FittingError`; report construction uses `ReportingError`.
