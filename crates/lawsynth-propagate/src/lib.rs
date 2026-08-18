//! Deterministic propagation of coefficient uncertainty into forecast intervals.
//!
//! A discovered model `ẋ = f(x; θ)` comes with uncertain coefficients: the
//! bootstrap ensemble of `lawsynth-uncertainty` gives their covariance `Cov(θ)`.
//! This crate propagates that **parameter** uncertainty into **trajectory**
//! uncertainty, so a forecast arrives with prediction bands. It offers two
//! deterministic methods that agree in the small-uncertainty limit and diverge —
//! honestly — under strong nonlinearity or large uncertainty.
//!
//! # The two methods
//!
//! - **Delta method** ([`delta_forecast`]) — analytic and linearised. Using the
//!   forward sensitivities `S(t) = ∂x(t)/∂θ` (integrated by `lawsynth-sensitivity`),
//!   the state covariance is the first-order image `Cov(x(t)) ≈ S(t)·Cov(θ)·S(t)ᵀ`.
//!   The per-state variance is that product's diagonal, and a band is
//!   `x(t) ± z·sqrt(variance)`.
//! - **Monte-Carlo** ([`monte_carlo_forecast`]) — a seeded ensemble. `M` parameter
//!   vectors are drawn from the ensemble (either resampling the bootstrap replicate
//!   coefficients directly, or sampling `N(mean, Cov(θ))` via a deterministic
//!   SplitMix64 + Box–Muller draw shaped by the Cholesky factor), each simulated
//!   with the same fixed-step RK4 the sensitivities use, and the bands are the
//!   per-time empirical mean and percentile interval.
//!
//! # From a bootstrap to a band
//!
//! [`covariance_from_ensemble`] turns a `CoefficientEnsemble` (the bootstrap
//! replicate coefficient vectors) straight into the sample covariance `Cov(θ)`,
//! which then feeds either method. [`z_for_confidence`] maps a two-sided
//! confidence level to the delta-method multiplier `z` so the two methods can be
//! compared on the same footing.
//!
//! # Guarantees
//!
//! - **Deterministic and offline.** No external crates, only internal LawSynth
//!   paths, and no wall-clock seeding: identical inputs yield **bit-identical**
//!   bands, and the Monte-Carlo ensemble is independent of iteration order
//!   because sample `m` is seeded from `(seed, m)`.
//! - **Honest about limits.** The delta method is first-order and undercovers
//!   under strong nonlinearity or large uncertainty; Monte-Carlo coverage is only
//!   as good as the ensemble/Gaussian assumption; both inherit the fixed-step
//!   integrator's accuracy and the upstream fit's biases. See
//!   `specs/forecast-uncertainty/README.md` for the full contract.

mod bands;
mod covariance;
mod delta;
mod error;
mod monte_carlo;
mod rng;
mod stats;

pub use bands::ForecastBands;
pub use covariance::covariance_from_ensemble;
pub use delta::delta_forecast;
pub use error::PropagateError;
pub use monte_carlo::{EnsembleSource, monte_carlo_forecast};
pub use stats::z_for_confidence;
