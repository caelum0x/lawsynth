//! Recovery of known coupling graphs from coupled-RK4 trajectories.

mod common;

use common::{dataset_from, id, integrate_multi, integrate_rk4};
use lawsynth_features::FeatureConfig;
use lawsynth_network::{NetworkConfig, discover_network};
use lawsynth_sparse::SparseConfig;

/// Linear config: pure linear coupling library with a small edge threshold and a
/// sparse threshold tuned to keep genuine couplings while dropping numerical
/// noise.
fn linear_config(edge_threshold: f64) -> NetworkConfig {
    NetworkConfig {
        features: FeatureConfig { polynomial_degree: 1, include_constant: true },
        sparse: SparseConfig { threshold: 0.02, max_iterations: 20, ridge: 1e-10 },
        edge_threshold,
        ..NetworkConfig::default()
    }
}

/// Collects the off-diagonal driver set of every node as sorted index vectors.
fn driver_sets(model: &lawsynth_network::NetworkModel) -> Vec<Vec<usize>> {
    (0..model.len()).map(|i| model.drivers_of(i)).collect()
}

#[test]
fn recovers_a_directed_chain_without_spurious_edges() {
    // ẋ1 = −x1, ẋ2 = −x2 + 2·x1, ẋ3 = −x3 + 2·x2  (chain 1 → 2 → 3).
    let c = 2.0;
    let (time, trajectory) = integrate_rk4(&[1.0, 0.5, -0.3], 0.01, 400, |x| {
        vec![-x[0], -x[1] + c * x[0], -x[2] + c * x[1]]
    });
    let dataset = dataset_from(time, &["x1", "x2", "x3"], trajectory);

    let model = discover_network(&dataset, &linear_config(0.1)).unwrap();

    // Self loops on every node.
    for i in 0..3 {
        assert!(model.has_self_loop(i), "node {i} should retain its self term");
    }
    // Exactly the chain edges 1 → 2 and 2 → 3, nothing else.
    assert_eq!(driver_sets(&model), vec![vec![], vec![0], vec![1]]);
    // The crucial no-1→3 guard is implied above, but assert it explicitly.
    assert!(!model.is_edge(2, 0), "x1 must not be reported as a direct driver of x3");
}

#[test]
fn recovers_diffusive_coupling_on_a_ring() {
    // Ring of 4: ẋ_i = k·(x_{i-1} + x_{i+1} − 2·x_i), a graph-Laplacian diffusion.
    // The symmetric ring Laplacian has degenerate eigenvalues, so a single
    // trajectory is rank-deficient; several initial conditions excite all Fourier
    // modes and make the coupling identifiable.
    let k = 0.3;
    let ring = |x: &[f64]| -> Vec<f64> {
        let n = x.len();
        (0..n)
            .map(|i| {
                let left = x[(i + n - 1) % n];
                let right = x[(i + 1) % n];
                k * (left + right - 2.0 * x[i])
            })
            .collect()
    };
    // Generic (non-eigenvector) initial conditions: each excites every Fourier
    // mode, unlike a symmetric IC that would stay within a single mode.
    let initial_conditions = vec![
        vec![1.0, 0.3, -0.7, 0.2],
        vec![-0.5, 1.2, 0.4, -0.9],
        vec![0.8, -1.1, 0.6, 0.1],
        vec![0.2, 0.5, -1.3, 1.0],
    ];
    let (time, trajectory) = integrate_multi(&initial_conditions, 0.01, 150, 1_000.0, ring);
    let dataset = dataset_from(time, &["x1", "x2", "x3", "x4"], trajectory);

    let model = discover_network(&dataset, &linear_config(0.05)).unwrap();

    // Each node's neighbours on the ring are its two ring-adjacent nodes; the
    // opposite node (0↔2, 1↔3) must NOT appear.
    assert_eq!(driver_sets(&model), vec![vec![1, 3], vec![0, 2], vec![1, 3], vec![0, 2]]);
    assert!(!model.is_edge(0, 2), "opposite ring nodes must not couple");
    assert!(!model.is_edge(1, 3), "opposite ring nodes must not couple");
}

#[test]
fn recovers_a_star_hub_with_no_leaf_to_leaf_edges() {
    // Hub (node 0) drives three leaves; leaves have distinct self-rates so their
    // columns stay linearly independent. Leaves never drive each other or the hub.
    let g = [1.5, 1.2, 2.0];
    let a = [1.0, 1.3, 1.7];
    let (time, trajectory) = integrate_rk4(&[1.0, 0.2, -0.1, 0.3], 0.01, 350, |x| {
        vec![
            -0.7 * x[0],
            -a[0] * x[1] + g[0] * x[0],
            -a[1] * x[2] + g[1] * x[0],
            -a[2] * x[3] + g[2] * x[0],
        ]
    });
    // Names chosen so the hub sorts first: "hub" < "x1" < "x2" < "x3".
    let dataset = dataset_from(time, &["hub", "x1", "x2", "x3"], trajectory);
    let model = discover_network(&dataset, &linear_config(0.1)).unwrap();

    let hub = model.node_index(&id("hub")).unwrap();
    assert_eq!(hub, 0);

    // The hub is autonomous: it has no drivers.
    assert_eq!(model.drivers_of(hub), Vec::<usize>::new());
    // Every leaf is driven by the hub and by nobody else.
    for leaf in 1..4 {
        assert_eq!(
            model.drivers_of(leaf),
            vec![hub],
            "leaf {leaf} should be driven only by the hub"
        );
        assert!(model.is_edge(leaf, hub));
    }
    // No leaf drives another leaf, and no leaf drives the hub.
    for i in 1..4 {
        assert!(!model.is_edge(hub, i), "hub must not be driven by a leaf");
        for j in 1..4 {
            if i != j {
                assert!(!model.is_edge(i, j), "leaf {j} must not drive leaf {i}");
            }
        }
    }
}

#[test]
fn decoupled_nodes_yield_no_false_edges() {
    // The false-positive guard: three independent nodes with distinct decay
    // rates and a degree-2 (quadratic + interaction) library. Nothing couples
    // them, so every off-diagonal entry must stay empty even though interaction
    // terms x_i·x_j are candidates.
    let a = [0.7, 1.3, 2.1];
    let decoupled = |x: &[f64]| -> Vec<f64> { vec![-a[0] * x[0], -a[1] * x[1], -a[2] * x[2]] };
    // Several initial conditions keep the (otherwise monotone, correlated)
    // per-node decays well conditioned so no spurious interaction edge is fitted.
    let initial_conditions = vec![vec![1.0, -0.8, 0.5], vec![0.3, 1.0, -0.6], vec![-0.7, 0.2, 0.9]];
    let (time, trajectory) = integrate_multi(&initial_conditions, 0.01, 150, 1_000.0, decoupled);
    let dataset = dataset_from(time, &["x1", "x2", "x3"], trajectory);

    let config = NetworkConfig {
        features: FeatureConfig { polynomial_degree: 2, include_constant: true },
        sparse: SparseConfig { threshold: 0.02, max_iterations: 20, ridge: 1e-10 },
        edge_threshold: 0.05,
        ..NetworkConfig::default()
    };
    let model = discover_network(&dataset, &config).unwrap();

    // No off-diagonal edges anywhere.
    for i in 0..3 {
        assert_eq!(model.drivers_of(i), Vec::<usize>::new(), "node {i} must have no drivers");
        // Each node still retains its own self dynamics.
        assert!(model.has_self_loop(i));
    }
}

#[test]
fn weak_coupling_is_gated_by_the_edge_threshold() {
    // ẋ1 = −x1, ẋ2 = −x2 + c·x1 with a genuinely weak coupling c = 0.08.
    let c = 0.08;
    let (time, trajectory) =
        integrate_rk4(&[1.0, 0.4], 0.01, 300, |x| vec![-x[0], -x[1] + c * x[0]]);
    let dataset = dataset_from(time, &["x1", "x2"], trajectory);

    // A sparse threshold small enough to keep the weak term, so that only the
    // edge threshold decides whether the coupling is reported.
    let base = NetworkConfig {
        features: FeatureConfig { polynomial_degree: 1, include_constant: true },
        sparse: SparseConfig { threshold: 1e-4, max_iterations: 20, ridge: 1e-10 },
        edge_threshold: 0.0,
        ..NetworkConfig::default()
    };

    // High threshold: the weak edge is honestly reported as absent.
    let strict = NetworkConfig { edge_threshold: 0.3, ..base.clone() };
    let strict_model = discover_network(&dataset, &strict).unwrap();
    assert!(!strict_model.is_edge(1, 0), "weak coupling below threshold must be no-edge");

    // The strength is nonetheless measured and small — near the true c.
    let strength = strict_model.edge_strength(1, 0);
    assert!(strength > 0.0 && strength < 0.2, "measured weak strength was {strength}");

    // Low threshold: the same weak edge is recovered.
    let lenient = NetworkConfig { edge_threshold: 0.02, ..base };
    let lenient_model = discover_network(&dataset, &lenient).unwrap();
    assert!(lenient_model.is_edge(1, 0), "lowering the threshold should recover the weak edge");
}
