//! Deterministic linear state-feedback design for LawSynth.
//!
//! Given a linearization `ẋ = A x + B u` — typically `A` the Jacobian of a
//! discovered field at a fixed point and `B` the control-input matrix — this
//! crate designs a stabilizing feedback gain `K` for the control law `u = −K x`,
//! so the closed loop `A − B K` has the desired (or optimal) spectrum. Two
//! workhorse methods are provided:
//!
//! - [`place_poles`] — single-input **pole placement** by Ackermann's formula.
//!   The desired closed-loop poles are placed exactly, requiring a controllable
//!   pair `(A, b)` and poles closed under conjugation (so `K` is real).
//! - [`lqr`] — infinite-horizon **LQR** solving the continuous-time algebraic
//!   Riccati equation by a deterministic Kleinman (Newton–Riccati) iteration
//!   bootstrapped with Bass's initial stabilizing gain.
//!
//! Both return a [`Gain`] whose `achieved_poles` are the eigenvalues of
//! `A − B K` computed by the shared deterministic eigensolver in
//! `lawsynth-koopman`, so a caller can verify stability directly. All linear
//! algebra is hand-rolled on the standard library only — Gaussian elimination,
//! a Kronecker-form Lyapunov solve, and matrix polynomials — with fixed loop and
//! pivot order, so identical inputs yield bit-identical `K`, `P`, and poles.
//!
//! See `specs/control-design/README.md` for the boundary specification and its
//! honest limits.

mod error;
mod gain;
mod linalg;
mod lqr;
mod place;

pub use error::FeedbackError;
pub use gain::Gain;
pub use lqr::lqr;
pub use place::place_poles;

// Re-export the shared linear-algebra types so callers can build `(A, B)` and
// read achieved poles without a direct dependency on `lawsynth-koopman`.
pub use lawsynth_koopman::{Complex, Matrix};
