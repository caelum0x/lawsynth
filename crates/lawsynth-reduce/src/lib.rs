//! Deterministic **symmetry & separability** structural reductions.
//!
//! AI-Feynman shrinks symbolic search by structurally reducing a target
//! `f(x1..xn)` before searching: dimensional analysis, then symmetry, then
//! separability (`f = g(A) + h(B)` additive, `f = g(A)·h(B)` multiplicative)
//! split an `n`-variable problem into smaller independent sub-problems. AI-Feynman
//! detects that structure by probing a **trained neural network** and reading its
//! gradients. LawSynth is deterministic and offline, so this crate replaces the
//! learned probe with the data's **own numerical partial derivatives** — the
//! three-point / finite-difference estimators in
//! [`lawsynth_differentiate`] — evaluated on a reconstructed Cartesian grid.
//!
//! # What a detection means
//!
//! A detected reduction is a **hypothesis**, not a proof. Each candidate is first
//! *screened* with a numerical mixed partial (separability) or a first-derivative
//! invariance (symmetry), then *verified* by reconstructing the data from the
//! reduced form and measuring a relative residual. Only reductions that pass both
//! are reported, each carrying its screening residual, its reconstruction
//! residual, and a confidence. Data with no structure yields an empty report.
//!
//! The public entry point is [`detect_reductions`]; the boundary specification
//! lives in `specs/structural-reductions/README.md`.
//!
//! # Example
//!
//! ```
//! use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
//! use lawsynth_core::Identifier;
//! use lawsynth_reduce::{detect_reductions, ReduceConfig, SeparabilityKind};
//!
//! // f = sin(x) + y^2 sampled on a 12x12 grid, flattened row-major.
//! let xs: Vec<f64> = (0..12).map(|i| i as f64 * 0.25).collect();
//! let ys: Vec<f64> = (0..12).map(|j| j as f64 * 0.25).collect();
//! let (mut xc, mut yc, mut fc) = (Vec::new(), Vec::new(), Vec::new());
//! for &x in &xs {
//!     for &y in &ys {
//!         xc.push(x);
//!         yc.push(y);
//!         fc.push(x.sin() + y * y);
//!     }
//! }
//! let time: Vec<f64> = (0..xc.len()).map(|i| i as f64).collect();
//! let dataset = Dataset::new(
//!     TimeAxis::new(time).unwrap(),
//!     [
//!         NumericColumn::new(Identifier::new("x").unwrap(), xc),
//!         NumericColumn::new(Identifier::new("y").unwrap(), yc),
//!         NumericColumn::new(Identifier::new("f").unwrap(), fc),
//!     ],
//! )
//! .unwrap();
//!
//! let report = detect_reductions(&dataset, &ReduceConfig::with_target("f")).unwrap();
//! assert!(report.grid.is_reconstructed());
//! let additive = report
//!     .separabilities
//!     .iter()
//!     .find(|s| s.kind == SeparabilityKind::Additive)
//!     .expect("additive separability detected");
//! assert_eq!(additive.group_a, vec!["x".to_string()]);
//! assert_eq!(additive.group_b, vec!["y".to_string()]);
//! assert!(additive.reconstruction_residual < 1e-3);
//! ```

mod config;
mod detect;
mod error;
mod grid;
mod report;
mod separability;
mod symmetry;

pub use config::ReduceConfig;
pub use detect::detect_reductions;
pub use error::ReduceError;
pub use report::{
    GridStatus, ReductionReport, Separability, SeparabilityKind, Symmetry, SymmetryKind,
};
