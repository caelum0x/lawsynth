//! Deterministic state estimation for LawSynth's discovered linear models.
//!
//! Given a linear(ized) model `ẋ = A x + B u`, `y = C x` — with `A` typically a
//! Jacobian at a fixed point and `C` the output map selecting the measured
//! states — this crate designs and runs a **state estimator** that reconstructs
//! the *full* state from *partial*, *noisy* measurements.
//!
//! An estimator runs a copy of the model corrected by the innovation
//! `y − C x̂`, `x̂̇ = A x̂ + B u + L (y − C x̂)`, so the estimation error
//! `e = x − x̂` obeys `ė = (A − L C) e`. Choosing the gain `L` to shape
//! `A − L C` is the **exact dual** of choosing a feedback gain `K` to shape
//! `A − B K`. This crate therefore *reuses* `lawsynth-feedback` rather than
//! re-deriving the numerics:
//!
//! - [`design_observer`] places the error poles exactly by calling
//!   `place_poles` on the dual pair `(Aᵀ, Cᵀ)` (single-output / SISO Ackermann),
//!   returning `L = place_poles(Aᵀ, Cᵀ, desired)ᵀ`. It requires **observability**
//!   (the dual of controllability) and reports the realized error spectrum via
//!   the shared `lawsynth-koopman` eigensolver.
//! - [`kalman_filter`] computes the optimal steady-state gain by calling `lqr` on
//!   `(Aᵀ, Cᵀ)`: the feedback CARE `AᵀP + PA − PBR⁻¹BᵀP + Q = 0` becomes the
//!   **filter** CARE `A P + P Aᵀ − P Cᵀ R⁻¹ C P + Q = 0`, so the returned Riccati
//!   matrix is the error covariance `P` and `L = (lqr gain)ᵀ = P Cᵀ R⁻¹`.
//! - [`run_observer`] integrates the plant and the observer in lockstep with a
//!   fixed-step RK4, optionally adding seeded Gaussian measurement noise, and
//!   returns both trajectories plus the estimation error over time — the numeric
//!   demonstration that `x̂ → x`.
//!
//! Everything is **deterministic, offline, and std-only**: identical inputs
//! yield bit-identical gains, covariance, and trajectories. Any measurement
//! noise comes from the project's seeded SplitMix64 generator, never the wall
//! clock. `Matrix` and `Complex` are re-exported from `lawsynth-koopman` so a
//! caller builds `(A, B, C)` and reads error poles without a separate
//! dependency.
//!
//! See `specs/state-estimation/README.md` for the boundary specification and its
//! honest limits.

mod error;
mod linalg;
mod noise;
mod observer;
mod simulate;

pub use error::EstimateError;
pub use noise::MeasurementNoise;
pub use observer::{
    Observer, ObserverMethod, design_observer, is_observable, kalman_filter, observability_matrix,
};
pub use simulate::{EstimateTrajectory, run_observer};

// Re-export the shared linear-algebra types so callers can build `(A, B, C)` and
// read error poles / covariance without a direct dependency on `lawsynth-koopman`.
pub use lawsynth_koopman::{Complex, Matrix};
