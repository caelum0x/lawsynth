use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_expr::symbols;
use lawsynth_features::{FeatureConfig, FeatureLibrary, FeatureMatrix, FeatureTerm};

use crate::NetworkError;

/// Builds the shared candidate library `Θ` over **all** node states.
///
/// The library is a polynomial expansion over the full node set — self and every
/// candidate neighbour — so that a term involving `x_j` can enter *any* node's
/// regression and reveal a coupling `j → i`. Term order is fixed by
/// `lawsynth-features`, which keeps the whole pipeline deterministic.
pub(crate) fn build_library(
    nodes: &[Identifier],
    config: &FeatureConfig,
) -> Result<FeatureLibrary, NetworkError> {
    Ok(FeatureLibrary::polynomial(
        nodes.iter().cloned(),
        config.polynomial_degree,
        config.include_constant,
    )?)
}

/// Materializes `Θ(X)` for the full multi-node dataset, one row per sample.
pub(crate) fn evaluate_library(
    library: &FeatureLibrary,
    dataset: &Dataset,
) -> Result<FeatureMatrix, NetworkError> {
    Ok(library.evaluate(dataset)?)
}

/// Maps each library term to the node indices whose state it involves.
///
/// The mapping is read *structurally* from each term's expression tree via
/// [`lawsynth_expr::symbols`], so it cannot be fooled by a misleading term label:
/// a constant term maps to nothing, a pure `x_j` term to `{j}`, and an
/// interaction `x_i · x_j` to `{i, j}`. Because the library is built only over
/// node identifiers, every symbol resolves to a node.
pub(crate) fn term_node_indices(terms: &[FeatureTerm], nodes: &[Identifier]) -> Vec<Vec<usize>> {
    terms
        .iter()
        .map(|term| {
            symbols(&term.expression)
                .iter()
                .filter_map(|symbol| nodes.iter().position(|node| node == symbol))
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn linear_library_terms_map_to_single_nodes() {
        let nodes = vec![id("x1"), id("x2"), id("x3")];
        let library =
            build_library(&nodes, &FeatureConfig { polynomial_degree: 1, include_constant: true })
                .unwrap();
        let mapping = term_node_indices(library.terms(), &nodes);
        // The polynomial library emits the constant first, then the linear terms
        // in reverse variable order (x3, x2, x1). Whatever the order, each linear
        // term maps to exactly one node and the constant maps to none.
        assert_eq!(mapping.len(), 4);
        assert_eq!(mapping[0], Vec::<usize>::new());
        let mut mapped: Vec<usize> = mapping[1..].iter().map(|term| term[0]).collect();
        mapped.sort_unstable();
        assert_eq!(mapped, vec![0, 1, 2]);
        assert!(mapping[1..].iter().all(|term| term.len() == 1));
    }

    #[test]
    fn interaction_terms_map_to_both_nodes() {
        let nodes = vec![id("x1"), id("x2")];
        let library =
            build_library(&nodes, &FeatureConfig { polynomial_degree: 2, include_constant: false })
                .unwrap();
        let mapping = term_node_indices(library.terms(), &nodes);
        // Degree-2, no constant over {x1, x2}: x1, x2, x1², x1·x2, x2².
        assert!(mapping.contains(&vec![0]));
        assert!(mapping.contains(&vec![1]));
        assert!(mapping.contains(&vec![0, 1]));
    }
}
