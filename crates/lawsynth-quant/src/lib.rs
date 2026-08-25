//! Deterministic quantitative-finance foundation contracts.
//!
//! QR0 begins with exact money and explicit UTC observation identity. Broader
//! market-data, calendar, corporate-action, portfolio, and experiment contracts
//! are intentionally not implied by this crate's first release.

mod currency;
mod error;
mod money;
mod observation;

pub use currency::Currency;
pub use error::QuantError;
pub use money::Money;
pub use observation::{ObservationKey, UtcTimestamp};
