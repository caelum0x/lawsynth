//! Deterministic quantitative-finance foundation contracts.
//!
//! QR0 begins with exact money, explicit UTC observation identity, and
//! single-position valuation built on that exact-integer money algebra. Broader
//! market-data, calendar, corporate-action, multi-position portfolio, and
//! experiment contracts are intentionally not implied by this crate's first
//! release.

mod currency;
mod error;
mod money;
mod observation;
mod position;

pub use currency::Currency;
pub use error::QuantError;
pub use money::Money;
pub use observation::{ObservationKey, UtcTimestamp};
pub use position::{Direction, Position};
