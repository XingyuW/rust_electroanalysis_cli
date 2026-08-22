# Mechanism mapping oracle

The Phase-B/reference mapping is closed:

| Phase-B status | reference outcome | category |
| --- | --- | --- |
| hypothesized, experimentally_supported, validated_for_domain | supports | support only when the configured support set contains the status |
| any | contradicts | critical contradiction |
| any | not_assessed | not_assessed_or_other |
| any | unavailable | excluded for software and a physical pre-scoring hard failure |

The software fixture has zero eligible mechanism rows; all three rate values are
`unavailable(0/0)` and the endpoint is `indeterminate`.
