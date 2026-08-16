//! Derivative estimation with deterministic methods appropriate for discovery.

mod config;
mod error;
mod finite;
mod irregular;
mod method;
mod savgol;
mod spectral;
mod spline;
mod tvreg;
mod weak_form;

pub use config::DerivativeConfig;
pub use error::DifferentiationError;
pub use finite::{differentiate_dataset, differentiate_dataset_with_config, differentiate_series};
pub use irregular::irregular_three_point_derivative;
pub use method::DerivativeMethod;
pub use savgol::savgol_series;
pub use spectral::spectral_derivative;
pub use spline::cubic_spline_derivative;
pub use tvreg::{tvreg_series, tvreg_smoothed_series};
pub use weak_form::weak_derivative_integral;
