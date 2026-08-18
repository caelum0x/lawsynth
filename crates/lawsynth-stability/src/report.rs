//! The report produced by a stability analysis.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_koopman::Complex;

use crate::classify::Classification;

/// A single located fixed point `x*` with `f(x*) ≈ 0`.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedPoint {
    /// The coordinates `x*` in `states` order.
    pub coordinates: Vec<f64>,
    /// The Jacobian eigenvalues at `x*`, in the eigensolver's canonical order
    /// (descending modulus, ties broken deterministically).
    pub eigenvalues: Vec<Complex>,
    /// The linear-stability verdict at `x*`.
    pub classification: Classification,
}

/// The result of [`crate::analyze_stability`].
///
/// Besides the located fixed points, the report is honest about the search: it
/// records how many seeds were tried and how many converged, so an empty
/// `fixed_points` list can be read as "the search found nothing" rather than
/// "the system has no fixed points".
#[derive(Clone, Debug, PartialEq)]
pub struct StabilityReport {
    /// The state ordering that indexes every fixed point's coordinates.
    pub states: Vec<Identifier>,
    /// The distinct fixed points inside the search box, ordered lexicographically
    /// by coordinate.
    pub fixed_points: Vec<FixedPoint>,
    /// The total number of deterministic seeds attempted.
    pub seeds_total: usize,
    /// How many seeds converged to a fixed point (before de-duplication).
    pub seeds_converged: usize,
}

impl StabilityReport {
    /// The number of distinct fixed points reported.
    pub fn len(&self) -> usize {
        self.fixed_points.len()
    }

    /// Whether the search located no fixed points inside the box.
    pub fn is_empty(&self) -> bool {
        self.fixed_points.is_empty()
    }

    /// A stable textual fingerprint of the whole report, encoding every float by
    /// its `f64` bit pattern. Two runs on identical input MUST produce identical
    /// strings; this is the basis of the determinism guarantee.
    pub fn to_canonical_string(&self) -> String {
        let mut output = String::new();
        output.push_str("states:");
        for state in &self.states {
            output.push_str(state.as_str());
            output.push(',');
        }
        let _ = writeln!(
            output,
            "\nseeds_total={} seeds_converged={}",
            self.seeds_total, self.seeds_converged
        );
        for point in &self.fixed_points {
            output.push_str("fp coords:");
            for coordinate in &point.coordinates {
                let _ = write!(output, "{:016x},", coordinate.to_bits());
            }
            let _ = write!(output, " class={:?} eig:", point.classification);
            for eigenvalue in &point.eigenvalues {
                let _ = write!(
                    output,
                    "{:016x}:{:016x},",
                    eigenvalue.re.to_bits(),
                    eigenvalue.im.to_bits()
                );
            }
            output.push('\n');
        }
        output
    }
}
