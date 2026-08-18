//! Bit-identical reproducibility of the discovered network model.

mod common;

use common::{dataset_from, integrate_rk4};
use lawsynth_network::{NetworkConfig, NetworkModel, discover_network};

fn chain_dataset() -> lawsynth_data::Dataset {
    let (time, trajectory) = integrate_rk4(&[1.0, 0.5, -0.3], 0.01, 250, |x| {
        vec![-x[0], -x[1] + 2.0 * x[0], -x[2] + 2.0 * x[1]]
    });
    dataset_from(time, &["x1", "x2", "x3"], trajectory)
}

/// Every `f64` in the model, flattened into a bit pattern for exact comparison.
fn bit_signature(model: &NetworkModel) -> Vec<u64> {
    let mut bits = Vec::new();
    for row in &model.strength {
        for value in row {
            bits.push(value.to_bits());
        }
    }
    for equation in &model.per_node_terms {
        for coefficient in &equation.coefficients {
            bits.push(coefficient.to_bits());
        }
        bits.push(equation.residual_sum_squares.to_bits());
    }
    bits
}

#[test]
fn identical_inputs_produce_bit_identical_models() {
    let dataset = chain_dataset();
    let config = NetworkConfig::default();

    let first = discover_network(&dataset, &config).unwrap();
    let second = discover_network(&dataset, &config).unwrap();

    // Structural equality plus explicit bit-for-bit float equality.
    assert_eq!(first.nodes, second.nodes);
    assert_eq!(first.adjacency, second.adjacency);
    assert_eq!(first.library_terms, second.library_terms);
    assert_eq!(bit_signature(&first), bit_signature(&second));
    // The derived PartialEq agrees with the bitwise check.
    assert_eq!(first, second);
}

#[test]
fn a_freshly_reintegrated_dataset_reproduces_the_same_model() {
    // Regenerating the fixture from scratch must land on the same bits, proving
    // the whole pipeline (integration excluded) is a pure function of the data.
    let config = NetworkConfig::default();
    let first = discover_network(&chain_dataset(), &config).unwrap();
    let second = discover_network(&chain_dataset(), &config).unwrap();
    assert_eq!(bit_signature(&first), bit_signature(&second));
}
