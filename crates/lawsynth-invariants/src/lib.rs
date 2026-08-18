//! Deterministic conserved-quantity (invariant) detection for LawSynth.
//!
//! Given a discovered autonomous vector field `ẋ = f(x)` — one expression tree
//! per state derivative — this crate searches for **conserved quantities**
//! `H(x)`: nonconstant functions that stay constant along the flow. A function
//! is conserved exactly when its **Lie derivative** vanishes everywhere,
//!
//! ```text
//! L_f H = ∇H · f = Σ_i (∂H/∂x_i) · f_i(x) = 0.
//! ```
//!
//! # Method
//!
//! 1. **Parametrize** `H` over a candidate library `{φ_1, …, φ_m}` (monomials up
//!    to a chosen degree, optionally with `sin`/`cos` terms), so
//!    `H(x) = Σ_j c_j φ_j(x)`. The constant function is deliberately excluded so
//!    the trivial `H = const` is never reported.
//! 2. **Linearize.** Because `L_f φ_j = ∇φ_j · f` is a known function of `x`,
//!    conservation is the linear constraint `Σ_j c_j (L_f φ_j) = 0`. Sampling it
//!    on a deterministic grid `x^(1)…x^(N)` yields a matrix
//!    `M[k][j] = (L_f φ_j)(x^(k))`; a conserved quantity is a nonzero `c` with
//!    `M c ≈ 0`, i.e. a vector in the numerical **nullspace** of `M`.
//! 3. **Nullspace via SVD.** The right-singular vectors of `M` with (near-)zero
//!    singular values span the conserved quantities. The decomposition is the
//!    deterministic one-sided Jacobi SVD from [`lawsynth_koopman`]. Each reported
//!    invariant carries its coefficient vector, its residual `‖M c‖`, and its
//!    singular value, and is canonically normalized (unit norm, largest-magnitude
//!    entry positive).
//!
//! The `∂φ_j/∂x_i` are exact symbolic derivatives from [`lawsynth_jacobian`], and
//! every stage — library order, sample grid, SVD, normalization — is
//! deterministic, so identical inputs yield a bit-identical [`InvariantReport`].
//! The crate is offline and depends only on internal LawSynth paths.
//!
//! # Honest limits
//!
//! A detection is a **hypothesis**, not a proof: it finds only invariants
//! expressible in the chosen library (a polynomial library cannot represent a
//! transcendental invariant), and a near-null vector is validated by its residual
//! over the sample grid, not proven to be exactly conserved. See
//! `specs/invariant-detection/README.md` for the full contract.
//!
//! # Example
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::{Expr, UnaryOperator};
//! use lawsynth_invariants::{InvariantConfig, detect_invariants};
//!
//! let x = Identifier::new("x").unwrap();
//! let y = Identifier::new("y").unwrap();
//! // Harmonic oscillator: ẋ = y, ẏ = -x. Energy x² + y² is conserved.
//! let fields = vec![
//!     (x.clone(), Expr::symbol(y.clone())),
//!     (y.clone(), Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()))),
//! ];
//! let config = InvariantConfig::default();
//! let report = detect_invariants(&fields, &[x, y], &config).unwrap();
//!
//! assert_eq!(report.invariants.len(), 1);
//! let invariant = &report.invariants[0];
//! // Recovered H is proportional to x² + y²: equal weight on the squares.
//! let x2 = invariant.coefficient(&report.basis_labels, "x^2").unwrap();
//! let y2 = invariant.coefficient(&report.basis_labels, "y^2").unwrap();
//! assert!((x2 - y2).abs() < 1e-9);
//! assert!(invariant.residual < 1e-9);
//! ```

mod basis;
mod config;
mod detect;
mod error;
mod grid;
mod lie;
mod report;

pub use basis::{BasisFunction, build_basis};
pub use config::InvariantConfig;
pub use detect::detect_invariants;
pub use error::InvariantError;
pub use report::{Invariant, InvariantReport};
