//! Deterministic, cross-validated hyperparameter selection for LawSynth
//! discovery.
//!
//! Choosing discovery hyperparameters — candidate-library polynomial degree,
//! sparsity threshold, solver — is otherwise manual. This crate runs a
//! deterministic **time-series cross-validation sweep** over a user-supplied grid
//! of [`DiscoveryConfig`](lawsynth_discovery::DiscoveryConfig) candidates and
//! selects the one whose discovered model *generalizes* best, so a user gets a
//! principled model without hand-tuning.
//!
//! # Method
//!
//! 1. **Folds over time.** The timeline is cut into `folds + 1` contiguous,
//!    near-equal segments — never shuffled, because shuffling would destroy the
//!    temporal structure of dynamics data. Two schemes are offered
//!    ([`CvScheme`]): expanding-window *forward chaining* (train on all past,
//!    test on the next segment) and *rolling blocks* (train on one block, test on
//!    the next). Each fold **discovers on its training segment**, **simulates the
//!    discovered world across the test segment** from the test segment's first
//!    observed state, and scores predictive fit against the held-out observations.
//! 2. **Sweep.** Each candidate's fold scores are averaged; the candidate with
//!    the best mean held-out score wins, with ties broken deterministically
//!    toward the *simpler* model (lower degree, then higher threshold, then fewer
//!    active terms, then lower grid index).
//! 3. **Honesty.** The [`SelectionReport`] contains the **full** score table —
//!    every candidate's mean and per-fold scores — so the selection is auditable.
//!    A candidate whose discovery or simulation fails on a fold is recorded as a
//!    fold *failure* (a documented worst-case score), never silently dropped.
//!
//! Scoring reuses [`lawsynth_score::fit_statistics`] (the same R²/RMSE helper the
//! CLI `validate` command uses) applied to the re-simulated trajectory, so the
//! generalization estimate is consistent with the shipped forecast diagnostics.
//!
//! Everything — fold boundaries, sweep order, discovery, simulation, scoring — is
//! deterministic: identical inputs produce a bit-identical report (verified to
//! [`f64::to_bits`]). The crate is offline and std-only, depending on internal
//! LawSynth crates only.
//!
//! # Example
//!
//! ```no_run
//! use lawsynth_data::Dataset;
//! use lawsynth_discovery::DiscoveryConfig;
//! use lawsynth_modelselect::{CvConfig, sweep_degrees_thresholds};
//!
//! # fn run(dataset: &Dataset, base: &DiscoveryConfig) {
//! let cv = CvConfig::new(3);
//! let report = sweep_degrees_thresholds(dataset, base, &[1, 2, 3], &[0.05, 0.1], &cv).unwrap();
//! println!("{}", report.render_table());
//! let best = report.best();
//! # let _ = best;
//! # }
//! ```

mod config;
mod error;
mod fold;
mod report;
mod score;
mod select;

pub use config::{CvConfig, CvScheme, ScoreMetric};
pub use error::ModelSelectError;
pub use fold::{FoldPlan, plan_folds};
pub use report::{CandidateScore, ConfigSummary, FoldScore, FoldStatus, SelectionReport};
pub use select::{select_model, sweep_degrees_thresholds};

// Re-exported so callers can read a candidate's swept solver without a direct
// `lawsynth-discovery` dependency.
pub use lawsynth_discovery::SparseMethod;
