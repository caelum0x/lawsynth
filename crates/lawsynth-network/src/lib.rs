//! Coupling-structure (network) discovery for LawSynth.
//!
//! Given multi-node time series from a networked dynamical system — `N` nodes,
//! each a state variable evolving as `ẋ_i = F_i(x_i, {x_j : j ∈ neighbours(i)})`
//! — this crate discovers the **directed coupling graph**: which node influences
//! which. It is the network analogue of strong-form SINDy.
//!
//! # Method
//!
//! For each node `i`:
//!
//! 1. The target `ẋ_i` is estimated by numerical differentiation of node `i`'s
//!    column (`lawsynth-differentiate`).
//! 2. A single shared candidate library `Θ` is built over **all** node states
//!    `{x_1 .. x_N}` — self and every candidate neighbour — as a polynomial
//!    expansion of configurable degree (`lawsynth-features`), and reused for
//!    every node.
//! 3. `ẋ_i` is sparsely regressed onto `Θ` (`lawsynth-sparse`, `stlsq`).
//! 4. The adjacency is read from the surviving cross terms: node `j` is a
//!    discovered driver of node `i` (`adjacency[i][j]`, a directed edge `j → i`)
//!    iff some surviving library term involving `x_j` has an aggregated
//!    coefficient magnitude at or above the configured `edge_threshold`. The
//!    per-edge `strength` is reported alongside the boolean adjacency.
//!
//! # Determinism
//!
//! Node order (lexicographic dataset schema), library term order, the derivative
//! estimator, the sparse solve, and the adjacency readout are all deterministic.
//! Identical `(Dataset, NetworkConfig)` inputs yield **bit-identical**
//! [`NetworkModel`] output (verified with `f64::to_bits`).
//!
//! # Honest limits
//!
//! - **Correlational, not causal.** The recovered graph is the structure that
//!   sparse regression attributes to each node's derivative. Confounding or a
//!   common drive can induce a spurious edge; a strong common input shared by two
//!   nodes can look like a direct coupling. This is *not* a causal-inference
//!   guarantee — for that distinction see `lawsynth-causal`.
//! - **Library-bounded.** Only couplings expressible in the chosen polynomial
//!   library and standing above `edge_threshold` are recovered. A coupling of a
//!   form the library cannot represent, or one weaker than the threshold, is
//!   reported as no edge.
//! - **Strong-form noise sensitivity.** Targets come from numerical
//!   differentiation, so heavy observation noise degrades recovery exactly as it
//!   does for strong-form SINDy.
//! - **Excitation.** Recovery needs persistently-exciting, well-sampled dynamics
//!   that keep the candidate columns well conditioned.
//!
//! One node maps to exactly one dataset column; block-structured nodes (a small
//! group of variables per node) are out of scope here. See
//! `specs/network-discovery/README.md` for the full boundary contract.
//!
//! # Example
//!
//! ```no_run
//! use lawsynth_network::{NetworkConfig, discover_network};
//! # use lawsynth_data::Dataset;
//! # fn demo(dataset: &Dataset) -> Result<(), lawsynth_network::NetworkError> {
//! let model = discover_network(dataset, &NetworkConfig::default())?;
//! for (i, node) in model.nodes.iter().enumerate() {
//!     for j in model.drivers_of(i) {
//!         println!("{} drives {} (strength {})", model.nodes[j], node, model.edge_strength(i, j));
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod config;
mod discover;
mod error;
mod library;
mod model;

pub use config::NetworkConfig;
pub use discover::discover_network;
pub use error::NetworkError;
pub use model::{NetworkModel, NodeEquation};
