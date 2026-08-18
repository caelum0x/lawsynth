//! Deterministic fixed-point and linear-stability analysis for LawSynth.
//!
//! Given a discovered **autonomous** vector field `ẋ = f(x)` — one expression
//! tree per state — this crate:
//!
//! 1. **locates fixed points** `f(x*) = 0` with a deterministic multivariate
//!    Newton iteration, `x_{k+1} = x_k − J(x_k)^{-1} f(x_k)`, started from a
//!    fixed lattice of seeds over a caller-provided search box plus the origin;
//!    and
//! 2. **classifies** each fixed point by the eigenvalues of the Jacobian there
//!    (stable node/spiral, unstable node/spiral, saddle, center, or marginal).
//!
//! # Design
//!
//! - **Reuse, not reinvention.** The Jacobian is the analytic Jacobian from
//!   [`lawsynth_jacobian`]; its eigenvalues come from the deterministic
//!   Householder-Hessenberg + Wilkinson-shifted complex-QR eigensolver in
//!   [`lawsynth_koopman`]. Only the small dense linear solve inside Newton is
//!   local (Gaussian elimination with partial pivoting, std-only).
//! - **Deterministic seeds.** Start points are a fixed, content-independent grid
//!   over the search box plus the origin — never random, never wall-clock
//!   derived. Roots are de-duplicated within a tolerance and ordered
//!   lexicographically.
//! - **Honest reporting.** A seed that does not converge is dropped and counted;
//!   the report states how many of the seeds converged. Non-hyperbolic points
//!   (eigenvalue real parts inside the tolerance band) are reported as `Center`
//!   or `Marginal` — the linearization is inconclusive there and the crate says
//!   so rather than inventing a definitive class.
//! - **Determinism.** Identical `(fields, states, config)` inputs yield a
//!   bit-identical [`StabilityReport`] (coordinates, classification, and
//!   eigenvalues down to their `f64` bit patterns).
//! - **Offline, std-only.** No external crates; only internal LawSynth paths.
//!
//! See `specs/stability-analysis/README.md` for the conformance contract.
//!
//! # Example
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::{Expr, UnaryOperator};
//! use lawsynth_stability::{Classification, StabilityConfig, analyze_stability};
//!
//! let x = Identifier::new("x").unwrap();
//! let y = Identifier::new("y").unwrap();
//! // A linear system with a stable node at the origin: x' = -x, y' = -2y.
//! let fields = vec![
//!     (x.clone(), Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()))),
//!     (
//!         y.clone(),
//!         Expr::product(Expr::constant(-2.0), Expr::symbol(y.clone())),
//!     ),
//! ];
//! let config = StabilityConfig::new(vec![(-1.0, 1.0), (-1.0, 1.0)]);
//! let report = analyze_stability(&fields, &[x, y], &config).unwrap();
//! assert_eq!(report.fixed_points.len(), 1);
//! assert_eq!(report.fixed_points[0].classification, Classification::StableNode);
//! ```

mod analysis;
mod classify;
mod config;
mod error;
mod linalg;
mod newton;
mod report;
mod seeds;

pub use analysis::analyze_stability;
pub use classify::{Classification, classify};
pub use config::{
    DEFAULT_DEDUP_TOLERANCE, DEFAULT_DIVERGENCE_LIMIT, DEFAULT_GRID_RESOLUTION,
    DEFAULT_MARGINAL_BAND, DEFAULT_MAX_ITERATIONS, DEFAULT_TOLERANCE, StabilityConfig,
};
pub use error::StabilityError;
pub use report::{FixedPoint, StabilityReport};

// Re-exported so callers can inspect eigenvalues without depending on
// `lawsynth-koopman` directly.
pub use lawsynth_koopman::Complex;
