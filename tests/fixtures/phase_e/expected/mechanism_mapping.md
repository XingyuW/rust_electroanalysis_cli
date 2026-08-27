# Mechanism mapping oracle

The Phase-B/reference mapping is closed:

| Phase-B status | reference outcome | category |
| --- | --- | --- |
| hypothesized, experimentally_supported, validated_for_domain | supports | support only when the configured support set contains the status |
| any | contradicts | critical contradiction |
| any | not_assessed | not_assessed_or_other |
| any | unavailable | excluded for software and a physical pre-scoring hard failure |

The software fixture has two eligible mechanism rows from independently named
acquisition families.  Both are `validated_for_domain` and independently
support the reference, so support is `2/2`, contradiction is `0/2`, and the
endpoint meets its configured protocol.
