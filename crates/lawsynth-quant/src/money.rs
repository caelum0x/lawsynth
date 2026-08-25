use std::str::FromStr;

use lawsynth_core::stable_hash;

use crate::{Currency, QuantError};

const MONEY_MAGIC: &[u8; 5] = b"LSQM1";
const MONEY_ENCODED_LEN: usize = 24;

/// An exact signed amount in the currency's declared minor unit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Money {
    currency: Currency,
    minor_units: i128,
}

impl Money {
    pub const fn from_minor_units(currency: Currency, minor_units: i128) -> Self {
        Self { currency, minor_units }
    }

    pub const fn currency(self) -> Currency {
        self.currency
    }

    pub const fn minor_units(self) -> i128 {
        self.minor_units
    }

    pub fn checked_add(self, other: Self) -> Result<Self, QuantError> {
        self.require_same_currency(other)?;
        self.minor_units
            .checked_add(other.minor_units)
            .map(|minor_units| Self { currency: self.currency, minor_units })
            .ok_or(QuantError::ArithmeticOverflow)
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, QuantError> {
        self.require_same_currency(other)?;
        self.minor_units
            .checked_sub(other.minor_units)
            .map(|minor_units| Self { currency: self.currency, minor_units })
            .ok_or(QuantError::ArithmeticOverflow)
    }

    /// Exactly scale the amount by an integer factor (e.g. quantity of units),
    /// preserving the currency and rejecting overflow rather than wrapping.
    pub fn checked_mul(self, factor: i128) -> Result<Self, QuantError> {
        self.minor_units
            .checked_mul(factor)
            .map(|minor_units| Self { currency: self.currency, minor_units })
            .ok_or(QuantError::ArithmeticOverflow)
    }

    /// Negate the amount, rejecting the single `i128::MIN` overflow case.
    pub fn checked_neg(self) -> Result<Self, QuantError> {
        self.minor_units
            .checked_neg()
            .map(|minor_units| Self { currency: self.currency, minor_units })
            .ok_or(QuantError::ArithmeticOverflow)
    }

    /// Absolute magnitude in the same currency, rejecting the `i128::MIN` case.
    pub fn checked_abs(self) -> Result<Self, QuantError> {
        self.minor_units
            .checked_abs()
            .map(|minor_units| Self { currency: self.currency, minor_units })
            .ok_or(QuantError::ArithmeticOverflow)
    }

    pub const fn is_zero(self) -> bool {
        self.minor_units == 0
    }

    pub fn canonical_bytes(self) -> [u8; MONEY_ENCODED_LEN] {
        let mut bytes = [0_u8; MONEY_ENCODED_LEN];
        bytes[..5].copy_from_slice(MONEY_MAGIC);
        bytes[5..8].copy_from_slice(self.currency.code().as_bytes());
        bytes[8..].copy_from_slice(&self.minor_units.to_be_bytes());
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, QuantError> {
        if bytes.len() != MONEY_ENCODED_LEN {
            return Err(QuantError::InvalidEncoding("money length must be exactly 24 bytes"));
        }
        if &bytes[..5] != MONEY_MAGIC {
            return Err(QuantError::InvalidEncoding("unsupported money encoding version"));
        }
        let code = std::str::from_utf8(&bytes[5..8])
            .map_err(|_| QuantError::InvalidEncoding("currency code is not UTF-8"))?;
        let currency = Currency::from_str(code)?;
        let minor_units = i128::from_be_bytes(
            bytes[8..]
                .try_into()
                .map_err(|_| QuantError::InvalidEncoding("invalid money amount"))?,
        );
        Ok(Self { currency, minor_units })
    }

    pub fn stable_fingerprint(self) -> u64 {
        stable_hash(self.canonical_bytes())
    }

    fn require_same_currency(self, other: Self) -> Result<(), QuantError> {
        if self.currency == other.currency {
            Ok(())
        } else {
            Err(QuantError::CurrencyMismatch { left: self.currency, right: other.currency })
        }
    }
}
