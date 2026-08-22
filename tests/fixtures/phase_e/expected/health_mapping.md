# Health mapping oracle

For evaluated health records, predicted positive/negative and reference
positive/negative sets form the disjoint `tp`, `tn`, `fp`, and `fn`
sets.  `indeterminate` and `data_quality_insufficient` remain disjoint
missing-state sets.  A label outside the declared universe is a hard binding
failure.  The software fixture has no eligible health rows, so coverage,
sensitivity, specificity, and balanced accuracy are unavailable.
