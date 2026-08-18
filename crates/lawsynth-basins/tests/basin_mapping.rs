//! Integration tests for `lawsynth-basins::map_basins`.
//!
//! Each test drives the public API on a fixture with a known basin structure and
//! asserts the labelled grid, fractions, and honest `Escaped` / `Undetermined`
//! outcomes against the analytic answer.

mod common;

use common::{bistable, damped_oscillator, divergent, duffing, id, pure_saddle};
use lawsynth_basins::{BasinConfig, BasinError, Classification, Label, map_basins};
use lawsynth_stability::StabilityError;

/// A five-point symmetric grid over `[-2, 2]` for the 1-D bistable field.
fn bistable_config() -> BasinConfig {
    BasinConfig::new(vec![(-2.0, 2.0)])
        .with_grid_resolution(5)
        .with_dt(0.01)
        .with_convergence_tolerance(1e-2)
        .with_max_time(40.0)
}

/// A config for the 2-D Duffing double-well over `[-2, 2]²`.
fn duffing_config(resolution: usize) -> BasinConfig {
    BasinConfig::new(vec![(-2.0, 2.0), (-2.0, 2.0)])
        .with_grid_resolution(resolution)
        .with_dt(0.02)
        .with_convergence_tolerance(1e-2)
        .with_max_time(60.0)
        .with_escape_margin(5.0)
}

// --- Bistable 1-D: the headline case -------------------------------------

#[test]
fn bistable_finds_two_stable_attractors_at_plus_minus_one() {
    let (fields, states) = bistable();
    let report = map_basins(&fields, &states, &bistable_config()).unwrap();

    assert_eq!(report.attractors.len(), 2);
    // Lexicographic order: the −1 well first, then the +1 well.
    assert!((report.attractors[0].coordinates[0] + 1.0).abs() < 1e-6);
    assert!((report.attractors[1].coordinates[0] - 1.0).abs() < 1e-6);
    for attractor in &report.attractors {
        assert_eq!(attractor.classification, Classification::StableNode);
    }
}

#[test]
fn bistable_positive_flows_to_plus_negative_to_minus() {
    let (fields, states) = bistable();
    let report = map_basins(&fields, &states, &bistable_config()).unwrap();

    // Grid samples: −2, −1, 0, +1, +2 (row-major, one axis).
    // Attractor 0 is the −1 well, attractor 1 is the +1 well.
    assert_eq!(report.grid_labels[0], Label::Attractor(0)); // x = −2 → −1
    assert_eq!(report.grid_labels[1], Label::Attractor(0)); // x = −1 → −1
    assert_eq!(report.grid_labels[3], Label::Attractor(1)); // x = +1 → +1
    assert_eq!(report.grid_labels[4], Label::Attractor(1)); // x = +2 → +1
}

#[test]
fn bistable_boundary_sits_at_zero_and_is_undetermined() {
    let (fields, states) = bistable();
    let report = map_basins(&fields, &states, &bistable_config()).unwrap();

    // x = 0 is the saddle: the flow stays there and never settles onto a well.
    assert_eq!(report.grid_labels[2], Label::Undetermined);
    // The basin label flips across the boundary (index 1 vs index 3).
    assert_eq!(report.grid_labels[1], Label::Attractor(0));
    assert_eq!(report.grid_labels[3], Label::Attractor(1));
    assert_eq!(report.undetermined, 1);
    assert_eq!(report.escaped, 0);
}

#[test]
fn bistable_fractions_are_symmetric_half_and_half() {
    let (fields, states) = bistable();
    let report = map_basins(&fields, &states, &bistable_config()).unwrap();

    assert_eq!(report.fractions, vec![0.5, 0.5]);
    // Four of the five ICs settle; the saddle point does not.
    assert_eq!(report.settled(), 4);
    assert_eq!(report.total(), 5);
}

#[test]
fn bistable_finer_grid_keeps_the_boundary_at_zero() {
    let (fields, states) = bistable();
    // 21 points across [−2, 2]: samples land on 0 at the centre (index 10).
    let config = bistable_config().with_grid_resolution(21);
    let report = map_basins(&fields, &states, &config).unwrap();

    // Everything left of 0 goes to the −1 well, everything right to +1.
    for (index, label) in report.grid_labels.iter().enumerate() {
        match index {
            i if i < 10 => assert_eq!(*label, Label::Attractor(0), "index {i}"),
            10 => assert_eq!(*label, Label::Undetermined),
            i => assert_eq!(*label, Label::Attractor(1), "index {i}"),
        }
    }
    assert_eq!(report.fractions, vec![0.5, 0.5]);
}

// --- 2-D Duffing double-well ---------------------------------------------

#[test]
fn duffing_recovers_two_stable_spiral_basins() {
    let (fields, states) = duffing();
    let report = map_basins(&fields, &states, &duffing_config(11)).unwrap();

    assert_eq!(report.attractors.len(), 2);
    for attractor in &report.attractors {
        assert_eq!(attractor.classification, Classification::StableSpiral);
    }
    // Both basins are actually populated.
    assert!(report.grid_labels.contains(&Label::Attractor(0)));
    assert!(report.grid_labels.contains(&Label::Attractor(1)));
    // The symmetric field splits its settled mass evenly.
    assert_eq!(report.fractions, vec![0.5, 0.5]);
}

#[test]
fn duffing_near_well_points_settle_into_that_well() {
    let (fields, states) = duffing();
    let report = map_basins(&fields, &states, &duffing_config(5)).unwrap();

    // Row-major over samples −2,−1,0,1,2 (x slowest); attractor 0 = −1 well.
    let at = |xi: usize, yi: usize| report.grid_labels[xi * 5 + yi];
    // The wells themselves (zero velocity) map to their own attractor.
    assert_eq!(at(3, 2), Label::Attractor(1)); // (+1, 0)
    assert_eq!(at(1, 2), Label::Attractor(0)); // (−1, 0)
    // Points sitting in a well with a small aligned velocity stay in it.
    assert_eq!(at(3, 3), Label::Attractor(1)); // (+1, +1)
    assert_eq!(at(1, 1), Label::Attractor(0)); // (−1, −1)
}

// --- Single global attractor ---------------------------------------------

#[test]
fn single_global_attractor_captures_every_settled_ic() {
    let (fields, states) = damped_oscillator();
    let config = BasinConfig::new(vec![(-2.0, 2.0), (-2.0, 2.0)])
        .with_grid_resolution(7)
        .with_dt(0.02)
        .with_convergence_tolerance(1e-2)
        .with_max_time(80.0)
        .with_escape_margin(3.0);
    let report = map_basins(&fields, &states, &config).unwrap();

    assert_eq!(report.attractors.len(), 1);
    assert_eq!(report.attractors[0].classification, Classification::StableSpiral);
    assert_eq!(report.escaped, 0);
    assert_eq!(report.undetermined, 0);
    assert_eq!(report.fractions, vec![1.0]);
    assert!(report.grid_labels.iter().all(|label| *label == Label::Attractor(0)));
}

// --- Escape / divergence --------------------------------------------------

#[test]
fn divergent_field_labels_runaway_ics_escaped_not_misclassified() {
    let (fields, states) = divergent();
    let config = BasinConfig::new(vec![(-2.0, 2.0)])
        .with_grid_resolution(5)
        .with_dt(0.01)
        .with_max_time(20.0)
        .with_escape_margin(1.0);
    let report = map_basins(&fields, &states, &config).unwrap();

    // The lone fixed point is unstable, so there is no attractor.
    assert!(report.attractors.is_empty());
    assert!(report.fractions.is_empty());
    // Samples −2,−1,0,1,2: everything but the origin runs off; the origin is a
    // fixed point but not an attractor, so it never settles.
    assert_eq!(report.grid_labels[0], Label::Escaped);
    assert_eq!(report.grid_labels[1], Label::Escaped);
    assert_eq!(report.grid_labels[2], Label::Undetermined); // x = 0 stays put
    assert_eq!(report.grid_labels[3], Label::Escaped);
    assert_eq!(report.grid_labels[4], Label::Escaped);
    assert_eq!(report.escaped, 4);
    assert_eq!(report.undetermined, 1);
    // No trajectory was ever mislabelled with a (non-existent) attractor.
    assert!(report.grid_labels.iter().all(|label| !matches!(label, Label::Attractor(_))));
}

// --- No stable attractor: honest empty report -----------------------------

#[test]
fn no_stable_attractor_yields_an_honest_empty_report() {
    let (fields, states) = pure_saddle();
    let config = BasinConfig::new(vec![(-2.0, 2.0), (-2.0, 2.0)])
        .with_grid_resolution(5)
        .with_dt(0.01)
        .with_max_time(20.0)
        .with_escape_margin(1.0);
    let report = map_basins(&fields, &states, &config).unwrap();

    // A saddle is not an attractor: no basins, no fractions.
    assert!(report.attractors.is_empty());
    assert!(report.fractions.is_empty());
    assert_eq!(report.settled(), 0);
    // Every IC is either escaped (x-direction runs off) or undetermined.
    assert!(
        report
            .grid_labels
            .iter()
            .all(|label| matches!(label, Label::Escaped | Label::Undetermined))
    );
    assert_eq!(report.escaped + report.undetermined, report.total());
}

// --- Determinism ----------------------------------------------------------

#[test]
fn identical_inputs_produce_a_bit_identical_report() {
    let (fields, states) = duffing();
    let config = duffing_config(7);
    let first = map_basins(&fields, &states, &config).unwrap();
    let second = map_basins(&fields, &states, &config).unwrap();

    assert_eq!(first, second);
    assert_eq!(first.to_canonical_string(), second.to_canonical_string());
    // Fractions and coordinates agree down to their bit patterns.
    for (a, b) in first.fractions.iter().zip(&second.fractions) {
        assert_eq!(a.to_bits(), b.to_bits());
    }
    for (a, b) in first.attractors.iter().zip(&second.attractors) {
        for (x, y) in a.coordinates.iter().zip(&b.coordinates) {
            assert_eq!(x.to_bits(), y.to_bits());
        }
    }
}

// --- Error paths ----------------------------------------------------------

#[test]
fn empty_state_space_is_a_typed_error() {
    let config = BasinConfig::new(vec![]);
    let error = map_basins(&[], &[], &config).unwrap_err();
    assert_eq!(error, BasinError::EmptyStateSpace);
}

#[test]
fn dimension_mismatch_is_a_typed_error() {
    let (fields, states) = bistable();
    // One state, but a two-dimensional search box.
    let config = BasinConfig::new(vec![(-1.0, 1.0), (-1.0, 1.0)]);
    let error = map_basins(&fields, &states, &config).unwrap_err();
    assert_eq!(error, BasinError::DimensionMismatch { states: 1, search_box: 2 });
}

#[test]
fn a_non_autonomous_field_is_an_unknown_symbol_error() {
    // ẋ = x − a: the free parameter `a` is not one of the states.
    let x = id("x");
    let a = id("a");
    let field = lawsynth_expr::Expr::difference(
        lawsynth_expr::Expr::symbol(x.clone()),
        lawsynth_expr::Expr::symbol(a.clone()),
    );
    let fields = vec![(x.clone(), field)];
    let config = BasinConfig::new(vec![(-1.0, 1.0)]);
    let error = map_basins(&fields, &[x], &config).unwrap_err();
    assert_eq!(error, BasinError::Stability(StabilityError::UnknownSymbol(a)));
}

#[test]
fn an_inverted_search_interval_is_a_typed_error() {
    let (fields, states) = bistable();
    let config = BasinConfig::new(vec![(2.0, -2.0)]);
    let error = map_basins(&fields, &states, &config).unwrap_err();
    assert!(matches!(error, BasinError::InvalidSearchInterval { index: 0, .. }));
}
