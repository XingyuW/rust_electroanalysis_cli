//! Tree-based circuit candidate representation for the genetic algorithm.
//!
//! This module defines the circuit genome as a **recursive tree structure**
//! (`CircuitTopology`) rather than a fixed-length byte array. Each node in
//! the tree is either a **leaf** (a single electrochemical element such as
//! R, CPE, TLMQ, T, Gs) or an **internal** Series / Parallel combinator.

use super::circuits::CircuitNode;
use super::elements::ElementType;
use super::parse_circuit_string;
use crate::domain::FittingError;

/// Circuit genome: the UTF-8 bytes of the canonical circuit string.
pub type CircuitGenome = Vec<u8>;

/// The canonical Randles-cell circuit string.
pub const RANDLES_SEED_CIRCUIT: &str = "R0-p(CPE1,R2)";

// ─────────────────────────────────────────────────────────────────────────────
// LeafKind
// ─────────────────────────────────────────────────────────────────────────────

/// All circuit element types available as leaves in a CircuitTopology tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeafKind {
    R,
    C,
    L,
    W,
    Cpe,
    Wo,
    Ws,
    La,
    Gw,
    G,
    Gs,
    Zarc,
    Tlmq,
    T,
}

impl LeafKind {
    /// Return the canonical token used when serializing this element kind into
    /// a circuit string.
    pub fn token(self) -> &'static str {
        match self {
            LeafKind::R => "R",
            LeafKind::C => "C",
            LeafKind::L => "L",
            LeafKind::W => "W",
            LeafKind::Cpe => "CPE",
            LeafKind::Wo => "Wo",
            LeafKind::Ws => "Ws",
            LeafKind::La => "La",
            LeafKind::Gw => "Gw",
            LeafKind::G => "G",
            LeafKind::Gs => "Gs",
            LeafKind::Zarc => "Zarc",
            LeafKind::Tlmq => "TLMQ",
            LeafKind::T => "T",
        }
    }

    /// Enumerate all supported mutable leaf kinds for GA operators.
    pub const fn all() -> [LeafKind; 14] {
        [
            LeafKind::R,
            LeafKind::C,
            LeafKind::L,
            LeafKind::W,
            LeafKind::Cpe,
            LeafKind::Wo,
            LeafKind::Ws,
            LeafKind::La,
            LeafKind::Gw,
            LeafKind::G,
            LeafKind::Gs,
            LeafKind::Zarc,
            LeafKind::Tlmq,
            LeafKind::T,
        ]
    }

    /// Subset of kinds typically used to represent diffusion tails.
    pub const fn diffusion_kinds() -> [LeafKind; 5] {
        [
            LeafKind::W,
            LeafKind::Gw,
            LeafKind::Wo,
            LeafKind::Ws,
            LeafKind::Gs,
        ]
    }

    /// Subset of kinds used for relaxation/interface branches.
    pub const fn relaxation_kinds() -> [LeafKind; 6] {
        [
            LeafKind::Cpe,
            LeafKind::C,
            LeafKind::G,
            LeafKind::Gs,
            LeafKind::Zarc,
            LeafKind::Tlmq,
        ]
    }

    /// Map a parsed [`ElementType`] into a mutable genome leaf kind.
    ///
    /// `ElementType::K` is intentionally excluded because it is specific to
    /// Lin-KK validation models and not part of the search topology pool.
    fn from_element_type(et: &ElementType) -> Option<Self> {
        Some(match et {
            ElementType::R => LeafKind::R,
            ElementType::C => LeafKind::C,
            ElementType::L => LeafKind::L,
            ElementType::W => LeafKind::W,
            ElementType::Cpe => LeafKind::Cpe,
            ElementType::Wo => LeafKind::Wo,
            ElementType::Ws => LeafKind::Ws,
            ElementType::La => LeafKind::La,
            ElementType::Gw => LeafKind::Gw,
            ElementType::G => LeafKind::G,
            ElementType::Gs => LeafKind::Gs,
            ElementType::Zarc => LeafKind::Zarc,
            ElementType::Tlmq => LeafKind::Tlmq,
            ElementType::T => LeafKind::T,
            ElementType::K => return None,
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CircuitTopology
// ─────────────────────────────────────────────────────────────────────────────

/// Recursive tree representing a circuit topology without parameter indices.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CircuitTopology {
    Leaf(LeafKind),
    Series(Vec<CircuitTopology>),
    Parallel(Vec<CircuitTopology>),
}

pub const MAX_TREE_DEPTH: usize = 5;
pub const MAX_TREE_LEAVES: usize = 10;

impl CircuitTopology {
    // ── Serialisation ──────────────────────────────────────────────────────

    /// Serialize a topology into the canonical numbered circuit syntax.
    ///
    /// Leaves are numbered in traversal order so equivalent trees have a
    /// stable string representation after normalization.
    pub fn to_circuit_string(&self) -> String {
        let mut counter = 0usize;
        self.serialize(&mut counter)
    }

    /// Internal recursive serializer used by [`to_circuit_string`].
    fn serialize(&self, counter: &mut usize) -> String {
        match self {
            CircuitTopology::Leaf(kind) => {
                let s = format!("{}{}", kind.token(), *counter);
                *counter += 1;
                s
            }
            CircuitTopology::Series(children) => children
                .iter()
                .map(|c| c.serialize(counter))
                .collect::<Vec<_>>()
                .join("-"),
            CircuitTopology::Parallel(children) => {
                let parts: Vec<_> = children.iter().map(|c| c.serialize(counter)).collect();
                format!("p({})", parts.join(","))
            }
        }
    }

    // ── Construction ────────────────────────────────────────────────────────

    /// Build a topology tree from a parsed circuit AST node.
    pub fn from_circuit_node(node: &CircuitNode) -> Self {
        match node {
            CircuitNode::Element(etype, _, _) => LeafKind::from_element_type(etype)
                .map(CircuitTopology::Leaf)
                .unwrap_or_else(|| CircuitTopology::Leaf(LeafKind::R)),
            CircuitNode::Series(children) => {
                let kids: Vec<_> = children.iter().map(Self::from_circuit_node).collect();
                if kids.len() == 1 {
                    kids.into_iter().next().unwrap()
                } else {
                    CircuitTopology::Series(kids)
                }
            }
            CircuitNode::Parallel(children) => {
                let kids: Vec<_> = children.iter().map(Self::from_circuit_node).collect();
                if kids.len() == 1 {
                    kids.into_iter().next().unwrap()
                } else {
                    CircuitTopology::Parallel(kids)
                }
            }
        }
    }

    // ── Structural queries ──────────────────────────────────────────────────

    /// Return maximum nesting depth of this topology tree.
    pub fn depth(&self) -> usize {
        match self {
            CircuitTopology::Leaf(_) => 1,
            CircuitTopology::Series(c) | CircuitTopology::Parallel(c) => {
                1 + c.iter().map(Self::depth).max().unwrap_or(0)
            }
        }
    }

    /// Return total leaf count in this topology tree.
    pub fn leaf_count(&self) -> usize {
        match self {
            CircuitTopology::Leaf(_) => 1,
            CircuitTopology::Series(c) | CircuitTopology::Parallel(c) => {
                c.iter().map(Self::leaf_count).sum()
            }
        }
    }

    /// Enforce search-space limits for mutation/crossover outputs.
    pub fn is_within_limits(&self) -> bool {
        self.depth() <= MAX_TREE_DEPTH && self.leaf_count() <= MAX_TREE_LEAVES
    }

    // ── Normalisation ───────────────────────────────────────────────────────

    /// Normalize topology shape by flattening nested series and removing
    /// one-child wrappers.
    pub fn normalize(self) -> Self {
        match self {
            CircuitTopology::Leaf(_) => self,
            CircuitTopology::Series(children) => {
                let normed: Vec<_> = children.into_iter().map(Self::normalize).collect();
                let mut flat = Vec::with_capacity(normed.len());
                for child in normed {
                    match child {
                        CircuitTopology::Series(inner) => flat.extend(inner),
                        other => flat.push(other),
                    }
                }
                if flat.len() == 1 {
                    flat.into_iter().next().unwrap()
                } else {
                    CircuitTopology::Series(flat)
                }
            }
            CircuitTopology::Parallel(children) => {
                let normed: Vec<_> = children.into_iter().map(Self::normalize).collect();
                if normed.len() == 1 {
                    normed.into_iter().next().unwrap()
                } else {
                    CircuitTopology::Parallel(normed)
                }
            }
        }
    }

    // ── Path operations ─────────────────────────────────────────────────────

    /// Return all valid address paths in the tree including the root path.
    pub fn all_paths(&self) -> Vec<Vec<usize>> {
        let mut paths = vec![vec![]];
        self.collect_paths(&[], &mut paths);
        paths
    }

    /// Return paths that resolve to leaf nodes only.
    pub fn leaf_paths(&self) -> Vec<Vec<usize>> {
        let mut paths = Vec::new();
        self.collect_leaf_paths(&[], &mut paths);
        paths
    }

    /// Recursive collector backing [`all_paths`].
    fn collect_paths(&self, current: &[usize], out: &mut Vec<Vec<usize>>) {
        if let CircuitTopology::Series(c) | CircuitTopology::Parallel(c) = self {
            for (i, child) in c.iter().enumerate() {
                let mut path = current.to_vec();
                path.push(i);
                out.push(path.clone());
                child.collect_paths(&path, out);
            }
        }
    }

    /// Recursive collector backing [`leaf_paths`].
    fn collect_leaf_paths(&self, current: &[usize], out: &mut Vec<Vec<usize>>) {
        match self {
            CircuitTopology::Leaf(_) => out.push(current.to_vec()),
            CircuitTopology::Series(c) | CircuitTopology::Parallel(c) => {
                for (i, child) in c.iter().enumerate() {
                    let mut path = current.to_vec();
                    path.push(i);
                    child.collect_leaf_paths(&path, out);
                }
            }
        }
    }

    /// Resolve an immutable reference to the node at `path`.
    pub fn get_at_path(&self, path: &[usize]) -> Option<&CircuitTopology> {
        if path.is_empty() {
            return Some(self);
        }
        match self {
            CircuitTopology::Leaf(_) => None,
            CircuitTopology::Series(c) | CircuitTopology::Parallel(c) => {
                c.get(path[0])?.get_at_path(&path[1..])
            }
        }
    }

    /// Return a new topology with the node at `path` replaced by `new_node`.
    pub fn set_at_path(self, path: &[usize], new_node: CircuitTopology) -> CircuitTopology {
        if path.is_empty() {
            return new_node;
        }
        match self {
            CircuitTopology::Leaf(_) => self,
            CircuitTopology::Series(mut children) => {
                let idx = path[0];
                if idx < children.len() {
                    let child = children.remove(idx);
                    children.insert(idx, child.set_at_path(&path[1..], new_node));
                }
                CircuitTopology::Series(children)
            }
            CircuitTopology::Parallel(mut children) => {
                let idx = path[0];
                if idx < children.len() {
                    let child = children.remove(idx);
                    children.insert(idx, child.set_at_path(&path[1..], new_node));
                }
                CircuitTopology::Parallel(children)
            }
        }
    }

    // ── Validation ──────────────────────────────────────────────────────────

    /// Validate serializability/parsability by round-tripping through the
    /// existing circuit parser.
    pub fn validate(&self) -> Result<(), FittingError> {
        parse_circuit_string(&self.to_circuit_string()).map(|_| ())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CircuitCandidate
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CircuitCandidate {
    pub topology: CircuitTopology,
}

impl CircuitCandidate {
    /// Construct the canonical Randles seed candidate.
    pub fn randles() -> Self {
        Self {
            topology: randles_topology(),
        }
    }

    /// Serialize this candidate to canonical circuit-string form.
    pub fn to_circuit_string(&self) -> String {
        self.topology.to_circuit_string()
    }

    /// Normalize topology for deterministic comparison and caching.
    pub fn normalize(self) -> Self {
        Self {
            topology: self.topology.normalize(),
        }
    }

    /// Validate candidate topology via parser round-trip.
    pub fn validate(&self) -> Result<(), FittingError> {
        self.topology.validate()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Genome encode / decode
// ─────────────────────────────────────────────────────────────────────────────

/// Encode a candidate to genome bytes.
pub fn genome_from_candidate(candidate: &CircuitCandidate) -> CircuitGenome {
    candidate.to_circuit_string().into_bytes()
}

/// Decode genome bytes into a candidate, defaulting to Randles when decode
/// fails.
pub fn candidate_from_genome(genome: &[u8]) -> CircuitCandidate {
    topology_from_genome(genome)
        .map(|topology| CircuitCandidate { topology })
        .unwrap_or_else(CircuitCandidate::randles)
}

/// Canonicalize arbitrary genome bytes by decode-normalize-reencode.
pub fn normalize_genome(genome: CircuitGenome) -> CircuitGenome {
    genome_from_candidate(&candidate_from_genome(&genome))
}

/// Decode genome bytes into a validated normalized topology.
pub fn topology_from_genome(genome: &[u8]) -> Option<CircuitTopology> {
    let s = std::str::from_utf8(genome).ok()?;
    let node = parse_circuit_string(s).ok()?;
    let topology = CircuitTopology::from_circuit_node(&node).normalize();
    if topology.validate().is_ok() {
        Some(topology)
    } else {
        None
    }
}

/// Encode a topology tree into genome bytes.
pub fn genome_from_topology(topology: &CircuitTopology) -> CircuitGenome {
    topology.to_circuit_string().into_bytes()
}

// ─────────────────────────────────────────────────────────────────────────────
// Seed population
// ─────────────────────────────────────────────────────────────────────────────

/// Return curated seed candidates used for initial population priming.
pub fn seed_candidates() -> Vec<CircuitCandidate> {
    seed_topologies()
        .into_iter()
        .map(|topology| CircuitCandidate { topology })
        .collect()
}

/// Return curated seed genomes used by the GA population builder.
pub fn seed_genomes() -> Vec<CircuitGenome> {
    seed_topologies().iter().map(genome_from_topology).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal topology builders
// ─────────────────────────────────────────────────────────────────────────────

/// Canonical Randles topology used as baseline seed and fallback.
pub(crate) fn randles_topology() -> CircuitTopology {
    CircuitTopology::Series(vec![
        CircuitTopology::Leaf(LeafKind::R),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(LeafKind::Cpe),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
    ])
}

/// Randles baseline with one extra diffusion tail element.
fn randles_with_tail(tail: LeafKind) -> CircuitTopology {
    CircuitTopology::Series(vec![
        CircuitTopology::Leaf(LeafKind::R),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(LeafKind::Cpe),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
        CircuitTopology::Leaf(tail),
    ])
}

/// Two-arc circuit with configurable shunt relaxation branch.
fn double_arc(shunt: LeafKind) -> CircuitTopology {
    CircuitTopology::Series(vec![
        CircuitTopology::Leaf(LeafKind::R),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(LeafKind::Cpe),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(shunt),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
    ])
}

/// Two-arc topology plus a diffusion tail branch.
fn double_arc_with_tail(shunt: LeafKind, tail: LeafKind) -> CircuitTopology {
    CircuitTopology::Series(vec![
        CircuitTopology::Leaf(LeafKind::R),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(LeafKind::Cpe),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(shunt),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
        CircuitTopology::Leaf(tail),
    ])
}

/// Nested film-style topology seed.
fn nested_with_film() -> CircuitTopology {
    CircuitTopology::Series(vec![
        CircuitTopology::Leaf(LeafKind::R),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Parallel(vec![
                CircuitTopology::Leaf(LeafKind::Cpe),
                CircuitTopology::Leaf(LeafKind::R),
            ]),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
    ])
}

/// TLMQ-based topology seed.
fn tlmq_circuit() -> CircuitTopology {
    CircuitTopology::Series(vec![
        CircuitTopology::Leaf(LeafKind::R),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(LeafKind::Tlmq),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
    ])
}

/// Porous-electrode T-element topology seed.
fn t_circuit() -> CircuitTopology {
    CircuitTopology::Series(vec![
        CircuitTopology::Leaf(LeafKind::R),
        CircuitTopology::Parallel(vec![
            CircuitTopology::Leaf(LeafKind::T),
            CircuitTopology::Leaf(LeafKind::R),
        ]),
    ])
}

/// Curated diverse topology seed set for population initialization.
fn seed_topologies() -> Vec<CircuitTopology> {
    vec![
        randles_topology(),
        randles_with_tail(LeafKind::W),
        randles_with_tail(LeafKind::Gw),
        randles_with_tail(LeafKind::Wo),
        randles_with_tail(LeafKind::Ws),
        double_arc(LeafKind::Cpe),
        double_arc(LeafKind::G),
        double_arc(LeafKind::Gs),
        double_arc(LeafKind::Zarc),
        double_arc_with_tail(LeafKind::Cpe, LeafKind::W),
        double_arc_with_tail(LeafKind::Cpe, LeafKind::Gw),
        double_arc_with_tail(LeafKind::Zarc, LeafKind::Wo),
        double_arc_with_tail(LeafKind::G, LeafKind::Gw),
        double_arc_with_tail(LeafKind::Gs, LeafKind::Ws),
        nested_with_film(),
        tlmq_circuit(),
        t_circuit(),
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn randles_roundtrip() {
        let candidate = CircuitCandidate::randles();
        let genome = genome_from_candidate(&candidate);
        let decoded = candidate_from_genome(&genome);
        assert_eq!(decoded.to_circuit_string(), candidate.to_circuit_string());
    }

    #[test]
    fn all_seeds_validate() {
        for candidate in seed_candidates() {
            assert!(
                candidate.validate().is_ok(),
                "seed failed: {}",
                candidate.to_circuit_string()
            );
        }
    }

    #[test]
    fn topology_path_set_get() {
        let topo = randles_topology();
        let leaf_paths = topo.leaf_paths();
        assert!(!leaf_paths.is_empty());
        let path = &leaf_paths[0];
        let new_leaf = CircuitTopology::Leaf(LeafKind::Zarc);
        let modified = topo.clone().set_at_path(path, new_leaf.clone());
        assert_eq!(modified.get_at_path(path), Some(&new_leaf));
    }

    #[test]
    fn normalize_unwraps_single_child() {
        let s = CircuitTopology::Series(vec![CircuitTopology::Leaf(LeafKind::R)]);
        assert_eq!(s.normalize(), CircuitTopology::Leaf(LeafKind::R));
    }

    #[test]
    fn normalize_flattens_nested_series() {
        let nested = CircuitTopology::Series(vec![
            CircuitTopology::Leaf(LeafKind::R),
            CircuitTopology::Series(vec![
                CircuitTopology::Leaf(LeafKind::Cpe),
                CircuitTopology::Leaf(LeafKind::W),
            ]),
        ]);
        let flat = nested.normalize();
        assert_eq!(
            flat,
            CircuitTopology::Series(vec![
                CircuitTopology::Leaf(LeafKind::R),
                CircuitTopology::Leaf(LeafKind::Cpe),
                CircuitTopology::Leaf(LeafKind::W),
            ])
        );
    }
}
