//! Deterministic receding-horizon (model-predictive) control for LawSynth.
//!
//! Given a discovered nonlinear model `ẋ = f(x, u)` — state fields expressed as
//! [`Expr`](lawsynth_expr::Expr) trees over state symbols and one or more control
//! symbols — this crate drives the state to a setpoint by **successive-
//! linearization MPC** (equivalently, gain-scheduled LQR). At each control step
//! it:
//!
//! 1. **linearizes** the model about the current `(x, u_ref)` — the analytic
//!    Jacobian gives `A = ∂f/∂x` (via [`lawsynth_jacobian`]) and the symbolic
//!    control partials give `B = ∂f/∂u`;
//! 2. **designs** an infinite-horizon LQR feedback `K` for that local `(A, B)`
//!    with weights `Q, R` (via [`lawsynth_feedback::lqr`]);
//! 3. **applies** the first move `u = clamp(u_ref − K (x − x_ref), u_min, u_max)`;
//! 4. **advances** the true nonlinear plant one fixed step by classical RK4 with
//!    that control held constant,
//!
//! and repeats. The closed-loop state and control trajectory is returned.
//!
//! # Determinism and scope
//!
//! Every stage is deterministic and offline (std-only, internal path deps): the
//! symbolic Jacobian, the Kleinman LQR solve, and the fixed-step RK4 all have a
//! fixed evaluation order, so identical inputs produce a **bit-identical**
//! trajectory (compare via [`f64::to_bits`] or
//! [`MpcTrajectory::bit_fingerprint`]).
//!
//! This is **successive-linearization LQR-MPC, not a constrained QP-MPC**:
//! saturation is applied by clamping (not a constraint-optimal projection),
//! optimality is only local to each linearization, and there is no
//! horizon/feasibility/recursive-stability guarantee. See
//! `specs/model-predictive-control/README.md` for the full contract and limits.
//!
//! # Example
//!
//! A double integrator `ẋ = y, ẏ = u` regulated from `x₀ = (1, 0)` to the origin.
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::Expr;
//! use lawsynth_feedback::Matrix;
//! use lawsynth_mpc::{MpcConfig, mpc_control};
//!
//! let x = Identifier::new("x").unwrap();
//! let y = Identifier::new("y").unwrap();
//! let u = Identifier::new("u").unwrap();
//!
//! // ẋ = y ; ẏ = u
//! let fields = vec![
//!     (x.clone(), Expr::symbol(y.clone())),
//!     (y.clone(), Expr::symbol(u.clone())),
//! ];
//!
//! let q = Matrix::identity(2);
//! let r = Matrix::from_rows(vec![vec![1.0]]).unwrap();
//! let config = MpcConfig::new(vec![1.0, 0.0], vec![0.0, 0.0], q, r, 0.05, 200);
//!
//! let trajectory =
//!     mpc_control(&fields, &[x, y], &[u], &config).unwrap();
//! // The controller drives the state to the origin.
//! assert!(trajectory.final_error_norm(&[0.0, 0.0]).unwrap() < 1e-3);
//! ```

mod config;
mod controller;
mod error;
mod model;
mod trajectory;

pub use config::MpcConfig;
pub use controller::mpc_control;
pub use error::MpcError;
pub use trajectory::MpcTrajectory;

// Re-export the matrix type so callers build `Q`/`R` and read per-step gains
// without a separate dependency on `lawsynth-koopman` or `lawsynth-feedback`.
pub use lawsynth_koopman::Matrix;
