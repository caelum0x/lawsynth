//! Regime detection for piecewise stationary scalar signals and finite-state models.
//!
//! All algorithms are deterministic.  PELT's public implementation retains the
//! exact dynamic-programming recurrence; pruning can be introduced only with a
//! proven cost condition, never by silently changing segmentation results.
pub mod binary;
pub mod bocpd;
pub mod config;
pub mod cost;
pub mod error;
pub mod hmm;
pub mod pelt;
pub mod regime_laws;
pub mod segmentation;
pub mod transitions;

pub use binary::{BinarySplit, best_binary_split};
pub use bocpd::{BocpdConfig, BocpdPoint, bocpd};
pub use config::SegmentationConfig;
pub use cost::{segment_cost, segment_moments};
pub use error::{RegimeError, Result};
pub use hmm::{DiscreteHmm, ViterbiPath};
pub use pelt::pelt;
pub use regime_laws::{AffineLaw, RegimeLawBook};
pub use segmentation::{Segment, Segmentation};
pub use transitions::TransitionMatrix;
