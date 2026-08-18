//! Weak / integral-form discovery for noise-robust system identification.
//!
//! Strong-form SINDy fits `Ẋ = Θ(X) Ξ` using **estimated derivatives** of the
//! data — and estimating a derivative amplifies observation noise. The weak (or
//! integral) form removes that step entirely: multiply the ODE by a
//! compactly-supported smooth test function `φ_k` supported on a subdomain and
//! integrate. Integration by parts moves the time-derivative off the noisy data
//! and onto the analytic `φ`:
//!
//! ```text
//! ∫ φ_k · ẋ dt  =  ∫ φ_k · Θ(x) dt · Ξ
//!        ⇕  (integration by parts, φ vanishes at the support boundary)
//! −∫ φ̇_k · x dt =  ∫ φ_k · Θ(x) dt · Ξ
//! ```
//!
//! Stacking over `K` test functions gives an over-determined linear system
//! `G Ξ = B` per state, built purely from **integrals of the data** against test
//! functions and their (closed-form) derivatives — no differentiation of the
//! observations. This is markedly more robust to noise than a finite-difference
//! strong-form fit.
//!
//! The public entry point is [`weak_discover`]; see [`WeakConfig`] for the
//! deterministic controls and [`WeakResult`] for the discovered laws and
//! diagnostics. The specification lives in `specs/weak-form/README.md`.
//!
//! # Example
//!
//! ```
//! use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
//! use lawsynth_core::Identifier;
//! use lawsynth_weakform::{weak_discover, WeakConfig};
//!
//! // x(t) = t, so ẋ = 1: a trivial linear law over a fine grid.
//! let time: Vec<f64> = (0..=400).map(|i| i as f64 * 0.01).collect();
//! let x: Vec<f64> = time.clone();
//! let dataset = Dataset::new(
//!     TimeAxis::new(time).unwrap(),
//!     [NumericColumn::new(Identifier::new("x").unwrap(), x)],
//! )
//! .unwrap();
//!
//! let result = weak_discover(&dataset, &WeakConfig { feature_degree: 1, ..Default::default() })
//!     .unwrap();
//! // Coefficient of the constant term is ≈ 1 (ẋ = 1); the x term is ≈ 0.
//! assert!((result.coefficients[0][0] - 1.0).abs() < 1e-3);
//! ```

mod assembly;
mod config;
mod discover;
mod error;
mod quadrature;
mod solve;
mod test_function;

pub use assembly::WeakSystem;
pub use config::WeakConfig;
pub use discover::{WeakDiagnostics, WeakLaw, WeakResult, WeakTerm, weak_discover};
pub use error::WeakError;
pub use solve::{StlsqConfig, StlsqFit, stlsq};
pub use test_function::{TestFunction, place};
