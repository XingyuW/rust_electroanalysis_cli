# Strata oracle

Each record is evaluated independently for the overall view and each closed
stratum predicate.  The record and independent-family minimums apply to every
view; a passing aggregate never rescues an underpowered stratum.  The software
fixture supplies two eligible, independently-family-separated records for the
overall view; stratum fixtures introduce the underpowered cases separately.
