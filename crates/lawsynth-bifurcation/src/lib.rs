//! Deterministic parameter continuation and bifurcation detection for LawSynth.
//!
//! Given a discovered vector field `ẋ = f(x; μ)` whose right-hand sides depend on
//! a scalar **parameter** `μ`, this crate sweeps `μ` over a range and:
//!
//! 1. **re-locates fixed points** at each `μ` by substituting the parameter out
//!    and reusing [`lawsynth_stability::analyze_stability`];
//! 2. **tracks branches** — stitches the fixed points across consecutive `μ` by
//!    nearest-coordinate matching, so each branch is one equilibrium followed as
//!    the parameter varies; and
//! 3. **detects bifurcations** — parameter values where a Jacobian eigenvalue
//!    crosses the imaginary axis. A (near-)real eigenvalue through zero is the
//!    saddle-node / transcritical / pitchfork family, reported generically as a
//!    [`BifurcationKind::Fold`]; a complex-conjugate pair crossing with non-zero
//!    imaginary part is a [`BifurcationKind::Hopf`].
//!
//! # Design
//!
//! - **Reuse, not reinvention.** Fixed points, Jacobian eigenvalues, and their
//!   classification all come from `lawsynth-stability` (which itself builds on the
//!   analytic Jacobian of `lawsynth-jacobian` and the eigensolver of
//!   `lawsynth-koopman`). This crate adds only the parameter substitution, the
//!   branch matching, and the crossing/fold localization.
//! - **Exact parameter substitution.** [`substitute`] structurally replaces every
//!   occurrence of the parameter symbol with a constant, yielding a parameter-free
//!   (autonomous) field. No folding is applied, so it is exact and reproducible.
//! - **Deterministic grid and localization.** The parameter grid is a fixed
//!   function of `(min, max, steps)`; critical values are localized by
//!   deterministic bisection (on the dominant real part for crossings, on
//!   fixed-point existence for collision folds).
//! - **Honest branches.** A branch that cannot be continued ends; a fixed point
//!   with no predecessor starts a new branch. No global connectivity is claimed.
//! - **Determinism.** Identical inputs yield a bit-identical [`ContinuationReport`]
//!   (grid, coordinates, eigenvalues, and bifurcation parameters down to `f64`
//!   bit patterns). See [`ContinuationReport::to_canonical_string`].
//! - **Offline, std-only.** No external crates; only internal LawSynth paths.
//!
//! See `specs/bifurcation-analysis/README.md` for the conformance contract.
//!
//! # Example
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::{BinaryOperator, Expr};
//! use lawsynth_stability::StabilityConfig;
//! use lawsynth_bifurcation::{BifurcationKind, Sweep, continuation};
//!
//! // Transcritical normal form: x' = mu*x - x^2.
//! let x = Identifier::new("x").unwrap();
//! let mu = Identifier::new("mu").unwrap();
//! let field = Expr::difference(
//!     Expr::product(Expr::symbol(mu.clone()), Expr::symbol(x.clone())),
//!     Expr::binary(BinaryOperator::Power, Expr::symbol(x.clone()), Expr::constant(2.0)),
//! );
//! let fields = vec![(x.clone(), field)];
//!
//! let sweep = Sweep::new(-1.0, 1.0, 21);
//! let stability = StabilityConfig::new(vec![(-2.0, 2.0)]);
//! let report = continuation(&fields, &[x], &mu, &sweep, &stability).unwrap();
//!
//! // A zero-eigenvalue (fold-family) bifurcation is found near mu = 0.
//! let bifurcation = &report.bifurcations[0];
//! assert_eq!(bifurcation.kind, BifurcationKind::Fold);
//! assert!(bifurcation.parameter_value.abs() < 1e-3);
//! ```

mod branch;
mod context;
mod continuation;
mod detect;
mod error;
mod report;
mod substitute;
mod sweep;

pub use continuation::continuation;
pub use error::BifurcationError;
pub use report::{
    Bifurcation, BifurcationKind, Branch, BranchPoint, ContinuationReport, Localization,
    ParameterSample,
};
pub use substitute::substitute;
pub use sweep::{
    DEFAULT_CROSSING_BAND, DEFAULT_DEDUP_COORDINATE_TOLERANCE, DEFAULT_DEDUP_PARAMETER_TOLERANCE,
    DEFAULT_FOLD_EIGENVALUE_TOLERANCE, DEFAULT_IMAGINARY_TOLERANCE,
    DEFAULT_LOCALIZATION_ITERATIONS, DEFAULT_MATCH_TOLERANCE, DEFAULT_STEPS, Sweep,
};

// Re-exported so callers can inspect eigenvalues and classifications without
// depending on the lower crates directly.
pub use lawsynth_koopman::Complex;
pub use lawsynth_stability::{Classification, FixedPoint, StabilityConfig, StabilityReport};
