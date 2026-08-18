//! Deterministic analytic Jacobian codegen for LawSynth.
//!
//! Given a discovered vector field — a set of expression trees, one per state
//! derivative `dx_i/dt = f_i(x, ...)` — this crate symbolically differentiates
//! to produce the analytic Jacobian matrix `J[i][j] = ∂f_i/∂x_j`. That matrix
//! is what implicit/stiff ODE integrators need, and its eigenvalues at a fixed
//! point give the local linear stability of the discovered law.
//!
//! # Design
//!
//! - **Exact symbolic differentiation** over the [`lawsynth_expr`] IR with the
//!   standard sum, product, quotient, chain, and power rules. A power with a
//!   constant exponent uses `d(f^c) = c·f^(c-1)·f'`, which stays correct for
//!   negative bases; the fully general `f^g` rule is applied only when needed.
//!   A node that has no real closed-form derivative returns a typed error
//!   ([`JacobianError::UnsupportedDerivative`]) — never a silent zero.
//! - **Conservative simplification** via [`lawsynth_expr::Expr::simplify`]:
//!   constant folding and the `+0`, `−0`, `*1`, `*0`, `^0`, `^1` identities.
//!   This is readability- and cost-oriented, not a canonical normal form.
//! - **Determinism**: rows and columns follow the caller's `states` ordering,
//!   fields are matched by identifier via linear scan, and no hash-map iteration
//!   order ever leaks into the output. Identical inputs yield bit-identical
//!   matrices (structure and float bits).
//! - **Offline, std-only**: no external crates; only internal LawSynth paths.
//!
//! See `specs/analytic-jacobian/README.md` for the conformance contract.
//!
//! # Example
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::{Environment, Expr, UnaryOperator};
//! use lawsynth_jacobian::analytic_jacobian;
//!
//! let x = Identifier::new("x").unwrap();
//! let y = Identifier::new("y").unwrap();
//! // Damped oscillator: x' = y, y' = -x - 0.3 y.
//! let fields = vec![
//!     (x.clone(), Expr::symbol(y.clone())),
//!     (
//!         y.clone(),
//!         Expr::difference(
//!             Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone())),
//!             Expr::product(Expr::constant(0.3), Expr::symbol(y.clone())),
//!         ),
//!     ),
//! ];
//! let jacobian = analytic_jacobian(&fields, &[x.clone(), y.clone()]).unwrap();
//! let point = Environment::from([(x, 0.0), (y, 0.0)]);
//! assert_eq!(jacobian.evaluate(&point).unwrap(), vec![vec![0.0, 1.0], vec![-1.0, -0.3]]);
//! ```

mod differentiate;
mod error;
mod jacobian;

pub use differentiate::differentiate;
pub use error::JacobianError;
pub use jacobian::{Jacobian, analytic_jacobian};
