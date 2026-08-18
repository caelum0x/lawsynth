//! Deterministic implicit / rational (nullspace) dynamics discovery.
//!
//! Explicit sparse regression fits `ẋ = Θ(x) ξ` and therefore cannot express a
//! law whose right-hand side is a *ratio* of polynomials — the Michaelis-Menten
//! kinetics `ẋ = -Vmax·x / (Km + x)` being the canonical example. This crate
//! ports the public *implicit SINDy / SINDy-PI* idea: it searches for a sparse,
//! non-zero vector `ξ` in the approximate nullspace of an augmented library
//! `Θ(x, ẋ)` — candidate terms in the states *and* in the derivative — so that
//! `Θ(x, ẋ) ξ ≈ 0`.
//!
//! The trivial solution `ξ = 0` is excluded by the SINDy-PI *alternating
//! left-hand-side* scheme: each library column in turn is moved to the LHS and
//! the remainder is fit by sparse regression, which pins that column's
//! coefficient to `1`. The candidate that is both consistent (small residual)
//! and sparse is kept. Because every derivative-bearing term carries `ẋ` to the
//! first power, a discovered relation `A(x) + ẋ·B(x) = 0` is affine in `ẋ` and
//! is honestly reported *both* as the implicit relation and, where `B(x) ≢ 0`,
//! as the explicit rational law `ẋ = P(x)/Q(x)` with `P = -A`, `Q = B`.
//!
//! Every step — derivative estimation, library construction, the STLSQ inner
//! solve (reused from `lawsynth-sparse`), and candidate selection — is
//! deterministic, offline, and standard-library only. See
//! `specs/implicit-dynamics/README.md` for the contract, and its honesty notes
//! on the identifiability limits of implicit symbolic regression.

mod config;
mod discover;
mod error;
mod library;
mod rational;
mod result;
mod solve;

pub use config::ImplicitConfig;
pub use discover::implicit_discover;
pub use error::ImplicitError;
pub use lawsynth_differentiate::DerivativeMethod;
pub use library::{AugmentedLibrary, AugmentedMatrix, AugmentedTerm};
pub use rational::{MonomialTerm, Polynomial, RationalLaw};
pub use result::{
    CandidateScore, ImplicitDiagnostics, ImplicitRelation, ImplicitResult, ImplicitTerm,
};
