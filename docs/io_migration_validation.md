# Canonical IO migration validation

## Reproducible provider dependency

The consumer pins `electrodata-io` in `Cargo.toml` to the reviewed commit
`dbb6b7d063972114c4208980723e12c807ab199e` using the public HTTPS Git URL:

```toml
electrodata-io = { git = "https://github.com/XingyuW/electrodata-io.git", rev = "dbb6b7d063972114c4208980723e12c807ab199e" }
```

`Cargo.lock` records the same Git source and resolved commit. No consumer path
dependency or sibling checkout is required.

## Completed independent parity gate

The independent legacy reference and parity test were deliberately removed in
Prompt 3B only after the final GO decision. Their tested consumer/provider
SHAs, fixture matrix, intentional-difference allowlist, scientific-output
references, and final result are retained in
`docs/io_migration_validation_archive.md`. Canonical boundary, typed-error,
XLSX, EIS semantic, and scientific workflow regressions remain active tests.

## Archived parity matrix

| Fixture | Classification | Recorded result |
| --- | --- | --- |
| regular two-column | Intentional canonical improvement | Numeric cells, nulls, timestamps, units, and order match; canonical retains `Potential/V` rather than the legacy stripped `Potential`. |
| multichannel | Intentional canonical improvement | All channels and cells match in order; canonical retains complete source headers. |
| headerless | Intentional canonical improvement | `cc6f283` rejects it; canonical compatibility mode infers the first numeric row as data. |
| missing cells | Intentional canonical improvement | Cells/null positions match; canonical retains complete source headers. |
| malformed timestamp | Intentional canonical improvement | Both retain the same valid rows; canonical reports the skipped source row and retains complete source header. |
| invalid numeric measurement | Intentional canonical improvement | Both produce a null cell; canonical retains recovery diagnostics and complete source header. |
| ragged rows | Intentional canonical improvement | Both retain/pad the same cells; canonical retains ragged-row diagnostics and complete source header. |
| DAT | Intentional canonical improvement | `cc6f283` CSV-only splitter rejects tab-delimited DAT; canonical reads the source table. |
| CHI OCPT | Intentional canonical improvement | Timestamp/value/unit parity; canonical retains the complete source header. |
| multichannel OCPT | Intentional canonical improvement | Channel order and every value match; canonical retains complete source headers. |
| 3-column EIS | Intentional canonical improvement | Legacy requires a phase column; canonical supports the valid three-role EIS table and derives magnitude/phase. |
| 4-column EIS | Exact parity | Frequency, real, imaginary/sign, source phase, derived magnitude/phase, row count, and units match. |
| 5-column EIS | Exact parity | Frequency, real, imaginary/sign, source magnitude/phase, derived magnitude/phase, row count, and units match. |
| reordered EIS | Intentional canonical improvement | Legacy requires the `Freq/Hz` header in column one; canonical resolves semantic roles in any order. |
| simple XLSX time-series | Exact parity | Archived XLSX reader and canonical reader retain raw timestamps, channel values, nulls, order, names, and units. |
| historical preamble XLSX | Exact parity | Same full time-series comparison against a workbook with leading historical metadata rows. |

There are zero unexplained differences. There are no scientifically
consequential regressions in this corpus and no intentional breaking migration
cases.

## Clean-checkout verification

Run this from a clean consumer clone. It never reads a sibling provider checkout:

```bash
git clone https://github.com/XingyuW/rust_electroanalysis_cli.git rust_electroanalysis_cli-clean
cd rust_electroanalysis_cli-clean
git rev-parse HEAD
git clone https://github.com/XingyuW/electrodata-io.git /tmp/electrodata-io-review
git -C /tmp/electrodata-io-review rev-parse dbb6b7d063972114c4208980723e12c807ab199e
cargo fetch --locked
cargo build --locked
cargo test --all --locked
```

The provider clone in the fourth command is a review assertion only. Cargo
resolves the provider directly from the exact `Cargo.toml` Git `rev` and the
matching `Cargo.lock` source entry.
