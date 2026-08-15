# Reduced-order ISM components V1 implementation plan

1. Extend the existing static component registry with activity-first, V1
   adapters. Reuse the established Nernst, Nicolsky--Eisenman, activity, and
   transient equations; do not duplicate their scientific calculations.
2. Add a generic explicit-event first-order potential mode, a candidate
   transduction instance, a constant reference-offset state, generic typed
   linear covariates, and a variance-only observation-noise component.
3. Replace the default model definition with the versioned `reduced_ism_v1`
   composition: activity adapter, Nernst equilibrium, two independent dynamic
   modes, reference offset, and observation variance. Keep candidate
   transduction and environmental covariates out of the active default graph.
4. Complete model-core equilibrium recognition using only supplied neutral
   evidence, validity, observability, and uncertainty status.
5. Add parity, analytical-solution, decomposition, validity, neutrality, and
   recognition tests; update the model documentation and example definition.
