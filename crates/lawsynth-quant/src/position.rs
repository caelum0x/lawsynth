use lawsynth_core::{Identifier, stable_hash};

use crate::{Money, QuantError};

const POSITION_MAGIC: &[u8; 5] = b"LSQP1";
const FIXED_POSITION_BYTES: usize = 5 + 2 + 8;

/// Whether a position is net long, net short, or flat.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Direction {
    Flat,
    Long,
    Short,
}

/// A signed holding of one instrument, measured in exact integer units.
///
/// Valuation delegates to [`Money`]'s overflow-checked integer algebra, so a
/// position never introduces rounding, binary floating point, or silent
/// wrapping. Quantity is a signed unit count: positive is long, negative is
/// short, zero is flat.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    instrument: Identifier,
    quantity: i64,
}

impl Position {
    pub fn new(instrument: impl Into<String>, quantity: i64) -> Result<Self, QuantError> {
        let instrument = Identifier::new(instrument)?;
        if instrument.as_str().len() > usize::from(u16::MAX) {
            return Err(QuantError::InstrumentTooLong { bytes: instrument.as_str().len() });
        }
        Ok(Self { instrument, quantity })
    }

    pub fn instrument(&self) -> &Identifier {
        &self.instrument
    }

    pub const fn quantity(&self) -> i64 {
        self.quantity
    }

    pub const fn is_flat(&self) -> bool {
        self.quantity == 0
    }

    pub const fn direction(&self) -> Direction {
        if self.quantity > 0 {
            Direction::Long
        } else if self.quantity < 0 {
            Direction::Short
        } else {
            Direction::Flat
        }
    }

    /// Signed mark-to-market value at `price` per unit: exactly `price * quantity`.
    pub fn market_value(&self, price: Money) -> Result<Money, QuantError> {
        price.checked_mul(i128::from(self.quantity))
    }

    /// Absolute exposure (gross notional) at `price`, ignoring position sign.
    pub fn notional(&self, price: Money) -> Result<Money, QuantError> {
        self.market_value(price)?.checked_abs()
    }

    /// Cash impact of establishing the position at `price`: the negation of its
    /// market value. Going long is a cash outflow; going short is an inflow.
    pub fn establish_cash_flow(&self, price: Money) -> Result<Money, QuantError> {
        self.market_value(price)?.checked_neg()
    }

    /// Net two holdings of the same instrument by adding their signed quantities.
    /// Rejects a differing instrument and quantity overflow rather than wrapping.
    pub fn combine(&self, other: &Self) -> Result<Self, QuantError> {
        if self.instrument != other.instrument {
            return Err(QuantError::InstrumentMismatch {
                left: self.instrument.as_str().to_owned(),
                right: other.instrument.as_str().to_owned(),
            });
        }
        let quantity =
            self.quantity.checked_add(other.quantity).ok_or(QuantError::ArithmeticOverflow)?;
        Ok(Self { instrument: self.instrument.clone(), quantity })
    }

    /// The offsetting position that flattens this one (negated quantity),
    /// rejecting the single `i64::MIN` boundary rather than wrapping.
    pub fn reverse(&self) -> Result<Self, QuantError> {
        let quantity = self.quantity.checked_neg().ok_or(QuantError::ArithmeticOverflow)?;
        Ok(Self { instrument: self.instrument.clone(), quantity })
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let instrument = self.instrument.as_str().as_bytes();
        let mut bytes = Vec::with_capacity(FIXED_POSITION_BYTES + instrument.len());
        bytes.extend_from_slice(POSITION_MAGIC);
        bytes.extend_from_slice(&(instrument.len() as u16).to_be_bytes());
        bytes.extend_from_slice(instrument);
        bytes.extend_from_slice(&self.quantity.to_be_bytes());
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, QuantError> {
        if bytes.len() < FIXED_POSITION_BYTES + 1 {
            return Err(QuantError::InvalidEncoding("position encoding is truncated"));
        }
        if &bytes[..5] != POSITION_MAGIC {
            return Err(QuantError::InvalidEncoding("unsupported position encoding version"));
        }
        let instrument_len = usize::from(u16::from_be_bytes([bytes[5], bytes[6]]));
        let expected_len = FIXED_POSITION_BYTES
            .checked_add(instrument_len)
            .ok_or(QuantError::InvalidEncoding("position length overflow"))?;
        if bytes.len() != expected_len {
            return Err(QuantError::InvalidEncoding(
                "position length does not match instrument length",
            ));
        }
        let instrument_end = 7 + instrument_len;
        let instrument = std::str::from_utf8(&bytes[7..instrument_end])
            .map_err(|_| QuantError::InvalidEncoding("instrument identifier is not UTF-8"))?;
        let quantity = i64::from_be_bytes(
            bytes[instrument_end..]
                .try_into()
                .map_err(|_| QuantError::InvalidEncoding("invalid position quantity"))?,
        );
        Self::new(instrument, quantity)
    }

    pub fn stable_fingerprint(&self) -> u64 {
        stable_hash(self.canonical_bytes())
    }
}
