use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_differentiate::differentiate_dataset_with_config;
use lawsynth_sparse::{RegressionProblem, SparseConfig, stlsq_standardized};

use crate::{
    NetworkConfig, NetworkError, NetworkModel, NodeEquation,
    library::{build_library, evaluate_library, term_node_indices},
};

/// Discovers the directed coupling graph of a networked dynamical system.
///
/// # Pipeline
///
/// 1. Collect the nodes as the dataset's columns in deterministic order and
///    require at least two of them.
/// 2. Build **one** shared candidate library `Θ` over all node states and
///    evaluate it into a single design matrix reused by every node.
/// 3. Differentiate every node column to form the targets `ẋ_i`.
/// 4. For each node `i`, sparsely regress `ẋ_i` onto `Θ`.
/// 5. Read the adjacency: aggregate the magnitudes of the surviving coefficients
///    of every term involving `x_j` into `strength[i][j]`; promote it to
///    `adjacency[i][j]` (edge `j → i`) when it reaches `edge_threshold`.
///
/// # Determinism
///
/// The node order is fixed (lexicographic dataset schema), the library term
/// order is fixed by `lawsynth-features`, the derivative estimator is
/// deterministic, `stlsq_standardized` is deterministic, and the adjacency
/// readout aggregates over deterministically ordered terms and symbols. Identical
/// `(Dataset, NetworkConfig)` inputs therefore yield **bit-identical**
/// [`NetworkModel`] output.
pub fn discover_network(
    dataset: &Dataset,
    config: &NetworkConfig,
) -> Result<NetworkModel, NetworkError> {
    config.validate()?;

    let nodes: Vec<Identifier> = dataset.schema().columns;
    if nodes.len() < 2 {
        return Err(NetworkError::SingleNode(nodes.len()));
    }

    let library = build_library(&nodes, &config.features)?;
    let matrix = evaluate_library(&library, dataset)?;
    let library_terms: Vec<String> = matrix.terms.iter().map(|term| term.name.clone()).collect();
    let term_nodes = term_node_indices(&matrix.terms, &nodes);

    let derivative = differentiate_dataset_with_config(dataset, &config.derivative)?;

    let node_count = nodes.len();
    let mut per_node_terms = Vec::with_capacity(node_count);
    let mut strength = vec![vec![0.0_f64; node_count]; node_count];
    let mut adjacency = vec![vec![false; node_count]; node_count];

    for (i, node) in nodes.iter().enumerate() {
        let target = &derivative
            .columns()
            .get(node)
            .expect("differentiated dataset retains every node column")
            .values;
        let equation = regress_node(node.clone(), &matrix.rows, target, &config.sparse)?;

        // Adjacency readout: every surviving term contributes the magnitude of
        // its coefficient to each node it structurally involves.
        for (term_index, coefficient) in equation.coefficients.iter().enumerate() {
            if *coefficient == 0.0 {
                continue;
            }
            for &j in &term_nodes[term_index] {
                strength[i][j] += coefficient.abs();
            }
        }
        for j in 0..node_count {
            adjacency[i][j] = strength[i][j] > 0.0 && strength[i][j] >= config.edge_threshold;
        }

        per_node_terms.push(equation);
    }

    Ok(NetworkModel { nodes, adjacency, strength, per_node_terms, library_terms })
}

/// Solves the sparse regression `Θ ξ ≈ ẋ_i` for one node derivative.
fn regress_node(
    node: Identifier,
    rows: &[Vec<f64>],
    target: &[f64],
    config: &SparseConfig,
) -> Result<NodeEquation, NetworkError> {
    if rows.len() != target.len() {
        return Err(NetworkError::LengthMismatch { targets: target.len(), rows: rows.len() });
    }
    let problem = RegressionProblem::new(rows.to_vec(), target.to_vec())?;
    let solution = stlsq_standardized(&problem, config)?;
    Ok(NodeEquation {
        node,
        coefficients: solution.coefficients,
        residual_sum_squares: solution.residual_sum_squares,
    })
}

#[cfg(test)]
mod tests {
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn two_node_dataset() -> Dataset {
        let time = (0..8).map(|i| i as f64 * 0.1).collect::<Vec<_>>();
        let x1 = time.iter().map(|t| 0.5 * t).collect::<Vec<_>>();
        let x2 = time.iter().map(|t| 1.0 - 0.2 * t).collect::<Vec<_>>();
        Dataset::new(
            TimeAxis::new(time).unwrap(),
            [NumericColumn::new(id("x1"), x1), NumericColumn::new(id("x2"), x2)],
        )
        .unwrap()
    }

    #[test]
    fn rejects_single_node_datasets() {
        let time = TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap();
        let dataset =
            Dataset::new(time, [NumericColumn::new(id("x1"), vec![0.0, 1.0, 2.0])]).unwrap();
        assert_eq!(
            discover_network(&dataset, &NetworkConfig::default()),
            Err(NetworkError::SingleNode(1))
        );
    }

    #[test]
    fn rejects_invalid_edge_threshold() {
        let dataset = two_node_dataset();
        let config = NetworkConfig { edge_threshold: -0.1, ..NetworkConfig::default() };
        assert!(matches!(
            discover_network(&dataset, &config),
            Err(NetworkError::InvalidThreshold(_))
        ));
    }

    #[test]
    fn propagates_too_few_samples_from_differentiation() {
        let time = TimeAxis::new(vec![0.0]).unwrap();
        let dataset = Dataset::new(
            time,
            [NumericColumn::new(id("x1"), vec![1.0]), NumericColumn::new(id("x2"), vec![2.0])],
        )
        .unwrap();
        assert!(matches!(
            discover_network(&dataset, &NetworkConfig::default()),
            Err(NetworkError::Differentiation(_))
        ));
    }

    #[test]
    fn regress_node_reports_length_mismatch() {
        let rows = vec![vec![1.0, 0.0], vec![1.0, 1.0]];
        let target = vec![0.0, 1.0, 2.0];
        let error = regress_node(id("x1"), &rows, &target, &SparseConfig::default()).unwrap_err();
        assert_eq!(error, NetworkError::LengthMismatch { targets: 3, rows: 2 });
    }

    #[test]
    fn produces_one_equation_per_node_aligned_with_the_library() {
        let dataset = two_node_dataset();
        let model = discover_network(&dataset, &NetworkConfig::default()).unwrap();
        assert_eq!(model.per_node_terms.len(), 2);
        assert_eq!(model.nodes, vec![id("x1"), id("x2")]);
        for equation in &model.per_node_terms {
            assert_eq!(equation.coefficients.len(), model.library_terms.len());
        }
        assert_eq!(model.adjacency.len(), 2);
        assert_eq!(model.strength.len(), 2);
    }
}
