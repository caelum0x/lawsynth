//! Deterministic quantitative-finance foundation contracts.
//!
//! QR0 begins with exact money, explicit UTC observation identity,
//! single-position valuation, exact mark-to-market profit and loss, and a
//! single-currency multi-instrument portfolio that aggregates those lots — all
//! built on the same exact-integer money algebra. Broader market-data, calendar,
//! corporate-action, and experiment contracts are intentionally not implied by
//! this crate's first release.

mod currency;
mod error;
mod money;
mod observation;
mod pnl;
mod portfolio;
mod position;

pub use currency::Currency;
pub use error::QuantError;
pub use money::Money;
pub use observation::{ObservationKey, UtcTimestamp};
pub use pnl::Lot;
pub use portfolio::Portfolio;
pub use position::{Direction, Position};
