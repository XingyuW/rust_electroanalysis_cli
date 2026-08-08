# Reduced-order ISM components V1

`reduced_ism_v1` is an activity-first, additive-potential model definition.
Its active terms are `E_equilibrium + E_dynamic_fast + E_dynamic_slow +
E_reference`; observation variance is retained separately in V². Optional
candidate transduction and typed environmental covariates are deliberately not
enabled by default.

The Nernst and Nicolsky--Eisenman components are adapters to the established
calibration equations. The Nernst form is `E = E0 + S log10(a)`; the
theoretical adapter obtains `S` from signed charge and temperature, while an
explicit empirical slope remains supported by the legacy adapter. Nicolsky--
Eisenman uses its established positive-selectivity effective-activity term.
Activities must already have passed the canonical IO boundary. Existing Ideal,
Davies, and Extended Debye--Huckel activity adapters remain available for
low-level concentration-to-activity conversion; an empirical conductivity
correction is not represented as a thermodynamic activity law.

The generic dynamic and candidate-transduction states use `dx/dt = -x/tau`.
Their event update is explicit: `x+ = x- + gain * u`. Dynamic V1 instances use
`u = delta_log10_activity`; candidate transduction may use
`transduction_drive`. Neither is inferred from the measured voltage. The fast
and slow labels only describe relative fitted timescales and remain
`Phenomenological`; the candidate transduction mode is `Hypothesized` and is
not assigned to solid-contact, double-layer, water-layer, transport, or
fouling mechanisms.

The reference state is constant (`db_ref/dt = 0`) and can represent reference,
junction, or unresolved baseline offset without diagnosing electrode failure.
The generic covariate term is `beta * (u - u_ref)` and must be explicitly
declared with a typed input and compatible coefficient unit. Observation noise
is a positive variance in V² and never contributes deterministic potential or
random samples.

Every component carries evidence requirements and identifiability metadata.
The two-mode construction warns about reversed or insufficiently separated
timescales; this is an identifiability warning, not a compilation failure.
Complete uncertainty requires caller-supplied runtime covariance and complete
analytical derivative coverage. Declared uncertainty is never substituted as
runtime covariance.

The model-core equilibrium recognizer requires dynamic derivative and magnitude,
equilibrium gap, elapsed-tau coverage, environmental stability, innovation,
residual autocorrelation, observability, valid component domain, and complete
uncertainty. It returns `Indeterminate` when required evidence is missing;
small measured-voltage slope alone cannot yield `Equilibrium`.

Deferred work: spatial transport, physical interface mechanisms, EIS mapping,
mechanism scoring, EKF/UKF integration, health rules, plotting, reporting, and
simulation-noise sampling.

## V1 scientific guardrails

Ion charge is a fixed, deterministic `DiscreteInteger`: it must be finite,
exactly integral, nonzero, and within `i32`; no rounded or fitted charge is
accepted. Applicability limits are explicit, source-provenanced inclusive
intervals rather than universal activity or temperature limits. Reports retain
physical validity separately from `InsideDomain`, `NearBoundary`,
`OutsideDomain`, or `DomainUnavailable`, with Warn or Reject enforcement.

The first-order transduction candidate is always `Hypothesized`. Equilibrium
evidence distinguishes `Present`, `NotApplicable`, and `Missing`; missing
disturbance evidence is indeterminate, never zero. Component-specific
identifiability requirements describe activity/transient excitation, duration
relative to tau, mode separation, reference anchoring, and independent
covariate/interferent variation. Linear covariates require compatible input,
reference, and V-per-input sensitivity units. Result serialization rejects any
nested nonfinite value instead of encoding it as JSON `null`.

When legacy metadata and typed constraints coexist, V1 normalizes and merges
both sources. An exact duplicate is evaluated once and records both origins;
competing constraints for the same subject return a typed conflict rather than
being silently intersected. Each violation uses its own Warn/Reject policy, so
a passing Reject constraint cannot turn a separate Warn violation into failure.
