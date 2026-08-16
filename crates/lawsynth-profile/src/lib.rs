//! Deterministic quality and distribution summaries for numerical datasets.

mod column_profile;
mod config;
mod delays;
mod dependence;
mod distribution;
mod error;
mod missingness;
mod profiler;
mod quality_flags;
mod time_profile;

pub use column_profile::ColumnProfile;
pub use config::ProfileConfig;
pub use delays::{DelayEstimate, estimate_delay};
pub use dependence::pearson_correlation;
pub use distribution::{DistributionProfile, distribution};
pub use error::ProfileError;
pub use missingness::{MissingnessProfile, profile_f64_missingness, profile_missingness};
pub use profiler::{DatasetProfile, profile, profile_with_config};
pub use quality_flags::{ColumnQuality, quality_flags};
pub use time_profile::TimeProfile;
