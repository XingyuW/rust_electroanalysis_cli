#![allow(clippy::large_enum_variant)]

//! Genetic algorithm evolution for equivalent-circuit model (ECM) discovery.
//!
//! # Architecture changes (ISM-AI v2)
//! - **Dynamic tree genome**: `CircuitGenome` is now the UTF-8 bytes of the
//!   canonical circuit string produced by `CircuitTopology`.  This allows
//!   arbitrarily nested parallel/series topologies rather than the previous
//!   fixed 5-byte encoding.
//! - **Expanded element pool**: TLMQ, T, Gs, G, Zarc, and all diffusion
//!   elements are available to the mutation operators.
//! - **Sub-tree operators**: `subtree_swap_mutation` exchanges two independent
//!   subtrees within one chromosome; `node_insertion` wraps a node in a new
//!   Parallel block, growing circuit complexity organically.
//! - **PINN fitness evaluator**: Each candidate is scored by running the
//!   Physics-Informed Neural Network optimizer (Adam gradient descent with
//!   physics-bound and Kramers-Kronig loss terms).  The resulting AIC is
//!   converted to a GA fitness value so that both data fit AND parsimony are
//!   rewarded simultaneously.

use super::PreparedImpedanceData;
use super::circuits::Impedance;
use super::ecm_candidate::{
    CircuitGenome, CircuitTopology, LeafKind, candidate_from_genome, genome_from_topology,
    normalize_genome, randles_topology, seed_genomes, topology_from_genome,
};
use super::ecm_scoring::{CandidateFitResult, bic, legacy_penalized_score, weighted_rmse};
use super::fitting::{
    BorrowedImpedanceFitter, guess_parameters, sanitize_physical_params, transform_backward,
    transform_forward,
};
use super::parse_circuit_string;
use super::pinn_optimizer::{PinnOptimizer, compute_aic};
use super::prepare_impedance_data;
use crate::domain::FittingError;
use genevo::genetic::{Children, Parents};
use genevo::operator::{CrossoverOp, GeneticOperator, MutationOp, SingleObjective};
use genevo::prelude::*;
use genevo::random::Rng;
use genevo::reinsertion::elitist::ElitistReinserter;
use genevo::selection::truncation::MaximizeSelector;
use levenberg_marquardt::LevenbergMarquardt;
use nalgebra::DVector;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

const FITNESS_SCALE: f64 = 1_000_000_000.0;

// ─────────────────────────────────────────────────────────────────────────────
// Public config / outcome types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EcmEvolutionConfig {
    pub population_size: usize,
    pub generation_limit: u64,
    pub num_individuals_per_parents: usize,
    pub selection_ratio: f64,
    pub mutation_rate: f64,
    pub reinsertion_ratio: f64,
}

impl Default for EcmEvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 24,
            generation_limit: 12,
            num_individuals_per_parents: 2,
            selection_ratio: 0.7,
            mutation_rate: 0.35,
            reinsertion_ratio: 0.75,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EcmEvolutionOutcome {
    pub generations_processed: u64,
    pub best_fitness: i64,
    pub unique_candidates_evaluated: usize,
    pub evaluated_candidates: Vec<CandidateFitResult>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Evaluation cache
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct EvaluationRecord {
    /// Integer fitness value consumed by the GA engine.
    fitness: i64,
    /// Optional full fit artifact cached for ranking/reporting output.
    fit: Option<CandidateFitResult>,
}

#[derive(Debug)]
enum CacheEntry {
    /// Evaluation is currently being computed by another worker.
    Pending,
    /// Evaluation completed and is ready for reuse.
    Ready(EvaluationRecord),
}

#[derive(Debug, Default)]
struct SearchEvaluationCache {
    entries: Mutex<HashMap<String, CacheEntry>>,
    wakeup: Condvar,
}

impl SearchEvaluationCache {
    /// Return an existing evaluation record for `key` or compute/store it once.
    ///
    /// Concurrency behavior:
    /// - first caller inserts `Pending` and performs work
    /// - concurrent callers block on `Condvar` until result is published
    /// - all later callers read cached `Ready` value without recomputation
    fn get_or_evaluate<F>(&self, key: String, evaluate: F) -> EvaluationRecord
    where
        F: FnOnce() -> EvaluationRecord,
    {
        loop {
            let mut entries = self.entries.lock().expect("cache mutex poisoned");
            match entries.get(&key) {
                Some(CacheEntry::Ready(record)) => return record.clone(),
                Some(CacheEntry::Pending) => {
                    drop(self.wakeup.wait(entries).expect("cache wait failed"));
                }
                None => {
                    entries.insert(key.clone(), CacheEntry::Pending);
                    drop(entries);
                    let result = evaluate();
                    let mut entries = self.entries.lock().expect("cache mutex poisoned");
                    entries.insert(key.clone(), CacheEntry::Ready(result.clone()));
                    self.wakeup.notify_all();
                    return result;
                }
            }
        }
    }

    /// Collect only successful fit artifacts from cache for post-run ranking.
    fn successful_fits(&self) -> Vec<CandidateFitResult> {
        self.entries
            .lock()
            .expect("cache mutex poisoned")
            .values()
            .filter_map(|entry| match entry {
                CacheEntry::Ready(r) => r.fit.clone(),
                CacheEntry::Pending => None,
            })
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Fitness evaluator (PINN-based)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct CircuitFitnessEvaluator {
    frequencies: Arc<Vec<f64>>,
    z_real: Arc<Vec<f64>>,
    z_imag: Arc<Vec<f64>>,
    phase_deg: Arc<Vec<f64>>,
    prepared: Option<Arc<PreparedImpedanceData>>,
    cache: Arc<SearchEvaluationCache>,
}

impl CircuitFitnessEvaluator {
    /// Construct an evaluator with optional prepared-data cache when input
    /// vectors are shape-compatible.
    fn new(frequencies: &[f64], z_real: &[f64], z_imag: &[f64], phase_deg: &[f64]) -> Self {
        let lengths_valid = frequencies.len() == z_real.len()
            && frequencies.len() == z_imag.len()
            && (phase_deg.is_empty() || frequencies.len() == phase_deg.len());
        let prepared = if lengths_valid {
            prepare_impedance_data(frequencies, z_real, z_imag, phase_deg)
                .ok()
                .map(Arc::new)
        } else {
            None
        };
        Self {
            frequencies: Arc::new(frequencies.to_vec()),
            z_real: Arc::new(z_real.to_vec()),
            z_imag: Arc::new(z_imag.to_vec()),
            phase_deg: Arc::new(phase_deg.to_vec()),
            prepared,
            cache: Arc::new(SearchEvaluationCache::default()),
        }
    }

    /// Return all cached successful candidate fits encountered so far.
    fn evaluated_candidates(&self) -> Vec<CandidateFitResult> {
        self.cache.successful_fits()
    }
}

impl FitnessFunction<CircuitGenome, i64> for CircuitFitnessEvaluator {
    /// Evaluates circuit fitness using the PINN optimizer.
    ///
    /// For each unique circuit topology:
    /// 1. Parse the circuit string into a `CircuitNode`.
    /// 2. Generate a physics-informed initial parameter guess.
    /// 3. Run `PinnOptimizer::optimize` (Adam gradient descent with
    ///    physics-bounds loss) to obtain a physically consistent warm-start.
    /// 4. Polish the warm-start with Levenberg–Marquardt, which has
    ///    Jacobian-based convergence and matches the accuracy of the original
    ///    GA-only approach.
    /// 5. Convert the AIC of the LM result to an integer fitness score.
    ///    Lower AIC → better fit → higher fitness.
    fn fitness_of(&self, genome: &CircuitGenome) -> i64 {
        let candidate = candidate_from_genome(genome);
        let circuit_string = candidate.to_circuit_string();

        self.cache
            .get_or_evaluate(circuit_string.clone(), || {
                let circuit = match parse_circuit_string(&circuit_string) {
                    Ok(c) => c,
                    Err(_) => {
                        return EvaluationRecord {
                            fitness: 0,
                            fit: None,
                        };
                    }
                };

                // Use pre-sorted/validated data from PreparedImpedanceData when available.
                let (frequencies, z_real, z_imag, phase_deg) =
                    if let Some(p) = self.prepared.as_deref() {
                        (
                            p.frequencies.as_slice(),
                            p.z_real.as_slice(),
                            p.z_imag.as_slice(),
                            p.phase_deg.as_slice(),
                        )
                    } else {
                        (
                            self.frequencies.as_slice(),
                            self.z_real.as_slice(),
                            self.z_imag.as_slice(),
                            self.phase_deg.as_slice(),
                        )
                    };

                // ── Step 1: PINN warm-start ────────────────────────────────
                // Adam gradient descent with physics-bounds regularisation
                // navigates to a physically consistent parameter region before
                // handing off to the Jacobian-driven LM solver.
                let initial_params =
                    guess_parameters(&circuit, frequencies, z_real, z_imag, phase_deg);
                let pinn_optimizer = PinnOptimizer::new(&circuit, frequencies, z_real, z_imag);
                let pinn = pinn_optimizer.optimize(&initial_params);

                if pinn.fitted_params.is_empty() {
                    return EvaluationRecord {
                        fitness: 0,
                        fit: None,
                    };
                }

                // ── Step 2: LM polish ─────────────────────────────────────
                // Run Levenberg–Marquardt from the PINN warm-start. LM uses
                // Jacobian information and converges to a precise local
                // minimum in far fewer evaluations than Adam, recovering the
                // fitting quality of the original GA-only approach.
                //
                // Build PreparedImpedanceData for LM (reuse cached copy if
                // available, otherwise construct a fresh one).
                let fresh_prep = if self.prepared.is_none() {
                    prepare_impedance_data(frequencies, z_real, z_imag, phase_deg).ok()
                } else {
                    None
                };
                let pd_opt: Option<&PreparedImpedanceData> =
                    self.prepared.as_deref().or(fresh_prep.as_ref());

                let constraints = circuit.get_constraints();
                let bounds = circuit.get_bounds();

                // Convert PINN physical params back to unconstrained LM space.
                let lm_init: Vec<f64> = pinn
                    .fitted_params
                    .iter()
                    .zip(constraints.iter())
                    .map(|(&p, &c)| transform_forward(p, c))
                    .collect();

                let (final_params, final_z_re, final_z_im) = match pd_opt {
                    Some(pd) => {
                        let fitter = BorrowedImpedanceFitter {
                            circuit: &circuit,
                            omegas: &pd.omegas,
                            z_real_data: &pd.z_real,
                            z_imag_data: &pd.z_imag,
                            weights: &pd.weights,
                            params: DVector::from_vec(lm_init),
                            constraints: &constraints,
                            bounds: &bounds,
                        };
                        let solver = LevenbergMarquardt::new()
                            .with_ftol(1e-10)
                            .with_xtol(1e-10)
                            .with_gtol(1e-10)
                            .with_patience(200)
                            .with_stepbound(50.0);
                        let (lm_result, _) = solver.minimize(fitter);
                        let params = sanitize_physical_params(
                            &lm_result
                                .params
                                .iter()
                                .zip(constraints.iter())
                                .map(|(&p, &c)| transform_backward(p, c))
                                .collect::<Vec<_>>(),
                            &constraints,
                            &bounds,
                        );
                        let z_re: Vec<f64> = pd
                            .omegas
                            .iter()
                            .map(|&w| circuit.calculate(w, &params).re)
                            .collect();
                        let z_im: Vec<f64> = pd
                            .omegas
                            .iter()
                            .map(|&w| circuit.calculate(w, &params).im)
                            .collect();
                        (params, z_re, z_im)
                    }
                    None => {
                        // Prepared data unavailable; fall back to PINN result.
                        (pinn.fitted_params, pinn.fitted_z_re, pinn.fitted_z_im)
                    }
                };

                if final_params.is_empty() || final_z_re.iter().any(|v| !v.is_finite()) {
                    return EvaluationRecord {
                        fitness: 0,
                        fit: None,
                    };
                }

                // ── Step 3: Score ─────────────────────────────────────────
                let legacy_score = legacy_penalized_score(z_real, z_imag, &final_z_re, &final_z_im);
                let w_rmse = weighted_rmse(z_real, z_imag, &final_z_re, &final_z_im);

                let n = frequencies.len();
                let k = final_params.len();
                // Compute MSE from unweighted residuals for consistent AIC.
                let sum_sq: f64 = (0..n.min(final_z_re.len()))
                    .map(|i| {
                        let re = final_z_re[i] - z_real[i];
                        let im = final_z_im[i] - z_imag[i];
                        re * re + im * im
                    })
                    .sum();
                let mse = (sum_sq / (2.0 * n.max(1) as f64)).max(1e-30);
                let aic = compute_aic(n, k, mse);
                let residual_sum_of_squares = sum_sq;
                let bic_val = bic(residual_sum_of_squares, k, 2 * n);

                if !aic.is_finite() {
                    return EvaluationRecord {
                        fitness: 0,
                        fit: None,
                    };
                }

                let fitted_magnitude: Vec<f64> = final_z_re
                    .iter()
                    .zip(final_z_im.iter())
                    .map(|(&re, &im)| re.hypot(im))
                    .collect();
                let fitted_phase: Vec<f64> = final_z_re
                    .iter()
                    .zip(final_z_im.iter())
                    .map(|(&re, &im)| im.atan2(re).to_degrees())
                    .collect();

                let fit = CandidateFitResult {
                    circuit_string,
                    residual_sum_of_squares,
                    weighted_residual_sum_of_squares: Some(legacy_score),
                    bic: bic_val,
                    legacy_penalized_score: Some(legacy_score),
                    weighted_rmse: w_rmse,
                    parameter_count: k,
                    fitted_parameters: final_params,
                    parameter_names: circuit.get_param_names(),
                    parameter_units: circuit.get_param_units(),
                    fitted_z_re: final_z_re,
                    fitted_z_im: final_z_im,
                    fitted_magnitude,
                    fitted_phase,
                };

                EvaluationRecord {
                    fitness: fitness_from_aic(aic),
                    fit: Some(fit),
                }
            })
            .fitness
    }

    fn average(&self, fitness_values: &[i64]) -> i64 {
        if fitness_values.is_empty() {
            return 0;
        }
        let sum: i128 = fitness_values.iter().map(|v| *v as i128).sum();
        (sum / fitness_values.len() as i128) as i64
    }

    fn highest_possible_fitness(&self) -> i64 {
        i64::MAX
    }
    fn lowest_possible_fitness(&self) -> i64 {
        0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Seeded genome builder
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct SeededCircuitGenomeBuilder {
    seeds: Vec<CircuitGenome>,
    max_seed_mutations: usize,
}

impl SeededCircuitGenomeBuilder {
    fn new(seeds: Vec<CircuitGenome>) -> Self {
        Self {
            seeds,
            max_seed_mutations: 2,
        }
    }
}

impl GenomeBuilder<CircuitGenome> for SeededCircuitGenomeBuilder {
    /// Build one initial genome for the GA population.
    ///
    /// Determinism policy:
    /// - first `seeds.len()` genomes are deterministic seed copies
    /// - remaining genomes are seed-derived with light random mutation to
    ///   improve initial diversity while preserving physically plausible starts
    fn build_genome<R>(&self, index: usize, rng: &mut R) -> CircuitGenome
    where
        R: Rng + Sized,
    {
        if self.seeds.is_empty() {
            return genome_from_topology(&randles_topology());
        }
        if index < self.seeds.len() {
            return self.seeds[index].clone();
        }
        let mut genome = self.seeds[rng.gen_range(0..self.seeds.len())].clone();
        let mutation_count = 1 + rng.gen_range(0..=self.max_seed_mutations);
        for _ in 0..mutation_count {
            genome = apply_tree_mutation_step(genome, rng);
        }
        genome
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tree crossover operator (Sub-tree Swap)
// ─────────────────────────────────────────────────────────────────────────────

/// Performs sub-tree swap crossover at the topology level.
///
/// Two parent genomes are decoded into `CircuitTopology` trees.  A random
/// subtree path is selected in each parent; the subtrees are exchanged to
/// produce two offspring.  If the resulting topologies are invalid or exceed
/// the size limits both offspring fall back to their respective parents.
#[derive(Clone, Debug, PartialEq)]
struct CircuitTreeCrossover;

impl GeneticOperator for CircuitTreeCrossover {
    fn name() -> String {
        "CircuitTreeCrossover".to_string()
    }
}

impl SingleObjective for CircuitTreeCrossover {}

impl CrossoverOp<CircuitGenome> for CircuitTreeCrossover {
    fn crossover<R>(&self, parents: Parents<CircuitGenome>, rng: &mut R) -> Children<CircuitGenome>
    where
        R: Rng + Sized,
    {
        // Parents<G> = Vec<G>: a flat list of parent genomes.
        // Process consecutive pairs, producing 2 children per pair.
        let mut children: Vec<CircuitGenome> = Vec::with_capacity(parents.len());
        let mut i = 0;
        while i + 1 < parents.len() {
            let p1 = topology_from_genome(&parents[i]).unwrap_or_else(randles_topology);
            let p2 = topology_from_genome(&parents[i + 1]).unwrap_or_else(randles_topology);
            let (c1, c2) = subtree_swap_crossover(&p1, &p2, rng);
            children.push(genome_from_topology(&c1));
            children.push(genome_from_topology(&c2));
            i += 2;
        }
        // Pass through any odd-out parent unchanged.
        if i < parents.len() {
            children.push(parents[i].clone());
        }
        children
    }
}

fn subtree_swap_crossover<R: Rng>(
    p1: &CircuitTopology,
    p2: &CircuitTopology,
    rng: &mut R,
) -> (CircuitTopology, CircuitTopology) {
    // Retry several random path pairs to find a valid within-limits swap.
    let paths1 = p1.all_paths();
    let paths2 = p2.all_paths();

    const ATTEMPTS: usize = 5;
    for _ in 0..ATTEMPTS {
        let path1 = paths1[rng.gen_range(0..paths1.len())].clone();
        let path2 = paths2[rng.gen_range(0..paths2.len())].clone();

        let sub1 = match p1.get_at_path(&path1) {
            Some(n) => n.clone(),
            None => continue,
        };
        let sub2 = match p2.get_at_path(&path2) {
            Some(n) => n.clone(),
            None => continue,
        };

        let c1 = p1.clone().set_at_path(&path1, sub2).normalize();
        let c2 = p2.clone().set_at_path(&path2, sub1).normalize();

        if c1.validate().is_ok()
            && c1.is_within_limits()
            && c2.validate().is_ok()
            && c2.is_within_limits()
        {
            return (c1, c2);
        }
    }
    (p1.clone(), p2.clone())
}

// ─────────────────────────────────────────────────────────────────────────────
// Mutation operator
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct CircuitMutationOperator {
    mutation_rate: f64,
}

impl CircuitMutationOperator {
    fn new(mutation_rate: f64) -> Self {
        Self { mutation_rate }
    }
}

impl GeneticOperator for CircuitMutationOperator {
    fn name() -> String {
        "TreeEcmMutator".to_string()
    }
}

impl SingleObjective for CircuitMutationOperator {}

impl MutationOp<CircuitGenome> for CircuitMutationOperator {
    /// Apply stochastic topology mutation.
    ///
    /// With probability `mutation_rate` performs one or two tree-mutation
    /// steps, then canonicalizes genome encoding.
    fn mutate<R>(&self, genome: CircuitGenome, rng: &mut R) -> CircuitGenome
    where
        R: Rng + Sized,
    {
        if rng.r#gen::<f64>() > self.mutation_rate {
            return normalize_genome(genome);
        }
        let steps = if rng.r#gen::<f64>() < (self.mutation_rate * 0.5).min(0.95) {
            2
        } else {
            1
        };
        let mut result = normalize_genome(genome);
        for _ in 0..steps {
            result = apply_tree_mutation_step(result, rng);
        }
        result
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

pub fn run_ecm_evolution(
    frequencies: &[f64],
    z_real: &[f64],
    z_imag: &[f64],
    phase_deg: &[f64],
    config: &EcmEvolutionConfig,
) -> Result<EcmEvolutionOutcome, FittingError> {
    // Minimal shape/emptiness guard before any expensive setup.
    if frequencies.is_empty() || z_real.is_empty() || z_imag.is_empty() {
        return Err(FittingError::invalid_input(
            "frequency, real impedance, and imaginary impedance data are required",
        ));
    }

    let evaluator = CircuitFitnessEvaluator::new(frequencies, z_real, z_imag, phase_deg);

    // Pre-warm the evaluation cache for all canonical seed genomes in parallel.
    // genevo calls `fitness_of` sequentially in its inner loop; by populating
    // the cache before the simulation starts, the first generation's evaluation
    // cost is paid up front across all available CPU cores instead of serially.
    // Subsequent generations benefit from cache hits for any seed topology that
    // resurfaces via crossover or mutation.
    {
        let seeds = seed_genomes();
        seeds.par_iter().for_each(|genome| {
            evaluator.fitness_of(genome);
        });
    }

    let initial_population: Population<CircuitGenome> = build_population()
        .with_genome_builder(SeededCircuitGenomeBuilder::new(seed_genomes()))
        .of_size(config.population_size.max(8))
        .uniform_at_random();

    let mut simulation = simulate(
        genetic_algorithm()
            .with_evaluation(evaluator.clone())
            .with_selection(MaximizeSelector::new(
                config.selection_ratio,
                config.num_individuals_per_parents.max(2),
            ))
            .with_crossover(CircuitTreeCrossover)
            .with_mutation(CircuitMutationOperator::new(config.mutation_rate))
            .with_reinsertion(ElitistReinserter::new(
                evaluator.clone(),
                false,
                config.reinsertion_ratio,
            ))
            .with_initial_population(initial_population)
            .build(),
    )
    .until(GenerationLimit::new(config.generation_limit.max(1)))
    .build();

    // Simulation loop: keep stepping until the configured generation limit.
    let (generations_processed, best_fitness) = loop {
        match simulation.step() {
            Ok(SimResult::Intermediate(_)) => {}
            Ok(SimResult::Final(step, _, _, _)) => {
                break (step.iteration, step.result.best_solution.solution.fitness);
            }
            Err(e) => return Err(FittingError::search(format!("ECM evolution failed: {e}"))),
        }
    };

    // Final ranked artifacts are ordered for deterministic report output.
    let mut evaluated_candidates = evaluator.evaluated_candidates();
    evaluated_candidates.par_sort_by(|a, b| {
        a.bic
            .partial_cmp(&b.bic)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                a.legacy_penalized_score
                    .unwrap_or(f64::INFINITY)
                    .partial_cmp(&b.legacy_penalized_score.unwrap_or(f64::INFINITY))
                    .unwrap_or(Ordering::Equal)
            })
    });

    Ok(EcmEvolutionOutcome {
        generations_processed,
        best_fitness,
        unique_candidates_evaluated: evaluated_candidates.len(),
        evaluated_candidates,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Fitness mapping
// ─────────────────────────────────────────────────────────────────────────────

/// Maps AIC to an integer fitness score.
///
/// Uses a sigmoid-like transformation so that both very good fits (large
/// negative AIC) and poor fits (large positive AIC) are well distinguished.
/// The mapping `FITNESS_SCALE / (1 + exp(aic / 100))` gives:
///   AIC = -600 → fitness ≈ 997_500_000  (excellent)
///   AIC =    0 → fitness ≈ 500_000_000  (average)
///   AIC = +600 → fitness ≈     2_500_000 (poor)
fn fitness_from_aic(aic: f64) -> i64 {
    if !aic.is_finite() {
        return 0;
    }
    let x = aic / 100.0;
    let sigmoid = 1.0 / (1.0 + x.exp());
    let scaled = (FITNESS_SCALE * sigmoid).round();
    scaled.clamp(1.0, FITNESS_SCALE) as i64
}

// ─────────────────────────────────────────────────────────────────────────────
// Tree mutation operators
// ─────────────────────────────────────────────────────────────────────────────

fn apply_tree_mutation_step<R: Rng>(genome: CircuitGenome, rng: &mut R) -> CircuitGenome {
    // Decode -> mutate topology -> re-encode canonical genome bytes.
    let tree = topology_from_genome(&genome).unwrap_or_else(randles_topology);
    let mutated = match rng.gen_range(0u32..8) {
        0 | 1 => mutate_leaf_kind(tree, rng), // 25%  change element type
        2 => insert_series_element(tree, rng), // 12.5% add element in series
        3 => wrap_node_in_parallel(tree, rng), // 12.5% wrap node in new parallel block
        4 => subtree_swap_mutation(tree, rng), // 12.5% swap two subtrees
        5 => node_insertion(tree, rng),       // 12.5% insert parallel arc
        6 => prune_leaf(tree, rng),           // 12.5% remove a leaf
        _ => mutate_leaf_kind(tree, rng),     // 12.5% fallback
    };
    genome_from_topology(&mutated)
}

/// Changes one randomly chosen leaf to a different element kind.
fn mutate_leaf_kind<R: Rng>(tree: CircuitTopology, rng: &mut R) -> CircuitTopology {
    let leaf_paths = tree.leaf_paths();
    if leaf_paths.is_empty() {
        return tree;
    }
    let path = leaf_paths[rng.gen_range(0..leaf_paths.len())].clone();
    let all = LeafKind::all();
    let new_kind = all[rng.gen_range(0..all.len())];
    let candidate = tree
        .clone()
        .set_at_path(&path, CircuitTopology::Leaf(new_kind))
        .normalize();
    if candidate.validate().is_ok() && candidate.is_within_limits() {
        candidate
    } else {
        tree
    }
}

/// Adds a new leaf as a child of an existing Series node, or wraps the root
/// in a new Series if no Series node exists.
fn insert_series_element<R: Rng>(tree: CircuitTopology, rng: &mut R) -> CircuitTopology {
    let all_paths = tree.all_paths();
    let series_paths: Vec<Vec<usize>> = all_paths
        .into_iter()
        .filter(|p| matches!(tree.get_at_path(p), Some(CircuitTopology::Series(_))))
        .collect();

    let new_leaf = CircuitTopology::Leaf(random_leaf(rng));

    if series_paths.is_empty() {
        // Wrap the whole tree in a new series with the new element.
        let candidate = CircuitTopology::Series(vec![tree.clone(), new_leaf]).normalize();
        return if candidate.validate().is_ok() && candidate.is_within_limits() {
            candidate
        } else {
            tree
        };
    }

    let path = series_paths[rng.gen_range(0..series_paths.len())].clone();

    // Extract the existing children, insert the new leaf, rebuild.
    let new_node_opt: Option<CircuitTopology> =
        if let Some(CircuitTopology::Series(children)) = tree.get_at_path(&path) {
            let mut new_children = children.clone();
            let pos = rng.gen_range(0..=new_children.len());
            new_children.insert(pos, new_leaf);
            Some(CircuitTopology::Series(new_children))
        } else {
            None
        };

    if let Some(new_node) = new_node_opt {
        let candidate = tree.clone().set_at_path(&path, new_node).normalize();
        if candidate.validate().is_ok() && candidate.is_within_limits() {
            return candidate;
        }
    }
    tree
}

/// Wraps a randomly selected subtree in a new Parallel block alongside a new
/// relaxation element, modelling an additional interfacial layer.
fn wrap_node_in_parallel<R: Rng>(tree: CircuitTopology, rng: &mut R) -> CircuitTopology {
    let all_paths = tree.all_paths();
    if all_paths.is_empty() {
        return tree;
    }
    let path = all_paths[rng.gen_range(0..all_paths.len())].clone();

    let existing = match tree.get_at_path(&path) {
        Some(n) => n.clone(),
        None => return tree,
    };

    let relax = LeafKind::relaxation_kinds();
    let new_leaf = CircuitTopology::Leaf(relax[rng.gen_range(0..relax.len())]);
    let new_parallel = CircuitTopology::Parallel(vec![existing, new_leaf]);
    let candidate = tree.clone().set_at_path(&path, new_parallel).normalize();
    if candidate.validate().is_ok() && candidate.is_within_limits() {
        candidate
    } else {
        tree
    }
}

/// Sub-tree Swap mutation: exchanges two independent subtrees within the same
/// chromosome, producing a topologically different circuit of similar complexity.
fn subtree_swap_mutation<R: Rng>(tree: CircuitTopology, rng: &mut R) -> CircuitTopology {
    let all_paths = tree.all_paths();
    if all_paths.len() < 2 {
        return tree;
    }

    for _ in 0..6usize {
        let i = rng.gen_range(0..all_paths.len());
        let j = rng.gen_range(0..all_paths.len());
        if i == j {
            continue;
        }

        let p1 = &all_paths[i];
        let p2 = &all_paths[j];

        // Skip ancestor/descendant pairs.
        if p2.starts_with(p1.as_slice()) || p1.starts_with(p2.as_slice()) {
            continue;
        }

        let sub1 = match tree.get_at_path(p1) {
            Some(n) => n.clone(),
            None => continue,
        };
        let sub2 = match tree.get_at_path(p2) {
            Some(n) => n.clone(),
            None => continue,
        };

        let candidate = tree
            .clone()
            .set_at_path(p1, sub2)
            .set_at_path(p2, sub1)
            .normalize();

        if candidate.validate().is_ok() && candidate.is_within_limits() {
            return candidate;
        }
    }
    tree
}

/// Node Insertion: selects a random node and inserts a new Parallel block
/// containing that node and a new relaxation element as siblings.
/// This allows the circuit to grow in complexity organically.
fn node_insertion<R: Rng>(tree: CircuitTopology, rng: &mut R) -> CircuitTopology {
    let leaf_paths = tree.leaf_paths();
    if leaf_paths.is_empty() {
        return tree;
    }
    let path = leaf_paths[rng.gen_range(0..leaf_paths.len())].clone();

    let existing = match tree.get_at_path(&path) {
        Some(n) => n.clone(),
        None => return tree,
    };
    let new_elem = CircuitTopology::Leaf(random_relaxation(rng));
    // Add a series resistor alongside the relaxation element for physical realism.
    let new_parallel = CircuitTopology::Parallel(vec![existing, new_elem]);
    let candidate = tree.clone().set_at_path(&path, new_parallel).normalize();
    if candidate.validate().is_ok() && candidate.is_within_limits() {
        candidate
    } else {
        tree
    }
}

/// Removes one child from a Series or Parallel node that has at least 3
/// children, simplifying the circuit by one element.
fn prune_leaf<R: Rng>(tree: CircuitTopology, rng: &mut R) -> CircuitTopology {
    let all_paths = tree.all_paths();
    let prunable: Vec<Vec<usize>> = all_paths
        .into_iter()
        .filter(|p| match tree.get_at_path(p) {
            Some(CircuitTopology::Series(c)) | Some(CircuitTopology::Parallel(c)) => c.len() >= 3,
            _ => false,
        })
        .collect();

    if prunable.is_empty() {
        return tree;
    }
    let parent_path = prunable[rng.gen_range(0..prunable.len())].clone();

    let remove_idx = match tree.get_at_path(&parent_path) {
        Some(CircuitTopology::Series(c)) | Some(CircuitTopology::Parallel(c)) => {
            rng.gen_range(0..c.len())
        }
        _ => return tree,
    };

    let new_parent: Option<CircuitTopology> = match tree.get_at_path(&parent_path) {
        Some(CircuitTopology::Series(children)) => {
            let mut c = children.clone();
            c.remove(remove_idx);
            Some(CircuitTopology::Series(c))
        }
        Some(CircuitTopology::Parallel(children)) => {
            let mut c = children.clone();
            c.remove(remove_idx);
            Some(CircuitTopology::Parallel(c))
        }
        _ => None,
    };

    if let Some(new_node) = new_parent {
        let candidate = tree.clone().set_at_path(&parent_path, new_node).normalize();
        if candidate.validate().is_ok() {
            return candidate;
        }
    }
    tree
}

// ─────────────────────────────────────────────────────────────────────────────
// Random element selectors
// ─────────────────────────────────────────────────────────────────────────────

fn random_leaf<R: Rng>(rng: &mut R) -> LeafKind {
    // Uniform draw from the full mutable element pool.
    let all = LeafKind::all();
    all[rng.gen_range(0..all.len())]
}

fn random_relaxation<R: Rng>(rng: &mut R) -> LeafKind {
    // Uniform draw from the relaxation-specific subset used in branch growth.
    let kinds = LeafKind::relaxation_kinds();
    kinds[rng.gen_range(0..kinds.len())]
}
