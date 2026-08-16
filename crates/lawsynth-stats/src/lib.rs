//! Reproducible statistical primitives used by discovery and uncertainty.

mod bootstrap;
mod config;
mod covariance;
mod distributions;
mod error;
mod information;
mod moments;
mod quantile;
mod robust;
mod sampling;

pub use bootstrap::{BootstrapConfig, PercentileInterval, bootstrap_indices, percentile_interval};
pub use config::HistogramConfig;
pub use covariance::{covariance, pearson_correlation};
pub use distributions::{normal_cdf, normal_pdf};
pub use error::StatsError;
pub use information::histogram_mutual_information;
pub use moments::{MomentSummary, moments};
pub use quantile::{median, quantile, quantile_sorted};
pub use robust::{median_absolute_deviation, winsorize};
pub use sampling::sample_without_replacement;
