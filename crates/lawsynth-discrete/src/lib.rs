//! Deterministic discrete-time control and estimation for LawSynth.
//!
//! For a sampled-data linear model
//!
//! ```text
//! x_{k+1} = A x_k + B u_k,   y_k = C x_k,
//! ```
//!
//! this crate provides the discrete-time analogues of the continuous
//! feedback/estimation crates:
//!
//! - [`dlqr`] — infinite-horizon **discrete LQR** solving the discrete algebraic
//!   Riccati equation (DARE) `P = AᵀPA − AᵀPB(R + BᵀPB)⁻¹BᵀPA + Q` by a
//!   deterministic value iteration from `P₀ = Q`, then `K = (R + BᵀPB)⁻¹BᵀPA`
//!   for the control law `u = −K x`.
//! - [`discrete_kalman`] — the steady-state **discrete Kalman filter** solving
//!   the dual (filter) DARE `P = APAᵀ − APCᵀ(R + CPCᵀ)⁻¹CPAᵀ + Q` with the
//!   *predictor* gain `L = APCᵀ(R + CPCᵀ)⁻¹`.
//! - [`discrete_observer_from_poles`] — a single-output **discrete Luenberger
//!   observer** whose error poles are placed exactly in the z-plane by the dual
//!   of Ackermann's formula.
//!
//! Every design returns the achieved closed-loop / error spectrum computed with
//! the shared deterministic eigensolver in `lawsynth-koopman`, so a caller can
//! confirm **discrete stability directly**: a discrete system is stable when its
//! spectral radius is below one — *all* eigenvalues lie strictly inside the
//! **unit circle** (`|λ| < 1`), not in the open left half-plane.
//!
//! All linear algebra is hand-rolled on the standard library only — Gaussian
//! elimination, matrix products, and matrix polynomials — with fixed loop and
//! pivot order, so identical inputs yield bit-identical `K`, `L`, `P`, and
//! spectra. No external crates, no RNG, no clock.
//!
//! See `specs/discrete-time-control/README.md` for the boundary specification
//! and its honest limits.

mod dare;
mod dlqr;
mod error;
mod gain;
mod kalman;
mod linalg;
mod observer;
mod place;
mod validate;

pub use dlqr::dlqr;
pub use error::DiscreteError;
pub use gain::DiscreteGain;
pub use kalman::discrete_kalman;
pub use observer::{DiscreteObserver, ObserverMethod};
pub use place::discrete_observer_from_poles;

// Re-export the shared linear-algebra types so callers can build `(A, B, C)` and
// read achieved spectra without a direct dependency on `lawsynth-koopman`.
pub use lawsynth_koopman::{Complex, Matrix};
