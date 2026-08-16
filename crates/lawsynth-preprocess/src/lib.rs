//! Reproducible numerical preprocessing transforms.

mod align;
mod config;
mod detrend;
mod error;
mod impute;
mod pipeline;
mod resample;
mod scale;
mod smooth;
mod transform;

pub use align::align_series_linear;
pub use config::PreprocessConfig;
pub use detrend::{DetrendReport, detrend_linear};
pub use error::PreprocessError;
pub use impute::{ImputationMethod, ImputationReport, impute_series};
pub use pipeline::PreprocessPipeline;
pub use resample::{ResampleReport, resample_linear, resample_linear_with_report};
pub use scale::{ScaleReport, standardize, unstandardize};
pub use smooth::{PreprocessReport, moving_average};
pub use transform::{AppliedTransform, PreprocessStep};
