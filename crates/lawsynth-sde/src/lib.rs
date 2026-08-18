//! Deterministic stochastic (SDE) law discovery for LawSynth.
//!
//! Given noisy sample paths of a (diagonal-noise) Itô stochastic differential
//! equation
//!
//! ```text
//! dX = a(X) dt + b(X) dW,
//! ```
//!
//! this crate estimates the **drift** `a(x)` and the **diffusion** `b²(x)` and
//! recovers closed-form laws for them. It closes the loop with
//! `lawsynth-sim::euler_maruyama`, which can *simulate* such an SDE: this crate
//! *discovers* one from data.
//!
//! # Method — Kramers–Moyal conditional moments
//!
//! For a time step `Δt`, the conditional moments of the increment
//! `ΔX = X(t+Δt) − X(t)` given `X(t)=x` estimate the first two Kramers–Moyal
//! coefficients:
//!
//! ```text
//! a(x)  ≈ E[ΔX  | X = x] / Δt          (drift)
//! b²(x) ≈ E[ΔX² | X = x] / Δt          (diffusion; the drift² term is higher order in Δt)
//! ```
//!
//! The conditional expectations are estimated by **binning** the observed state
//! space and averaging `ΔX/Δt` and `ΔX²/Δt` within each bin (see
//! [`BinnedEstimate`]). The trusted bins — those with enough occupancy — are then
//! **sparse-regressed** (via `lawsynth-sparse`) onto a polynomial candidate
//! library (via `lawsynth-features`) to yield closed-form [`DiscoveredLaw`]s for
//! the drift and diffusion. Both the raw binned table and the fitted laws are
//! reported.
//!
//! # Determinism
//!
//! Binning, averaging, library evaluation and the sparse solve all run in a fixed
//! order with no hidden randomness. Identical `(Dataset, SdeConfig)` inputs
//! produce a bit-identical [`SdeModel`].
//!
//! # Honest limits
//!
//! This is a **statistical estimator**. Accuracy depends on the path length, on
//! `Δt` being small enough for the Kramers–Moyal expansion yet large enough to
//! average out sampling noise, and on bin occupancy. Longer paths tighten the
//! estimate; rarely-visited bins are unreliable. The finite-`Δt` bias of the
//! coefficients, multiplicative-noise / Stratonovich subtleties, and jumps are
//! out of scope. See `specs/sde-discovery/README.md` for the full contract.
//!
//! # Example
//!
//! ```
//! use lawsynth_core::Identifier;
//! use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
//! use lawsynth_sde::{BinRule, SdeConfig, discover_sde};
//!
//! // A short, evenly sampled path (see the crate's integration tests for the
//! // long deterministic Euler–Maruyama fixtures used for real recovery).
//! let time = TimeAxis::new((0..6).map(|i| i as f64 * 0.01).collect()).unwrap();
//! let column = NumericColumn::new(
//!     Identifier::new("x").unwrap(),
//!     vec![0.0, 0.02, 0.05, 0.03, 0.06, 0.04],
//! );
//! let dataset = Dataset::new(time, [column]).unwrap();
//!
//! let config = SdeConfig::new().with_bins(BinRule::Count(2)).with_min_bin_count(1);
//! // With so few samples the fit is meaningless — this only shows the shape of
//! // the API; real recovery needs long paths.
//! let _ = discover_sde(&dataset, &config);
//! ```

mod binning;
mod config;
mod discover;
mod error;
mod model;

pub use config::{BinRule, SdeConfig};
pub use discover::discover_sde;
pub use error::SdeError;
pub use model::{BinnedEstimate, DiscoveredLaw, LawTerm, SdeModel, StateModel};
