//! Typed deterministic feature libraries used by sparse and symbolic discovery.

mod config;
mod constraints;
mod delay;
mod error;
mod interaction;
mod library;
mod partition;
mod polynomial;
mod rational;
mod term;
mod trigonometric;

pub use config::FeatureConfig;
pub use constraints::FeatureConstraint;
pub use delay::{DelayEmbedding, delayed_columns};
pub use error::FeatureError;
pub use library::{FeatureLibrary, FeatureMatrix};
pub use partition::row_partitions;
pub use term::FeatureTerm;
