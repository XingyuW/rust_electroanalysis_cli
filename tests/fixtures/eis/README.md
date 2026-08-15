# Deterministic EIS fitting fixture

`randles_cpe_weighted_fit.csv` is a synthetic, deterministic impedance spectrum.
It was generated from the `R0-p(CPE1,R1)` circuit with `R0 = 260 ohm`,
`R1 = 5000 ohm`, `Q = 2.0e-5`, and `alpha = 0.88`, then perturbed by a small,
fixed sinusoidal measurement-like error.  It contains no experimental or
redistributed source data.  The fixture verifies the bounded weighted-error
behavior of the scientific circuit fitter through the canonical input path.
