//! Typed errors for successive-linearization MPC.

use std::fmt;

use lawsynth_expr::EvaluationError;
use lawsynth_feedback::FeedbackError;
use lawsynth_jacobian::JacobianError;
use lawsynth_koopman::KoopmanError;

/// Errors returned by [`mpc_control`](crate::mpc_control) and its configuration.
///
/// Every failure mode is explicit. Boundary violations (bad dimensions,
/// non-finite configuration, empty problem) are distinct from failures that
/// propagate out of the reused numerical stages: linearization
/// ([`JacobianError`]), local LQR design ([`FeedbackError`]), plant/partial
/// evaluation ([`EvaluationError`]), and dense-matrix assembly
/// ([`KoopmanError`]).
#[derive(Clone, Debug, PartialEq)]
pub enum MpcError {
    /// The `states` slice was empty — there is no plant to control.
    EmptyStates,
    /// The `controls` slice was empty — there is no actuation, so no feedback
    /// law can be applied.
    EmptyControls,
    /// A configuration vector or matrix had the wrong length/shape for the
    /// declared state (`n`) or control (`m`) dimension.
    DimensionMismatch {
        /// Which quantity was mis-sized (e.g. `"setpoint"`, `"state_weight"`).
        what: &'static str,
        /// The dimension the quantity was required to have.
        expected: usize,
        /// The dimension it actually had.
        actual: usize,
    },
    /// The integration step `dt` was not a strictly positive, finite number.
    InvalidTimeStep(f64),
    /// The horizon requested zero control steps.
    EmptyHorizon,
    /// A configuration value (setpoint, initial state, control reference, or a
    /// saturation bound) was not finite.
    NonFiniteConfig(&'static str),
    /// A saturation bound was invalid: a `control_min` entry exceeded the
    /// matching `control_max` entry.
    InvalidSaturation {
        /// The control index whose bounds were inconsistent.
        index: usize,
    },
    /// Building or evaluating the analytic linearization failed (missing field
    /// for a state, an unsupported symbolic derivative, or an evaluation error
    /// inside a Jacobian entry).
    Linearization(JacobianError),
    /// The local LQR design failed for the current `(A, B)` (e.g. `R` not
    /// positive definite, an unstabilizable linearization, or non-convergence).
    Design(FeedbackError),
    /// Evaluating the nonlinear field (RK4 plant step) or a control partial at a
    /// numeric point failed (unknown symbol, domain error, non-finite result).
    Evaluation(EvaluationError),
    /// Assembling a dense matrix from evaluated entries failed.
    Matrix(KoopmanError),
}

impl fmt::Display for MpcError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyStates => write!(formatter, "the state vector must be non-empty"),
            Self::EmptyControls => {
                write!(formatter, "the control vector must be non-empty (no actuation)")
            }
            Self::DimensionMismatch { what, expected, actual } => {
                write!(formatter, "{what} has dimension {actual}, expected {expected}")
            }
            Self::InvalidTimeStep(dt) => {
                write!(formatter, "time step must be strictly positive and finite, got {dt}")
            }
            Self::EmptyHorizon => write!(formatter, "the horizon must request at least one step"),
            Self::NonFiniteConfig(what) => {
                write!(formatter, "configuration value '{what}' is not finite")
            }
            Self::InvalidSaturation { index } => {
                write!(formatter, "control_min[{index}] exceeds control_max[{index}]")
            }
            Self::Linearization(error) => write!(formatter, "linearization failed: {error}"),
            Self::Design(error) => write!(formatter, "local LQR design failed: {error}"),
            Self::Evaluation(error) => write!(formatter, "plant evaluation failed: {error}"),
            Self::Matrix(error) => write!(formatter, "matrix assembly failed: {error}"),
        }
    }
}

impl std::error::Error for MpcError {}

impl From<JacobianError> for MpcError {
    fn from(error: JacobianError) -> Self {
        Self::Linearization(error)
    }
}

impl From<FeedbackError> for MpcError {
    fn from(error: FeedbackError) -> Self {
        Self::Design(error)
    }
}

impl From<EvaluationError> for MpcError {
    fn from(error: EvaluationError) -> Self {
        Self::Evaluation(error)
    }
}

impl From<KoopmanError> for MpcError {
    fn from(error: KoopmanError) -> Self {
        Self::Matrix(error)
    }
}
