//! Deterministic forward sensitivity analysis for LawSynth.
//!
//! Given a discovered model `ẋ = f(x; θ)` — a set of expression-tree fields over
//! state symbols `x` and parameter symbols `θ` — this crate integrates the
//! **forward sensitivity (variational) equations** to obtain the trajectory
//! sensitivities `S_j(t) = ∂x(t)/∂θ_j`: how a change in each discovered
//! coefficient perturbs the forecast. These drive uncertainty propagation,
//! identifiability diagnosis, and optimal experimental design.
//!
//! # The system
//!
//! For state `x ∈ Rⁿ`, parameters `θ ∈ Rᵖ`, and `ẋ = f(x; θ)`:
//!
//! ```text
//! ẋ   = f(x; θ)
//! Ṡ_j = J_x · S_j + f_{θ_j}        S_j(0) = 0     (j = 1 … p)
//! ```
//!
//! where `J_x = ∂f/∂x` is the analytic `n × n` Jacobian and
//! `f_{θ_j} = ∂f/∂θ_j` is the `n`-vector of field partials with respect to
//! parameter `θ_j`. The augmented state `(x, S_1, …, S_p)` is advanced by a
//! single fixed-step fourth-order Runge–Kutta integrator, so the state and the
//! sensitivities share stage points and stay consistent.
//!
//! The initial sensitivities are zero because the initial state is taken to be
//! independent of the parameters. (Sensitivity to the initial condition, which
//! would use `S(0) = I`, is out of scope here.)
//!
//! # Design
//!
//! - **Everything analytic.** `J_x` is reused from [`lawsynth_jacobian`]; each
//!   `f_{θ_j}` is obtained by symbolically differentiating the field with the
//!   same crate's [`differentiate`](lawsynth_jacobian::differentiate) and then
//!   evaluating. No finite differences appear in the integrator.
//! - **Deterministic and offline.** Fixed evaluation and accumulation order, no
//!   external crates, only internal LawSynth paths. Identical inputs yield a
//!   bit-identical trajectory and sensitivities.
//! - **Honest about parameters.** A parameter that never appears in the fields
//!   differentiates to zero, so its sensitivity is exactly zero — the correct,
//!   non-fabricated answer.
//!
//! See `specs/sensitivity-analysis/README.md` for the conformance contract.
//!
//! # Example
//!
//! The linear law `ẋ = −θ·x` with `x(0) = x₀` has the closed form
//! `x(t) = x₀·e^{−θ t}` and `∂x/∂θ = −t·x₀·e^{−θ t}`. The integrated sensitivity
//! matches:
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::{Expr, UnaryOperator};
//! use lawsynth_sensitivity::{SensitivityConfig, forward_sensitivities};
//!
//! let x = Identifier::new("x").unwrap();
//! let theta = Identifier::new("theta").unwrap();
//! // ẋ = -theta * x
//! let field = Expr::unary(
//!     UnaryOperator::Negate,
//!     Expr::product(Expr::symbol(theta.clone()), Expr::symbol(x.clone())),
//! );
//! let fields = vec![(x.clone(), field)];
//!
//! let config = SensitivityConfig::new(0.0, 0.01, 100); // integrate to t = 1.0
//! let trajectory = forward_sensitivities(
//!     &fields,
//!     &[x.clone()],
//!     &[theta.clone()],
//!     &[2.0],  // x0
//!     &[0.5],  // theta
//!     &config,
//! )
//! .unwrap();
//!
//! let last = trajectory.sample_count() - 1;
//! let t = trajectory.times()[last];
//! let closed_form = -t * 2.0 * (-0.5 * t).exp();
//! let integrated = trajectory.partial(0, 0, last).unwrap();
//! assert!((integrated - closed_form).abs() < 1e-4);
//! ```

mod config;
mod error;
mod integrate;
mod system;
mod trajectory;

pub use config::SensitivityConfig;
pub use error::SensitivityError;
pub use integrate::forward_sensitivities;
pub use trajectory::SensitivityTrajectory;
