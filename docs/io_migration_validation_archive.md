# Canonical IO migration parity evidence archive

This archive is the required evidence record for the completed independent
legacy-parity gate. It preserves the evidence before removal of the test-only
legacy snapshot, its parity test, and the direct test-only `calamine`
dependency. The approval is recorded in
[`reviews/canonical_io_migration_final_go.md`](reviews/canonical_io_migration_final_go.md).

## Reviewed revisions and decision

| Item | Value |
|---|---|
| Tested consumer SHA | `5f1d73c31419287772a8fdc28093b594596111e4` |
| Prompt 3B baseline consumer SHA | `424f93e334fcf9956fa24fb21d8484f3f0ab0cd6` |
| Tested provider SHA | `dbb6b7d063972114c4208980723e12c807ab199e` |
| Archived implementation provenance | consumer `cc6f28379b04616cd54f5e8c94836bd4d14a2107` |
| Independent final-review decision | **GO WITH DOCUMENTED NON-BLOCKING DEBT — safe to proceed to Prompt 3B** |
| Gate result | six parity tests passed; the independent review reports 302 executable tests passed and zero failed |

## Fixture matrix

| Fixture group | Cases | Final comparison result |
|---|---|---|
| Time series | two-column, multichannel, missing cells, malformed timestamp, invalid numeric, ragged rows, CHI OCPT, multichannel OCPT | Complete consumer-domain parity; canonical source-coordinate and diagnostics additions are intentional. |
| EIS parity | four-column phase, five-column | Frequency, real/imaginary values and sign, source Bode values, derived Bode values, metadata, and rows matched. |
| XLSX parity | simple time series, historical-preamble time series | Complete time-series domain parity. |
| Canonical-only acceptance | headerless, DAT, three-column EIS, four-column magnitude EIS, reordered EIS, XLSX EIS | Explicitly classified provider-backed improvements, not unexplained drift. |

## Intentional-difference allowlist

The removed parity test allowed only these reviewed additions:

1. source time-coordinate name and unit;
2. logical channel name plus exact source-header metadata;
3. optional channel variance/sensor/analyte fields;
4. canonical provider recovery diagnostics;
5. source-measured EIS Bode quantities distinct from derived quantities;
6. structured EIS acquisition/provenance metadata; and
7. resolved EIS circuit-model hint.

Each was classified as a schema/provenance addition with no unexplained
scientifically consequential difference.

## Scientific-output regression reference

The independent review compared deterministic workflows with the pre-migration
consumer and recorded these approved reference outcomes:

| Workflow | Approved reference outcome |
|---|---|
| EIS fit | report byte-identical |
| Transient fit | scientific JSON exact after path/timestamp and additive diagnostics |
| Calibration extraction | scientific JSON exact after provenance fields |
| Signal characterize | scientific JSON exact after provenance/config paths |
| Signal compare | result JSON byte-identical |
| Estimate run | numeric/scientific JSON exact after documented channel-identity and diagnostic additions |
| Estimate compare | numeric/scientific JSON exact after schema/diagnostic additions, excluding runtime timing |

These exact-output references, the fixture matrix, and the intentional-
difference allowlist are the permanent migration evidence. Canonical input and
scientific regression tests remain in the repository; this archive replaces
only the completed independent legacy implementation and comparison gate.
