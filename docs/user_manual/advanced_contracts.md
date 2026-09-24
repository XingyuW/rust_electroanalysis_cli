# Advanced evidence and validation configuration contracts

[Master manual](../USER_MANUAL.md) · [Configuration guide](configuration.md) · [Workflows](workflows.md)

These are optional, explicit input contracts for Phase-B mechanism evidence, Phase-C health and Phase-E validation. They are separate from the permissive legacy TOMLs. The field tables below enumerate nested input types reachable from each contract. Type names identify tables/objects, not TOML keys. `Vec<T>` means an array of values/tables, `Option<T>` means an optional value, `String` is quoted text, `usize`/`u32`/`u64` are nonnegative integers, `f64` is a number and `bool` is true/false. Stable evidence IDs are strings.

**Default rule:** fields require explicit values unless the table says optional/default. An enum deriving a Rust default does not itself make a TOML field optional. Choose thresholds from your experimental design; copying a fixture does not calibrate them. Unknown fields are rejected by these closed contracts. Changing a domain, gate or threshold changes which scientific claim is evaluated.

## Choosing and validating thresholds

| Contract | Constraints and scientific consequences |
|---|---|
| Phase B | schema_version=1; algorithms exactly log_ratio_v1, signed_relative_error_v1, independent_ln_tau_sample_sd_v1, bound_inputs_v1. Temporal tolerance is finite and nonnegative; fractions in [0,1]. Hypothesis/requirement IDs must resolve and be unique. Log-distance and relative-error maxima are finite/nonnegative; amplitude floor is positive with a valid UCUM unit. Repeatability requires at least two requirement IDs and two independent families; SD of ln(tau) is nonnegative. Identifiability uses two bound requirement IDs and a positive threshold. Validation, when supplied, requires nonempty protocol/version and at least two acquisition families. Smaller distance/error thresholds make agreement harder; larger family minima demand more independent replication. |
| Phase C | schema_version and required groups are checked by the strict loader. LevelThreshold orders watch <= degraded <= critical. Fractions, sample counts, positive ranges and condition-number thresholds undergo semantic checks. Select an actual event and available baseline feature names. Thresholds set evidence severity, not a diagnosis. See accepted-but-inactive fields in the main configuration chapter before relying on alignment or causal-promotion knobs. |
| Phase E | schema_version=1; all fields explicit. IDs are canonical and referenced IDs must exist. Registration SHA-256 is lowercase hex. Target/claim domains constrain eligible records. Endpoint minima and stratum minima are positive; development is not scoreable. Rule IDs, selector IDs and strata are sorted/unique where required. Rates are [0,1]; temperature bands are finite positive lower < upper and nonoverlapping. Mechanism needs a support_fraction lower-bound rule; health needs coverage, sensitivity and specificity lower-bound rules. Balanced accuracy has point_estimate only. Physical requests additionally need blinded quantified references, independent support and approval; an unprovisioned trust store prevents approval. |

Use [Phase-B fixtures](../../tests/fixtures/phase_b/), [Phase-C example](../../tests/fixtures/phase_c/config/valid_phase_c.toml) and [software Phase-E protocol](../../tests/fixtures/phase_e/protocol/software_valid.toml) as complete syntax examples. They are software/conformance examples, not approved physical protocols.

## MechanismEvidenceConfig

### MechanismEvidenceConfig

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `schema_version` | `u32` | Explicit; no implicit value | Contract version; use the version accepted by this reader. |
| `timescale` | `TimescaleEvidenceConfig` | Explicit; no implicit value | Timescale. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `amplitude` | `AmplitudeEvidenceConfig` | Explicit; no implicit value | Amplitude. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `repeatability` | `RepeatabilityEvidenceConfig` | Explicit; no implicit value | Repeatability. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `temporal` | `crate::mechanism::temporal::TemporalJoinConfig` | Explicit; no implicit value | Temporal. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `identifiability` | `IdentifiabilityGateConfig` | Explicit; no implicit value | Identifiability. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `promotion` | `HypothesisPromotionConfig` | Explicit; no implicit value | Promotion. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `validation` | `Option<crate::mechanism::validation::ValidationProtocol>` | Optional / absent | Validation. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `hypotheses` | `Vec<MechanismHypothesisDefinition>` | Explicit; no implicit value | Hypotheses. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### TimescaleEvidenceConfig

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `algorithm` | `String` | Explicit; no implicit value | Closed algorithm identifier listed above. |

### AmplitudeEvidenceConfig

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `algorithm` | `String` | Explicit; no implicit value | Closed algorithm identifier listed above. |

### RepeatabilityEvidenceConfig

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `algorithm` | `String` | Explicit; no implicit value | Closed algorithm identifier listed above. |

### TemporalJoinConfig

Source: [`src/mechanism/temporal.rs`](../../src/mechanism/temporal.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `point_tolerance_s` | `f64` | Explicit; no implicit value | Point tolerance s. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `window_overlap_rule` | `WindowOverlapRule` | Explicit; no implicit value | Explicit window overlap rule; allowed alternatives are enumerated below. |
| `event_identity_rule` | `EventIdentityRule` | Explicit; no implicit value | Explicit event identity rule; allowed alternatives are enumerated below. |
| `minimum_classified_fraction` | `f64` | Explicit; no implicit value | Lower support/acceptance bound for classified fraction. Change to match the required evidence strength. |
| `minimum_equilibrium_fraction` | `f64` | Explicit; no implicit value | Lower support/acceptance bound for equilibrium fraction. Change to match the required evidence strength. |
| `clock_mismatch_behavior` | `ClockMismatchBehavior` | Explicit; no implicit value | Clock mismatch behavior. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `scope_mismatch_behavior` | `ScopeMismatchBehavior` | Explicit; no implicit value | Scope mismatch behavior. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `mixed_state_policy` | `MixedStatePolicy` | Explicit; no implicit value | Explicit mixed state policy; allowed alternatives are enumerated below. |

### IdentifiabilityGateConfig

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `algorithm` | `String` | Explicit; no implicit value | Closed algorithm identifier listed above. |

### HypothesisPromotionConfig

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `minimum_independent_support` | `usize` | Explicit; no implicit value | Minimum independently supported evidence for promotion. |

### ValidationProtocol

Source: [`src/mechanism/validation.rs`](../../src/mechanism/validation.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `protocol_id` | `String` | Explicit; no implicit value | Stable protocol id; must match the corresponding declaration/reference. |
| `version` | `String` | Explicit; no implicit value | Version. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `minimum_acquisition_families` | `usize` | Explicit; no implicit value | Lower support/acceptance bound for acquisition families. Change to match the required evidence strength. |
| `required_conditions` | `Vec<ValidationCondition>` | Explicit; no implicit value | Required conditions. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### MechanismHypothesisDefinition

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `hypothesis_id` | `MechanismHypothesisId` | Explicit; no implicit value | Stable hypothesis id; must match the corresponding declaration/reference. |
| `display_name` | `String` | Explicit; no implicit value | Display name. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `target_components` | `Vec<String>` | Explicit; no implicit value | Target components. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `evidence_requirements` | `Vec<EvidenceRequirementBinding>` | Explicit; no implicit value | Evidence requirements. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `pair_requirements` | `Vec<EvidencePairRequirement>` | Explicit; no implicit value | Pair requirements. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `critical_requirement_ids` | `Vec<EvidenceRequirementId>` | Explicit; no implicit value | Requirements whose failure blocks the intended promotion. |
| `timescale_gate` | `Option<TimescaleGate>` | Optional / absent | Timescale gate. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `amplitude_gates` | `Vec<AmplitudeGate>` | Explicit; no implicit value | Amplitude gates. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `repeatability_gates` | `Vec<RepeatabilityGate>` | Explicit; no implicit value | Repeatability gates. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `identifiability_bindings` | `Vec<IdentifiabilityBinding>` | Explicit; no implicit value | Identifiability bindings. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `validation_applicability` | `ValidationApplicability` | Explicit; no implicit value | Validation applicability. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `role_bindings` | `Vec<MechanismEvidenceRoleBinding>` | Explicit; no implicit value | Role bindings. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### WindowOverlapRule

Source: [`src/mechanism/temporal.rs`](../../src/mechanism/temporal.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(transparent)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
enum WindowOverlapRule {
    PositiveDuration,
}
```

### EventIdentityRule

Source: [`src/mechanism/temporal.rs`](../../src/mechanism/temporal.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(transparent)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum EventIdentityRule {
    Exact,
}
```

### ClockMismatchBehavior

Source: [`src/mechanism/temporal.rs`](../../src/mechanism/temporal.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(transparent)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum ClockMismatchBehavior {
    Indeterminate,
}
```

### ScopeMismatchBehavior

Source: [`src/mechanism/temporal.rs`](../../src/mechanism/temporal.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(transparent)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum ScopeMismatchBehavior {
    Indeterminate,
}
```

### MixedStatePolicy

Source: [`src/mechanism/temporal.rs`](../../src/mechanism/temporal.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(transparent)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum MixedStatePolicy {
    RequireAllSteady {
        allow_quasi_equilibrium: bool,
    },
    MinimumSteadyFraction {
        minimum_fraction: f64,
        allow_quasi_equilibrium: bool,
        reject_if_disturbed: bool,
    },
    WorstCase,
}
```

### ValidationCondition

Source: [`src/mechanism/validation.rs`](../../src/mechanism/validation.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `condition_id` | `String` | Explicit; no implicit value | Stable condition id; must match the corresponding declaration/reference. |
| `requirement_ids` | `Vec<EvidenceRequirementId>` | Explicit; no implicit value | Stable requirement ids; must match the corresponding declaration/reference. |
| `experiment_scope` | `String` | Explicit; no implicit value | Experiment scope. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### EvidenceRequirementBinding

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable requirement id; must match the corresponding declaration/reference. |
| `target_selector` | `EvidenceTargetSelector` | Explicit; no implicit value | Target selector. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `source_class_selectors` | `Vec<PhaseBEvidenceSourceClass>` | Explicit; no implicit value | Source class selectors. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `source_field_path` | `String` | Explicit; no implicit value | Exact upstream evidence field to select; changing it changes the observed quantity. |
| `quantity_semantic` | `PhaseBQuantitySemantic` | Explicit; no implicit value | Quantity semantic. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `required_unit` | `String` | Explicit; no implicit value | UCUM unit required of the selected evidence; incompatible units prevent matching. |
| `expected_direction` | `RequiredEvidenceDirection` | Explicit; no implicit value | Candidate presence, support or contradiction expected from selected evidence. |
| `validity_requirement` | `EvidenceValidityRequirement` | Explicit; no implicit value | Validity requirement. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `gate` | `RequirementGate` | Explicit; no implicit value | Whether the requirement must be evaluated or is explicitly not applicable. |
| `stage` | `EvidenceRequirementStage` | Explicit; no implicit value | Use in support, validation, or both; does not establish independence. |

### EvidencePairRequirement

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable requirement id; must match the corresponding declaration/reference. |
| `left_requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable left requirement id; must match the corresponding declaration/reference. |
| `right_requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable right requirement id; must match the corresponding declaration/reference. |
| `temporal` | `TemporalRequirement` | Explicit; no implicit value | Temporal. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `gate` | `RequirementGate` | Explicit; no implicit value | Whether the requirement must be evaluated or is explicitly not applicable. |

### TimescaleGate

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `pair_requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable pair requirement id; must match the corresponding declaration/reference. |
| `maximum_log_distance` | `f64` | Explicit; no implicit value | Upper acceptance/severity bound for log distance. Raise only with a justified tolerance; changes evidence acceptance. |

### AmplitudeGate

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `predicted_requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable predicted requirement id; must match the corresponding declaration/reference. |
| `observed_requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable observed requirement id; must match the corresponding declaration/reference. |
| `expected_effect` | `ExpectedEffect` | Explicit; no implicit value | Expected effect. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `maximum_relative_error` | `f64` | Explicit; no implicit value | Upper acceptance/severity bound for relative error. Raise only with a justified tolerance; changes evidence acceptance. |
| `floor` | `AmplitudeThreshold` | Explicit; no implicit value | Floor. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### RepeatabilityGate

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `requirement_ids` | `Vec<EvidenceRequirementId>` | Explicit; no implicit value | Stable requirement ids; must match the corresponding declaration/reference. |
| `maximum_sample_standard_deviation_ln_tau` | `f64` | Explicit; no implicit value | Upper acceptance/severity bound for sample standard deviation ln tau. Raise only with a justified tolerance; changes evidence acceptance. |
| `minimum_independent_families` | `usize` | Explicit; no implicit value | Lower support/acceptance bound for independent families. Change to match the required evidence strength. |

### IdentifiabilityBinding

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `requirement_id` | `IdentifiabilityRequirementId` | Explicit; no implicit value | Stable requirement id; must match the corresponding declaration/reference. |
| `gate` | `RequirementGate` | Explicit; no implicit value | Whether the requirement must be evaluated or is explicitly not applicable. |
| `kind` | `IdentifiabilityKind` | Explicit; no implicit value | Kind. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `threshold` | `f64` | Explicit; no implicit value | Threshold. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `input` | `IdentifiabilityInputBinding` | Explicit; no implicit value | Input. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### ValidationApplicability

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
enum ValidationApplicability {
    #[default]
    NotApplicable,
    Required,
}
```

### MechanismEvidenceRoleBinding

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `hypothesis_id` | `MechanismHypothesisId` | Explicit; no implicit value | Stable hypothesis id; must match the corresponding declaration/reference. |
| `requirement_id` | `EvidenceRequirementId` | Explicit; no implicit value | Stable requirement id; must match the corresponding declaration/reference. |
| `evidence_id` | `crate::evidence::EvidenceId` | Explicit; no implicit value | Stable evidence id; must match the corresponding declaration/reference. |
| `role` | `MechanismEvidenceRole` | Explicit; no implicit value | Role. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### EvidenceTargetSelector

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum EvidenceTargetSelector {
    ExactComponent { value: String },
}
```

### PhaseBEvidenceSourceClass

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum PhaseBEvidenceSourceClass {
    Observed,
    ModelDerived,
    ProducerAssessment,
    ExternalReference,
}
```

### PhaseBQuantitySemantic

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum PhaseBQuantitySemantic {
    TimeConstant,
    Potential,
    Dimensionless,
    Other,
}
```

### RequiredEvidenceDirection

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum RequiredEvidenceDirection {
    CandidatePresence,
    Supports,
    Contradicts,
}
```

### EvidenceValidityRequirement

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum EvidenceValidityRequirement {
    Valid,
    ValidOrNotAssessed,
}
```

### RequirementGate

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum RequirementGate {
    #[default]
    Required,
    NotApplicable,
}
```

### EvidenceRequirementStage

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum EvidenceRequirementStage {
    #[default]
    Support,
    Validation,
    SupportAndValidation,
}
```

### TemporalRequirement

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum TemporalRequirement {
    NotApplicable,
    Required { join_mode: TemporalJoinMode },
}
```

### ExpectedEffect

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
enum ExpectedEffect {
    Increase,
    Decrease,
    SameSign,
}
```

### AmplitudeThreshold

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `value` | `f64` | Explicit; no implicit value | Value. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `unit` | `String` | Explicit; no implicit value | Unit. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### IdentifiabilityKind

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
enum IdentifiabilityKind {
    ModeSeparation,
}
```

### IdentifiabilityInputBinding

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `requirement_ids` | `Vec<EvidenceRequirementId>` | Explicit; no implicit value | Stable requirement ids; must match the corresponding declaration/reference. |
| `selection` | `IdentifiabilityInputSelection` | Explicit; no implicit value | Selection. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### MechanismEvidenceRole

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum MechanismEvidenceRole {
    Support,
    Validation,
    Calibration,
    Training,
}
```

### TemporalJoinMode

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
enum TemporalJoinMode {
    PointPoint,
    PointWindow,
    WindowPoint,
    WindowWindow,
    EventEvent,
}
```

### IdentifiabilityInputSelection

Source: [`src/mechanism/config.rs`](../../src/mechanism/config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum IdentifiabilityInputSelection {
    ExactPair {
        pair_requirement_id: EvidenceRequirementId,
    },
    AllEligible,
}
```

## PhaseCHealthEvidenceConfig

### PhaseCHealthEvidenceConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `schema_version` | `u32` | Explicit; no implicit value | Contract version; use the version accepted by this reader. |
| `maximum_reference_alignment_difference_s` | `f64` | Explicit; no implicit value | Upper acceptance/severity bound for reference alignment difference s. Raise only with a justified tolerance; changes evidence acceptance. |
| `data_quality` | `PhaseCDataQualityConfig` | Explicit; no implicit value | Data quality. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `signal_integrity` | `PhaseCSignalIntegrityConfig` | Explicit; no implicit value | Signal integrity. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `calibration_health` | `PhaseCCalibrationHealthConfig` | Explicit; no implicit value | Calibration health. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `dynamic_response_health` | `PhaseCDynamicResponseHealthConfig` | Explicit; no implicit value | Dynamic response health. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `environmental_robustness` | `PhaseCEnvironmentalRobustnessConfig` | Explicit; no implicit value | Environmental robustness. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `model_consistency` | `PhaseCModelConsistencyConfig` | Explicit; no implicit value | Model consistency. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `observability` | `PhaseCObservabilityConfig` | Explicit; no implicit value | Observability. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `uncertainty_health` | `PhaseCUncertaintyHealthConfig` | Explicit; no implicit value | Uncertainty health. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `causal_promotion` | `PhaseCCausalPromotionConfig` | Explicit; no implicit value | Causal promotion. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `phase_b_hypothesis_bindings` | `Vec<PhaseCHypothesisBinding>` | Empty list | Phase b hypothesis bindings. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### PhaseCDataQualityConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `minimum_finite_samples` | `usize` | Explicit; no implicit value | Lower support/acceptance bound for finite samples. Change to match the required evidence strength. |
| `maximum_missing_fraction` | `f64` | Explicit; no implicit value | Upper acceptance/severity bound for missing fraction. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_interval_cv` | `f64` | Explicit; no implicit value | Upper acceptance/severity bound for interval cv. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_duplicate_timestamps` | `usize` | Explicit; no implicit value | Upper acceptance/severity bound for duplicate timestamps. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_non_monotonic_timestamps` | `usize` | Explicit; no implicit value | Upper acceptance/severity bound for non monotonic timestamps. Raise only with a justified tolerance; changes evidence acceptance. |
| `allow_interpolation_gap_exceeded` | `bool` | Explicit; no implicit value | Allow interpolation gap exceeded. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### PhaseCSignalIntegrityConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `maximum_rms_noise_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for rms noise v. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_robust_noise_standard_deviation_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for robust noise standard deviation v. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_spike_fraction` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for spike fraction. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_absolute_drift_v_per_s` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for absolute drift v per s. Raise only with a justified tolerance; changes evidence acceptance. |

### PhaseCCalibrationHealthConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `maximum_absolute_slope_efficiency_error` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for absolute slope efficiency error. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_rmse_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for rmse v. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_absolute_prediction_bias_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for absolute prediction bias v. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_hysteresis_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for hysteresis v. Raise only with a justified tolerance; changes evidence acceptance. |

### PhaseCDynamicResponseHealthConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `selected_event_index` | `usize` | Explicit; no implicit value | Selected event index. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `baseline_tau_fast_feature` | `String` | Explicit; no implicit value | Baseline tau fast feature. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `baseline_tau_slow_feature` | `String` | Explicit; no implicit value | Baseline tau slow feature. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `baseline_time_to_90_percent_feature` | `String` | Explicit; no implicit value | Baseline time to 90 percent feature. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `baseline_response_amplitude_feature` | `String` | Explicit; no implicit value | Baseline response amplitude feature. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `maximum_tau_fast_ratio` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for tau fast ratio. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_tau_slow_ratio` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for tau slow ratio. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_time_to_90_percent_ratio` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for time to 90 percent ratio. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_response_amplitude_relative_loss` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for response amplitude relative loss. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_fit_rmse_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for fit rmse v. Raise only with a justified tolerance; changes evidence acceptance. |

### PhaseCEnvironmentalRobustnessConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `covariate` | `EnvironmentalCovariate` | Explicit; no implicit value | Covariate. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `minimum_points` | `usize` | Explicit; no implicit value | Lower support/acceptance bound for points. Change to match the required evidence strength. |
| `minimum_covariate_range` | `f64` | Explicit; no implicit value | Lower support/acceptance bound for covariate range. Change to match the required evidence strength. |
| `minimum_absolute_spearman_correlation` | `LevelThreshold` | Explicit; no implicit value | Lower support/acceptance bound for absolute spearman correlation. Change to match the required evidence strength. |
| `minimum_residual_rms_v` | `f64` | Explicit; no implicit value | Lower support/acceptance bound for residual rms v. Change to match the required evidence strength. |

### PhaseCModelConsistencyConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `maximum_residual_rms_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for residual rms v. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_residual_bias_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for residual bias v. Raise only with a justified tolerance; changes evidence acceptance. |

### PhaseCObservabilityConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `maximum_condition_number` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for condition number. Raise only with a justified tolerance; changes evidence acceptance. |
| `require_empirical_identifiability` | `bool` | Explicit; no implicit value | Require empirical identifiability. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### PhaseCUncertaintyHealthConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `maximum_partial_uncertainty_fraction` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for partial uncertainty fraction. Raise only with a justified tolerance; changes evidence acceptance. |
| `maximum_standard_error_v` | `LevelThreshold` | Explicit; no implicit value | Upper acceptance/severity bound for standard error v. Raise only with a justified tolerance; changes evidence acceptance. |

### PhaseCCausalPromotionConfig

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `minimum_independent_supporting_evidence` | `usize` | Explicit; no implicit value | Lower support/acceptance bound for independent supporting evidence. Change to match the required evidence strength. |

### PhaseCHypothesisBinding

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `hypothesis_id` | `MechanismHypothesisId` | Explicit; no implicit value | Stable hypothesis id; must match the corresponding declaration/reference. |
| `health_dimension` | `HealthDimension` | Explicit; no implicit value | Health dimension. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `relationship` | `PhaseCHypothesisRelationship` | Explicit; no implicit value | Declared Phase-B relationship; see inactive-field caveat. |

### LevelThreshold

Source: [`src/health_config.rs`](../../src/health_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `watch` | `f64` | Explicit; no implicit value | First severity boundary; ordered below degraded. |
| `degraded` | `f64` | Explicit; no implicit value | Middle severity boundary. |
| `critical` | `f64` | Explicit; no implicit value | Highest severity boundary. |

### EnvironmentalCovariate

Source: [`src/health_config.rs`](../../src/health_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum EnvironmentalCovariate {
    TemperatureK,
    ConductivitySPerM,
    IonicStrengthMolL,
    Flow,
}
```

### PhaseCHypothesisRelationship

Source: [`src/health_config.rs`](../../src/health_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum PhaseCHypothesisRelationship {
    PossiblePhysicalDegradation,
}
```

## MhiValidationProtocolV1

### MhiValidationProtocolV1

Source: [`src/mhi_validation/protocol.rs`](../../src/mhi_validation/protocol.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `schema_version` | `u32` | Explicit; no implicit value | Contract version; use the version accepted by this reader. |
| `protocol_id` | `String` | Explicit; no implicit value | Stable protocol id; must match the corresponding declaration/reference. |
| `title` | `String` | Explicit; no implicit value | Title. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `registration` | `ProtocolRegistrationV1` | Explicit; no implicit value | Registration. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `physical_approval_authority` | `PhysicalApprovalAuthorityV1` | Explicit; no implicit value | Explicit approval declaration; does not provision trust or authority. |
| `target_domain` | `crate::validation_config::DomainSelectorV1` | Explicit; no implicit value | Target domain. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `mechanism_endpoints` | `Vec<MechanismEndpointV1>` | Explicit; no implicit value | Mechanism endpoints. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `health_endpoints` | `Vec<HealthEndpointV1>` | Explicit; no implicit value | Health endpoints. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `statistics` | `StatisticsV1` | Explicit; no implicit value | Frozen interval, unavailable-rule and rule-composition contract. |
| `release_scope` | `Vec<ReleaseClaimV1>` | Explicit; no implicit value | Claims and their target domains/endpoints; evaluation cannot exceed this declaration. |

### ProtocolRegistrationV1

Source: [`src/mhi_validation/protocol.rs`](../../src/mhi_validation/protocol.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `registration_id` | `String` | Explicit; no implicit value | Stable registration id; must match the corresponding declaration/reference. |
| `immutable_reference_uri` | `String` | Explicit; no implicit value | Immutable reference uri. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `document_sha256` | `String` | Explicit; no implicit value | Document sha256. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### PhysicalApprovalAuthorityV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum PhysicalApprovalAuthorityV1 {
    NotRequested,
    EmbeddedTrustRoot { trust_root_id: String },
}
```

### DomainSelectorV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `analyte` | `CategoricalSelectorV1` | Explicit; no implicit value | Analyte. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `matrix` | `CategoricalSelectorV1` | Explicit; no implicit value | Matrix. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `sensor_design` | `CategoricalSelectorV1` | Explicit; no implicit value | Sensor design. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `sensor` | `CategoricalSelectorV1` | Explicit; no implicit value | Sensor. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `campaign` | `CategoricalSelectorV1` | Explicit; no implicit value | Campaign. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `temperature` | `TemperatureSelectorV1` | Explicit; no implicit value | Temperature. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### MechanismEndpointV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `endpoint_id` | `String` | Explicit; no implicit value | Stable endpoint id; must match the corresponding declaration/reference. |
| `hypothesis_id` | `String` | Explicit; no implicit value | Stable hypothesis id; must match the corresponding declaration/reference. |
| `cohort_role` | `CohortRoleV1` | Explicit; no implicit value | Validation or holdout cohort for scoring; development is unscoreable. |
| `domain` | `DomainSelectorV1` | Explicit; no implicit value | Domain. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `mechanism_artifact_required` | `bool` | Explicit; no implicit value | Mechanism artifact required. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `reference_rule` | `ReferenceAuthorityRuleV1` | Explicit; no implicit value | Explicit reference rule; allowed alternatives are enumerated below. |
| `support_levels` | `Vec<String>` | Explicit; no implicit value | Support levels. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `critical_policy` | `String` | Explicit; no implicit value | Explicit critical policy; allowed alternatives are enumerated below. |
| `minimum_eligible_records` | `u64` | Explicit; no implicit value | Lower support/acceptance bound for eligible records. Change to match the required evidence strength. |
| `minimum_independent_families` | `u64` | Explicit; no implicit value | Lower support/acceptance bound for independent families. Change to match the required evidence strength. |
| `required_strata` | `Vec<RequiredStratumV1>` | Explicit; no implicit value | Required strata. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `acceptance_rules` | `Vec<AcceptanceRuleV1>` | Explicit; no implicit value | Acceptance rules. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### HealthEndpointV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `endpoint_id` | `String` | Explicit; no implicit value | Stable endpoint id; must match the corresponding declaration/reference. |
| `target` | `HealthTargetV1` | Explicit; no implicit value | Target. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `cohort_role` | `CohortRoleV1` | Explicit; no implicit value | Validation or holdout cohort for scoring; development is unscoreable. |
| `domain` | `DomainSelectorV1` | Explicit; no implicit value | Domain. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `health_artifact_required` | `bool` | Explicit; no implicit value | Health artifact required. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `reference_rule` | `ReferenceAuthorityRuleV1` | Explicit; no implicit value | Explicit reference rule; allowed alternatives are enumerated below. |
| `predicted_positive_statuses` | `Vec<String>` | Explicit; no implicit value | Predicted positive statuses. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `predicted_negative_statuses` | `Vec<String>` | Explicit; no implicit value | Predicted negative statuses. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `reference_label_universe` | `Vec<String>` | Explicit; no implicit value | Reference label universe. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `reference_positive_labels` | `Vec<String>` | Explicit; no implicit value | Reference positive labels. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `reference_negative_labels` | `Vec<String>` | Explicit; no implicit value | Reference negative labels. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `minimum_eligible_records` | `u64` | Explicit; no implicit value | Lower support/acceptance bound for eligible records. Change to match the required evidence strength. |
| `minimum_independent_families` | `u64` | Explicit; no implicit value | Lower support/acceptance bound for independent families. Change to match the required evidence strength. |
| `required_strata` | `Vec<RequiredStratumV1>` | Explicit; no implicit value | Required strata. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `acceptance_rules` | `Vec<AcceptanceRuleV1>` | Explicit; no implicit value | Acceptance rules. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### StatisticsV1

Source: [`src/mhi_validation/protocol.rs`](../../src/mhi_validation/protocol.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `interval_method` | `String` | Explicit; no implicit value | Interval method. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `confidence_level` | `String` | Explicit; no implicit value | Confidence level. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `undefined_metric` | `String` | Explicit; no implicit value | Undefined metric. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `required_rule_unavailable` | `String` | Explicit; no implicit value | Required rule unavailable. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `rule_composition` | `String` | Explicit; no implicit value | Rule composition. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### ReleaseClaimV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `claim_id` | `String` | Explicit; no implicit value | Stable claim id; must match the corresponding declaration/reference. |
| `requested_level` | `RequestedValidationLevelV1` | Explicit; no implicit value | Requested level. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `statement` | `String` | Explicit; no implicit value | Statement. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `domain` | `DomainSelectorV1` | Explicit; no implicit value | Domain. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `supporting_endpoint_ids` | `Vec<String>` | Explicit; no implicit value | Stable supporting endpoint ids; must match the corresponding declaration/reference. |

### CategoricalSelectorV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum CategoricalSelectorV1 {
    AnyDeclared,
    Allowed { ids: Vec<String> },
}
```

### TemperatureSelectorV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum TemperatureSelectorV1 {
    AnyDeclared,
    Bands { bands: Vec<TemperatureBandV1> },
}
```

### CohortRoleV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum CohortRoleV1 {
    Development,
    Validation,
    Holdout,
}
```

### ReferenceAuthorityRuleV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ReferenceAuthorityRuleV1 {
    Mechanism {
        allowed_methods: Vec<ReferenceMethodV1>,
        allowed_authority_ids: Vec<String>,
        blinding_rule: BlindingRuleV1,
        uncertainty_rule: ReferenceUncertaintyRuleV1,
    },
    Health {
        allowed_methods: Vec<ReferenceMethodV1>,
        allowed_authority_ids: Vec<String>,
        blinding_rule: BlindingRuleV1,
        uncertainty_rule: ReferenceUncertaintyRuleV1,
    },
}
```

### RequiredStratumV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `stratum_id` | `String` | Explicit; no implicit value | Stable stratum id; must match the corresponding declaration/reference. |
| `predicates` | `Vec<StratumPredicateV1>` | Explicit; no implicit value | Predicates. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `minimum_eligible_records` | `u64` | Explicit; no implicit value | Lower support/acceptance bound for eligible records. Change to match the required evidence strength. |
| `minimum_independent_families` | `u64` | Explicit; no implicit value | Lower support/acceptance bound for independent families. Change to match the required evidence strength. |

### AcceptanceRuleV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum AcceptanceRuleV1 {
    Count {
        rule_id: String,
        metric: CountMetricV1,
        comparator: ComparatorV1,
        threshold_u64: u64,
    },
    Rate {
        rule_id: String,
        metric: RateMetricV1,
        target: RateTargetV1,
        comparator: ComparatorV1,
        threshold: f64,
    },
}
```

### HealthTargetV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum HealthTargetV1 {
    Dimension { dimension_id: String },
    Aggregate,
}
```

### RequestedValidationLevelV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum RequestedValidationLevelV1 {
    Software,
    Physical,
}
```

### TemperatureBandV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `lower_kelvin_inclusive` | `f64` | Explicit; no implicit value | Lower kelvin inclusive. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |
| `upper_kelvin_exclusive` | `f64` | Explicit; no implicit value | Upper kelvin exclusive. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### ReferenceMethodV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

| Field | Type | Presence/default | Meaning and use |
|---|---|---|---|
| `method_id` | `String` | Explicit; no implicit value | Stable method id; must match the corresponding declaration/reference. |
| `method_version` | `String` | Explicit; no implicit value | Method version. Select explicitly for the experiment/protocol; nested values follow the referenced type below. |

### BlindingRuleV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum BlindingRuleV1 {
    RequireBlinded,
    AllowDeclaredUnblinded,
}
```

### ReferenceUncertaintyRuleV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ReferenceUncertaintyRuleV1 {
    RequireQuantified {
        measure_id: String,
        unit: String,
        maximum_inclusive: f64,
    },
    AllowUnavailableWithLimitation,
}
```

### StratumPredicateV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum StratumPredicateV1 {
    AnalyteEquals {
        id: String,
    },
    MatrixEquals {
        id: String,
    },
    SensorDesignEquals {
        id: String,
    },
    SensorEquals {
        id: String,
    },
    CampaignEquals {
        id: String,
    },
    TemperatureBand {
        lower_kelvin_inclusive: f64,
        upper_kelvin_exclusive: f64,
    },
}
```

### CountMetricV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum CountMetricV1 {
    DeclaredCount,
    EligibleCount,
    ExcludedCount,
    NotApplicableCount,
    IndependentFamilyCount,
    SupportCount,
    CriticalContradictionCount,
    NotAssessedOrOtherCount,
    Tp,
    Tn,
    Fp,
    Fn,
    IndeterminateCount,
    DataQualityInsufficientCount,
    EvaluableCount,
}
```

### ComparatorV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum ComparatorV1 {
    GreaterThanOrEqual,
    LessThanOrEqual,
}
```

### RateMetricV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum RateMetricV1 {
    ExclusionRate,
    SupportFraction,
    ContradictionFraction,
    NotAssessedFraction,
    Coverage,
    IndeterminateRate,
    DataQualityInsufficientRate,
    Sensitivity,
    Specificity,
    FalsePositiveRate,
    FalseNegativeRate,
    BalancedAccuracy,
}
```

### RateTargetV1

Source: [`src/validation_config.rs`](../../src/validation_config.rs).

Allowed alternatives below are the exact declaration. With `rename_all = "snake_case"`, write `MinimumSteadyFraction` as `minimum_steady_fraction`, for example. Tagged alternatives require the shown `type` or `kind` plus their payload fields.

```rust
#[serde(rename_all = "snake_case")]
enum RateTargetV1 {
    PointEstimate,
    LowerConfidenceBound,
    UpperConfidenceBound,
}
```

## Phase-E statistics literal values

The software protocol fixture declares these closed values; alternative statistical methods are not selected by arbitrary strings.

```toml
[statistics]
interval_method = "wilson_95_v1"
confidence_level = "0.95"
undefined_metric = "unavailable"
required_rule_unavailable = "indeterminate"
rule_composition = "and"
```

The complete fixture is authoritative for the exact accepted literals; the configuration reader rejects unsupported statistics.
