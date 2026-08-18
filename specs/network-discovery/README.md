# Coupling-structure (network) discovery boundary (v2-A)

This directory specifies **deterministic coupling-graph discovery** — the
per-node sparse-regression method implemented in `crates/lawsynth-network`. It is
a **boundary specification** in the house style: it states what a conforming
implementation MUST do, and — crucially — what a discovered edge is and is not
allowed to claim.

## Motivation

Many systems are **networks of interacting units**: `N` nodes, each a state
variable evolving as `ẋ_i = F_i(x_i, {x_j : j ∈ neighbours(i)})`. The scientific
question is often not the exact right-hand side but the **coupling structure** —
which node influences which. Ordinary SINDy fits one autonomous law per variable
over a library of that variable's own dynamics; network discovery generalises
that by building each node's candidate library over **all** node states, so a
term involving a *neighbour's* state can enter the regression. A node `j` is then
read as a driver of node `i` exactly when some library term involving `x_j`
survives the sparse fit of `ẋ_i`. Reading which cross terms survive, across every
node, reconstructs a directed adjacency matrix.

The output is **correlational structure recovered by regression**, not a proof of
mechanism. That distinction is the whole point of the boundary below.

## What a discovered edge IS

A discovered edge `j → i` is a **regression attribution**, never a causal fact.
The contract is:

1. **Per-node sparse regression over the all-nodes library.** For each node `i`
   the target `ẋ_i` (numerically differentiated from column `i`) is sparsely
   regressed onto a single shared library `Θ` built over every node state
   `{x_1 .. x_N}` — self and all candidate neighbours. The same design matrix is
   reused for every node; only the target changes.
2. **Adjacency from surviving cross terms.** After the solve, `adjacency[i][j]`
   is `true` iff some surviving (non-zero) library term that *structurally
   involves* `x_j` has an aggregated coefficient magnitude at or above the
   configured `edge_threshold`. "Structurally involves" is read from the term's
   expression tree (`lawsynth_expr::symbols`), so it cannot be spoofed by a
   misleading term label. The diagonal `adjacency[i][i]` is the node's own self
   term.
3. **Edge strength is reported alongside the boolean.** `strength[i][j]` is the
   sum of the absolute surviving coefficients of every term involving `x_j` in
   node `i`'s equation. It is reported so a caller can see *how strong* each
   coupling is, not merely whether it cleared the threshold. A boolean edge
   without its strength is not a conforming report.
4. **No causal claim.** A discovered edge is the structure regression attributes
   to a node's derivative. It is correlational: a common drive, a confounder, or
   collinear candidate columns can induce a spurious edge, and a genuine coupling
   below the threshold or outside the library is reported as no edge. For causal
   guarantees see `crates/lawsynth-causal`; this crate makes no such claim.

## Requirements

1. **One column per node, at least two nodes.** Each dataset column is a node;
   node order is the dataset's deterministic (lexicographic) schema order.
   Discovery needs at least two nodes — a one-column dataset is ordinary
   single-series dynamics discovery and MUST be rejected. Block-structured nodes
   (a group of variables per node) are out of scope for this boundary.
2. **Shared all-nodes library.** The candidate library MUST be built over the
   full node set using the shared feature machinery (`crates/lawsynth-features`),
   NOT re-implemented, so self, neighbour, quadratic, and interaction terms all
   arise from the same deterministic polynomial expansion. The degree and
   constant-term policy are configurable; degree 1 recovers linear couplings,
   degree ≥ 2 admits nonlinear and product couplings.
3. **Numerically differentiated targets.** Each target `ẋ_i` MUST come from the
   deterministic derivative estimators in `crates/lawsynth-differentiate` applied
   to node `i`'s column. The usual strong-form noise caveat applies: the
   estimator amplifies observation noise, so the discovered structure is only as
   clean as the derivatives.
4. **Deterministic sparse solve.** Each node's regression MUST use the
   deterministic sparse solvers in `crates/lawsynth-sparse` (`stlsq`, applied
   after deterministic feature scaling).
5. **Determinism.** Node order, library term order, the derivative estimator, the
   sparse solve, and the adjacency readout (aggregating over deterministically
   ordered terms and symbols) MUST all be deterministic. Identical
   `(Dataset, NetworkConfig)` inputs MUST produce a **bit-identical**
   `NetworkModel` output. The reference implementation verifies this bit-for-bit
   via `f64::to_bits`.
6. **Two independent thresholds, honestly separated.** The sparse `threshold`
   decides which *library terms* survive the per-node fit; the `edge_threshold`
   decides which *cross couplings* become boolean edges. The implementation MUST
   expose both and MUST document that raising `edge_threshold` trades sensitivity
   for fewer false positives (and vice versa). It MUST NOT silently widen a
   threshold to force or suppress an edge.

## Honest scope & limits

- **Correlational, not causal (the central caveat).** The recovered adjacency is
  regression structure. A strong common input shared by two nodes, or a
  confounding third node, can appear as a direct coupling. This boundary makes
  **no** causal-inference guarantee — `crates/lawsynth-causal` is the place that
  distinction is drawn.
- **Library-bounded.** Only couplings expressible in the chosen library and
  standing above `edge_threshold` are recovered. A coupling of a form the library
  cannot represent, or one weaker than the threshold, is honestly reported as no
  edge. The reference implementation demonstrates both sides of the threshold: a
  weak coupling is suppressed at a high `edge_threshold` and recovered when it is
  lowered, with the measured strength reported throughout.
- **Persistent excitation and conditioning are required.** Recovery needs the
  candidate columns to be well conditioned. A single trajectory of a **symmetric**
  linear network (e.g. a diffusive ring) has degenerate modes and is
  rank-deficient: the coupling is unidentifiable and the solve may spread it
  across non-neighbours. Exciting the network from several initial conditions
  restores full rank — the standard SINDy remedy, and what the reference ring and
  decoupled fixtures do.
- **Strong-form noise sensitivity.** Targets are differentiated from the data, so
  heavy observation noise degrades the recovered structure exactly as it does for
  strong-form SINDy. A weak-form network variant would be the noise-robust
  companion and is future work.
- **No proof.** A discovered graph is a sparse, quantified, threshold-gated fit,
  subject to the excitation, conditioning, noise, and library limits above.

## Public API

```text
discover_network(&Dataset, &NetworkConfig) -> Result<NetworkModel, NetworkError>
```

`NetworkConfig` reuses the feature, derivative, and sparse configuration types of
the crates it drives, plus an `edge_threshold`. `NetworkModel` returns the node
identifiers, the boolean directed `adjacency` (`adjacency[i][j]` ⇒ `j → i`), the
per-edge `strength` matrix, one `NodeEquation` per node (`per_node_terms`, each a
sparse coefficient row over the shared library), and the human-readable
`library_terms`. Helpers `drivers_of(i)`, `is_edge(i, j)`, `edge_strength(i, j)`,
and `has_self_loop(i)` read the graph.

## Reference recovery results

The reference implementation recovers, from coupled fixed-step RK4 trajectories
of **known** graphs:

- **Directed chain `1 → 2 → 3`** (`ẋ1 = −x1`, `ẋ2 = −x2 + 2x1`, `ẋ3 = −x3 + 2x2`):
  exactly edges `1 → 2` and `2 → 3` with self loops, and **no** spurious `1 → 3`.
- **Diffusive ring of 4** (`ẋ_i = k(x_{i-1} + x_{i+1} − 2x_i)`): each node's
  drivers are exactly its two ring neighbours; the opposite node never couples
  (recovered from four initial conditions to break the ring's mode degeneracy).
- **Star / hub of 4**: the hub drives every leaf, no leaf drives another leaf, and
  the autonomous hub has no drivers.
- **Decoupled nodes** (three independent decays, degree-2 library with interaction
  candidates): the false-positive guard — every off-diagonal entry stays empty.

## Non-goals

- No causal-inference claim; that is `crates/lawsynth-causal`.
- No block/multi-variable nodes; one dataset column per node.
- No weak/integral network form and no claim of noise robustness beyond what the
  strong-form derivative estimator provides.
- No trained probe, no stochastic sampling, no network or platform service.
