use lawsynth_core::Identifier;

/// The fitted sparse dynamics for a single node's derivative `ẋ_i`.
///
/// Coefficients are aligned positionally with
/// [`NetworkModel::library_terms`](crate::NetworkModel::library_terms): entry `k`
/// multiplies term `k` of the shared all-nodes library `Θ`. A zero entry means
/// the sparse solve dropped that term.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeEquation {
    /// The node whose derivative this equation predicts.
    pub node: Identifier,
    /// Sparse coefficient row over the shared library, in library-term order.
    pub coefficients: Vec<f64>,
    /// Residual sum of squares `‖Θ ξ − ẋ‖²` of the fit.
    pub residual_sum_squares: f64,
}

impl NodeEquation {
    /// Returns the surviving `(term_label, coefficient)` pairs in library order.
    ///
    /// `labels` must be the model's library terms; only non-zero coefficients are
    /// returned so the result reads as the discovered right-hand side.
    pub fn active_terms<'a>(&self, labels: &'a [String]) -> Vec<(&'a str, f64)> {
        self.coefficients
            .iter()
            .zip(labels)
            .filter(|(coefficient, _)| **coefficient != 0.0)
            .map(|(coefficient, label)| (label.as_str(), *coefficient))
            .collect()
    }
}

/// A discovered directed coupling graph over a set of nodes.
///
/// # Adjacency orientation
///
/// `adjacency[i][j] == true` iff node `j`'s state appears — with a surviving
/// coefficient whose aggregated magnitude reaches the configured edge threshold —
/// in node `i`'s fitted derivative equation. That is, `j` is a discovered
/// **driver** of `i`: a directed influence `j → i` in the dynamics
/// `ẋ_i = F_i(x_i, {x_j})`. The diagonal `adjacency[i][i]` reflects the node's
/// own self term.
///
/// `strength[i][j]` is the aggregate magnitude backing that decision: the sum of
/// the absolute surviving coefficients of every library term that involves
/// `x_j`, read from node `i`'s equation. It is reported alongside the boolean
/// adjacency so a caller can see *how strong* each recovered coupling is, not
/// just whether it cleared the threshold.
///
/// This is **correlational** structure recovered by regression, not a causal
/// guarantee — see the crate docs and `specs/network-discovery/README.md`.
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkModel {
    /// Node identifiers in deterministic (lexicographic) order. Row and column
    /// `i` of every matrix below refer to `nodes[i]`.
    pub nodes: Vec<Identifier>,
    /// Boolean directed adjacency: `adjacency[i][j]` means `j → i` (see type docs).
    pub adjacency: Vec<Vec<bool>>,
    /// Aggregated per-edge coupling strength backing `adjacency`.
    pub strength: Vec<Vec<f64>>,
    /// One fitted equation per node, in `nodes` order.
    pub per_node_terms: Vec<NodeEquation>,
    /// Human-readable labels for every library term, in column order.
    pub library_terms: Vec<String>,
}

impl NetworkModel {
    /// Number of nodes in the network.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the network has no nodes. Always `false` for a discovered model
    /// (discovery requires at least two nodes); provided for completeness.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Looks up the index of a node by identifier.
    pub fn node_index(&self, node: &Identifier) -> Option<usize> {
        self.nodes.iter().position(|candidate| candidate == node)
    }

    /// Whether node `j` is a discovered driver of node `i` (directed edge `j → i`).
    ///
    /// Returns `false` for out-of-range indices.
    pub fn is_edge(&self, i: usize, j: usize) -> bool {
        self.adjacency.get(i).and_then(|row| row.get(j)).copied().unwrap_or(false)
    }

    /// The aggregated coupling strength of the edge `j → i`.
    ///
    /// Returns `0.0` for out-of-range indices.
    pub fn edge_strength(&self, i: usize, j: usize) -> f64 {
        self.strength.get(i).and_then(|row| row.get(j)).copied().unwrap_or(0.0)
    }

    /// Whether node `i` has a discovered self term on its own derivative.
    pub fn has_self_loop(&self, i: usize) -> bool {
        self.is_edge(i, i)
    }

    /// The off-diagonal drivers of node `i`: the node indices `j ≠ i` with a
    /// discovered directed edge `j → i`, in ascending order.
    pub fn drivers_of(&self, i: usize) -> Vec<usize> {
        let Some(row) = self.adjacency.get(i) else {
            return Vec::new();
        };
        row.iter().enumerate().filter(|&(j, edge)| *edge && j != i).map(|(j, _)| j).collect()
    }

    /// Looks up the fitted equation for a given node.
    pub fn equation(&self, node: &Identifier) -> Option<&NodeEquation> {
        self.per_node_terms.iter().find(|equation| &equation.node == node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn sample_model() -> NetworkModel {
        // A hand-built 3-node model encoding the chain 1→2→3 with self loops.
        NetworkModel {
            nodes: vec![id("x1"), id("x2"), id("x3")],
            adjacency: vec![
                vec![true, false, false],
                vec![true, true, false],
                vec![false, true, true],
            ],
            strength: vec![vec![1.0, 0.0, 0.0], vec![2.0, 1.0, 0.0], vec![0.0, 2.0, 1.0]],
            per_node_terms: vec![
                NodeEquation {
                    node: id("x1"),
                    coefficients: vec![-1.0, 0.0, 0.0],
                    residual_sum_squares: 0.0,
                },
                NodeEquation {
                    node: id("x2"),
                    coefficients: vec![2.0, -1.0, 0.0],
                    residual_sum_squares: 0.0,
                },
                NodeEquation {
                    node: id("x3"),
                    coefficients: vec![0.0, 2.0, -1.0],
                    residual_sum_squares: 0.0,
                },
            ],
            library_terms: vec!["x1".into(), "x2".into(), "x3".into()],
        }
    }

    #[test]
    fn reads_edges_and_drivers_in_directed_orientation() {
        let model = sample_model();
        assert_eq!(model.len(), 3);
        // 1 → 2 and 2 → 3, with self loops, and no 1 → 3.
        assert!(model.is_edge(1, 0));
        assert!(model.is_edge(2, 1));
        assert!(!model.is_edge(2, 0));
        assert!(model.has_self_loop(0));
        assert_eq!(model.drivers_of(0), Vec::<usize>::new());
        assert_eq!(model.drivers_of(1), vec![0]);
        assert_eq!(model.drivers_of(2), vec![1]);
    }

    #[test]
    fn out_of_range_lookups_are_safe() {
        let model = sample_model();
        assert!(!model.is_edge(9, 0));
        assert_eq!(model.edge_strength(9, 9), 0.0);
        assert_eq!(model.drivers_of(9), Vec::<usize>::new());
        assert_eq!(model.node_index(&id("x2")), Some(1));
        assert_eq!(model.node_index(&id("absent")), None);
    }

    #[test]
    fn active_terms_report_surviving_labels() {
        let model = sample_model();
        let equation = model.equation(&id("x2")).unwrap();
        let active = equation.active_terms(&model.library_terms);
        assert_eq!(active, vec![("x1", 2.0), ("x2", -1.0)]);
    }
}
