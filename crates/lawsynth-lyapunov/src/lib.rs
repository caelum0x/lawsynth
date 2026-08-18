//! Deterministic Lyapunov-exponent estimation (chaos diagnostic) for LawSynth.
//!
//! Given a discovered autonomous vector field `ẋ = f(x)` — a set of
//! expression-tree fields over the state symbols `x` — this crate estimates the
//! **Lyapunov spectrum**: the average exponential rates at which infinitesimally
//! nearby trajectories separate. A positive largest exponent is the signature of
//! chaos. This complements the local fixed-point analysis of
//! `crates/lawsynth-stability` and the bifurcation tracking with a global,
//! long-term diagnostic of the flow.
//!
//! # The method (Benettin / QR)
//!
//! Evolve the state `x(t)` together with an orthonormal frame `Q ∈ Rⁿˣⁿ` of
//! perturbation vectors under the variational flow:
//!
//! ```text
//! ẋ   = f(x)                      (fixed-step RK4)
//! q̇_j = J(x) · q_j                J(x) = ∂f/∂x, the analytic Jacobian
//! ```
//!
//! The augmented `(x, Q)` is advanced by one shared RK4 integrator so the state
//! and the frame stay consistent. Every `k` steps the evolved frame is
//! QR-decomposed, `Q = Q'·R`; `Q'` becomes the new orthonormal frame and
//! `ln R_ii` is accumulated for each `i`. After discarding a transient, the
//! `i`-th exponent is
//!
//! ```text
//! λ_i = (Σ ln R_ii) / T,
//! ```
//!
//! where `T` is the elapsed time of the averaging window. Exponents are returned
//! sorted descending.
//!
//! Two derived diagnostics accompany the spectrum:
//!
//! - the **sum** `Σ λ_i`, which equals the time-averaged divergence (the mean
//!   trace of `J` along the trajectory) — the tightest, most reliable quantity;
//! - the **Kaplan–Yorke (Lyapunov) dimension**
//!   `D_KY = j + (Σ_{i≤j} λ_i)/|λ_{j+1}|`, where `j` is the largest index whose
//!   partial sum is non-negative (`0` if none, `n` if all are).
//!
//! # Design
//!
//! - **Everything analytic in its derivatives.** `J(x)` is reused from
//!   [`lawsynth_jacobian`]; only the time integration is numerical. No finite
//!   differencing of the field appears anywhere.
//! - **Deterministic and offline.** The initial frame is the fixed identity, the
//!   RK4 stages and Gram–Schmidt QR run in a fixed arithmetic order, and no RNG
//!   or clock is consulted. Identical inputs yield a bit-identical report.
//! - **std-only.** A small local dense linear algebra ([`linalg`]) supplies the
//!   Gram–Schmidt QR; no external crates, only internal LawSynth paths.
//!
//! # Honest limits
//!
//! The spectrum is a **time-averaged estimate**. Its accuracy depends on the
//! integration length, the step `dt`, and the reorthonormalization interval. The
//! individual chaotic exponent converges slowly (fluctuating like `1/√T`), while
//! the sum (divergence) is tight. The trajectory must explore the attractor (the
//! initial condition in its basin, past the transient), and fixed-step RK4 error
//! grows on stiff or fast systems. See `specs/lyapunov-exponents/README.md` for
//! the full conformance contract and limits.
//!
//! # Example
//!
//! The linear decay `ẋ = −x` has the single exact exponent `−1`.
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_expr::{Expr, UnaryOperator};
//! use lawsynth_lyapunov::{LyapunovConfig, lyapunov_spectrum};
//!
//! let x = Identifier::new("x").unwrap();
//! let field = Expr::unary(UnaryOperator::Negate, Expr::symbol(x.clone()));
//! let fields = vec![(x.clone(), field)];
//!
//! let config = LyapunovConfig::default().with_steps(4000);
//! let report = lyapunov_spectrum(&fields, &[x], &[1.0], &config).unwrap();
//!
//! assert_eq!(report.dimension(), 1);
//! assert!((report.largest() - (-1.0)).abs() < 1e-3);
//! ```

mod config;
mod error;
mod linalg;
mod report;
mod spectrum;
mod system;

pub use config::LyapunovConfig;
pub use error::LyapunovError;
pub use report::LyapunovReport;
pub use spectrum::{largest_lyapunov, lyapunov_spectrum};
