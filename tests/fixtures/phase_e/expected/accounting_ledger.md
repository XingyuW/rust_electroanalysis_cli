# Accounting oracle

For the software fixture, each endpoint declares record `record_1`.
The record lacks its required artifact and matching reference endpoint, so the
primary exclusion is `missing_endpoint_artifact_path`, with the ordered
secondary reason `missing_reference_endpoint`.  Thus, for both endpoints:

| declared | eligible | excluded | not-applicable | outcome |
| ---: | ---: | ---: | ---: | --- |
| 1 | 0 | 1 | 0 | indeterminate |

The identity `declared = eligible + excluded + not-applicable` holds exactly.
