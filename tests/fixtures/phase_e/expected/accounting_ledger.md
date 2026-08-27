# Accounting oracle

For the software fixture, each endpoint declares records `record_1` and
`record_2`; both are eligible and neither is excluded.
Each record has a phase-B/Phase-C schema-4 source and an allowed blinded,
quantified reference endpoint.  Thus, for both endpoints:

| declared | eligible | excluded | not-applicable | outcome |
| ---: | ---: | ---: | ---: | --- |
| 2 | 2 | 0 | 0 | meets_protocol |

The identity `declared = eligible + excluded + not-applicable` holds exactly.
