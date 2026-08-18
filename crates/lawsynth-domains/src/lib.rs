//! Curated, self-validated domain presets for LawSynth discovery.
//!
//! A [`DomainPreset`] makes LawSynth usable out-of-the-box for a common
//! scientific domain. Each preset bundles a candidate feature-library
//! configuration, an optional structural [`template prior`](lawsynth_discovery::TemplatePrior),
//! honestly-expressible unit hints, and a documented [`ReferenceSystem`] — the
//! canonical governing law together with a deterministic fixed-step RK4 trajectory
//! generator.
//!
//! # The preset contract
//!
//! Every shipped preset is **self-validated by round-trip recovery**: integrating
//! its reference law into a clean trajectory and running discovery with the
//! preset's own [`discovery_config`](DomainPreset::discovery_config) recovers that
//! same law — coefficients to a tight tolerance and matching term structure. The
//! crate's integration tests assert this for each preset; a preset that cannot
//! recover its own law is not shipped.
//!
//! Everything is **deterministic and offline**: preset lookup, trajectory
//! generation, and discovery are pure functions of their inputs, so identical
//! inputs yield bit-identical outputs. The crate is std-only with internal path
//! dependencies.
//!
//! # Honest limits
//!
//! A preset is a **starting point, not a guarantee**. It is tuned on clean
//! synthetic data generated from a standard textbook law. Real measurements carry
//! noise, sampling irregularity, and unmodeled effects; recovering a law from them
//! generally needs the noise-handling and smoothing knobs on
//! [`DiscoveryConfig`](lawsynth_discovery::DiscoveryConfig) and user judgement. A
//! preset shrinks and centers the search; it does not certify the result.
//!
//! # Example
//!
//! ```
//! use lawsynth_domains::{ReferenceSystem, preset};
//! use lawsynth_discovery::discover;
//!
//! let lotka = preset("lotka-volterra").expect("registered preset");
//! let data = lotka.reference().trajectory();
//! let result = discover(&data, &lotka.discovery_config()).expect("discovery runs");
//! assert!(!result.candidates.is_empty());
//! # let _: &ReferenceSystem = lotka.reference();
//! ```

mod catalog;
mod preset;
mod reference;
mod registry;

pub use preset::{DomainPreset, UnitHint};
pub use reference::{ReferenceLaw, ReferenceSystem, ReferenceTerm};
pub use registry::{DomainPresetKind, PresetError, all, names, preset};
