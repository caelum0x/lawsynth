//! Deterministic linear model-order reduction by **balanced truncation**.
//!
//! Given a stable linear model `ẋ = A x + B u`, `y = C x`, this crate produces a
//! lower-order model `(Aᵣ, Bᵣ, Cᵣ)` of order `k < n` that preserves the dominant
//! input-output response. The method is textbook square-root balanced truncation
//! (Moore 1981; Laub et al. 1987):
//!
//! 1. **Stability precondition.** `A` must be Hurwitz (every eigenvalue with
//!    `Re < 0`) for the infinite-horizon gramians to exist, checked with the
//!    deterministic eigensolver of `lawsynth-koopman`.
//! 2. **Gramians.** Solve the continuous Lyapunov equations
//!    `A Wc + Wc Aᵀ + B Bᵀ = 0` and `Aᵀ Wo + Wo A + Cᵀ C = 0` exactly by
//!    Kronecker vectorization and local Gaussian elimination.
//! 3. **Balancing.** Factor `Wc = R Rᵀ`, diagonalize `Rᵀ Wo R = U Σ² Uᵀ`; the
//!    diagonal of `Σ` holds the **Hankel singular values**, and `T = R U Σ^{-1/2}`
//!    balances the realization so both transformed gramians equal `diag(σ)`.
//! 4. **Truncation.** Keep the `k` states with the largest Hankel singular values.
//!
//! All linear algebra is hand-rolled on the standard library only — a Kronecker
//! Lyapunov solve, Gaussian elimination, and a cyclic Jacobi symmetric
//! eigensolver — with fixed loop and pivot order, so identical inputs yield
//! **bit-identical** reduced models and Hankel singular values.
//!
//! [`Matrix`] and [`Complex`] are re-exported from `lawsynth-koopman` so a caller
//! builds `(A, B, C)` and reads results without a separate dependency.
//!
//! See `specs/model-reduction/README.md` for the boundary specification and its
//! honest limits (continuous-time stable systems only; conditioning degrades for
//! highly non-normal `A` or tiny Hankel-singular-value gaps).

mod balance;
mod error;
mod gramian;
mod linalg;
mod reduce;

pub use error::ModelReduceError;
pub use gramian::{controllability_gramian, observability_gramian};
pub use reduce::{ReducedModel, ReductionSpec, balanced_truncation, hankel_singular_values};

// Re-export the shared linear-algebra value types so callers need not depend on
// `lawsynth-koopman` directly to build inputs and read achieved spectra.
pub use lawsynth_koopman::{Complex, Matrix, eigen};
