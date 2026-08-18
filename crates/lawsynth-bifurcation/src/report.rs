//! The data structures produced by a parameter continuation.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_koopman::Complex;
use lawsynth_stability::{Classification, StabilityReport};

/// The fixed-point picture at a single parameter value.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterSample {
    /// The parameter value `μ` at which the field was analysed.
    pub parameter_value: f64,
    /// The full stability report (fixed points, classifications, eigenvalues,
    /// and the seed accounting) at this `μ`.
    pub report: StabilityReport,
}

/// One fixed point observed at one parameter value, as it sits on a branch.
#[derive(Clone, Debug, PartialEq)]
pub struct BranchPoint {
    /// The parameter value at which this point was observed.
    pub parameter_value: f64,
    /// The fixed-point coordinates in `states` order.
    pub coordinates: Vec<f64>,
    /// The Jacobian eigenvalues at this point, in the eigensolver's canonical order.
    pub eigenvalues: Vec<Complex>,
    /// The linear-stability verdict at this point.
    pub classification: Classification,
}

/// A branch: the continuation of one fixed point across consecutive parameter
/// values, assembled by nearest-coordinate matching.
///
/// A branch's `points` are ordered by ascending `parameter_value`. A branch that
/// cannot be continued (no nearby fixed point at the next value) simply ends; a
/// fixed point that appears with no predecessor starts a new branch. Branches are
/// therefore an *honest* reconstruction, not a claim of global connectivity.
#[derive(Clone, Debug, PartialEq)]
pub struct Branch {
    /// A stable, creation-ordered identifier for the branch.
    pub id: usize,
    /// The points on the branch, ordered by ascending parameter value.
    pub points: Vec<BranchPoint>,
}

/// The family a detected bifurcation belongs to.
///
/// The distinction is exactly what linearization can see: a single real
/// eigenvalue passing through zero, versus a complex-conjugate pair crossing the
/// imaginary axis. Which *specific* zero-eigenvalue bifurcation occurred
/// (saddle-node, transcritical, or pitchfork) is a normal-form question this
/// crate does not answer, so all three are reported as [`BifurcationKind::Fold`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BifurcationKind {
    /// A (near-)real eigenvalue crossed zero: the saddle-node / transcritical /
    /// pitchfork family. Reported generically; distinguishing them needs
    /// normal-form analysis not performed here.
    Fold,
    /// A complex-conjugate pair crossed the imaginary axis with non-zero
    /// imaginary part: a Hopf bifurcation (birth/death of an oscillation).
    Hopf,
}

/// How a critical parameter value was localized within its bracketing interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Localization {
    /// Deterministic bisection on the sign of the dominant eigenvalue's real part
    /// (crossings on a persisting branch).
    BisectionOnRealPart,
    /// Deterministic bisection on fixed-point existence, i.e. the parameter at
    /// which the branch appears/disappears (folds where fixed points collide).
    BisectionOnExistence,
}

/// A detected bifurcation: a parameter value at which a branch changes stability
/// or is born/destroyed through a zero eigenvalue.
#[derive(Clone, Debug, PartialEq)]
pub struct Bifurcation {
    /// The branch on which the event was detected.
    pub branch_id: usize,
    /// The localized critical parameter value `μ*`.
    pub parameter_value: f64,
    /// The bifurcation family (see [`BifurcationKind`]).
    pub kind: BifurcationKind,
    /// How `parameter_value` was localized.
    pub localization: Localization,
    /// The fixed-point coordinates at (or arbitrarily close to) the crossing.
    pub fixed_point: Vec<f64>,
    /// The crossing eigenvalue (the dominant one) at the localized value.
    pub eigenvalue: Complex,
}

/// The complete result of [`crate::continuation`].
#[derive(Clone, Debug, PartialEq)]
pub struct ContinuationReport {
    /// The state ordering that indexes every coordinate vector.
    pub states: Vec<Identifier>,
    /// The swept parameter.
    pub parameter: Identifier,
    /// The per-parameter samples, in ascending parameter order.
    pub samples: Vec<ParameterSample>,
    /// The assembled branches, in creation order.
    pub branches: Vec<Branch>,
    /// The detected bifurcations, ordered by ascending parameter value.
    pub bifurcations: Vec<Bifurcation>,
}

impl ContinuationReport {
    /// The number of assembled branches.
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }

    /// The number of detected bifurcations.
    pub fn bifurcation_count(&self) -> usize {
        self.bifurcations.len()
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
        let _ = writeln!(output, "\nparameter:{}", self.parameter.as_str());

        for sample in &self.samples {
            let _ = write!(output, "sample mu={:016x} ", sample.parameter_value.to_bits());
            output.push_str(&sample.report.to_canonical_string());
        }

        for branch in &self.branches {
            let _ = write!(output, "branch {} ", branch.id);
            for point in &branch.points {
                let _ = write!(output, "@{:016x}[", point.parameter_value.to_bits());
                for coordinate in &point.coordinates {
                    let _ = write!(output, "{:016x},", coordinate.to_bits());
                }
                let _ = write!(output, "]{:?} ", point.classification);
            }
            output.push('\n');
        }

        for bifurcation in &self.bifurcations {
            let _ = write!(
                output,
                "bif branch={} mu={:016x} kind={:?} loc={:?} eig={:016x}:{:016x} fp[",
                bifurcation.branch_id,
                bifurcation.parameter_value.to_bits(),
                bifurcation.kind,
                bifurcation.localization,
                bifurcation.eigenvalue.re.to_bits(),
                bifurcation.eigenvalue.im.to_bits(),
            );
            for coordinate in &bifurcation.fixed_point {
                let _ = write!(output, "{:016x},", coordinate.to_bits());
            }
            output.push_str("]\n");
        }
        output
    }
}
