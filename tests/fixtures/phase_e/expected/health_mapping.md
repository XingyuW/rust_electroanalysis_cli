# Health mapping oracle

For evaluated health records, predicted positive/negative and reference
positive/negative sets form the disjoint `tp`, `tn`, `fp`, and `fn`
sets.  `indeterminate` and `data_quality_insufficient` remain disjoint
missing-state sets.  A label outside the declared universe is a hard binding
failure.  The software fixture has two eligible rows for `signal_integrity`:
`record_1` is a normal within-baseline TN and `record_2` is an alert critical
TP. Coverage, sensitivity, specificity, and balanced accuracy are therefore
all exactly `1.0`.
