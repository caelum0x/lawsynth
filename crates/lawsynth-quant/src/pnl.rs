use lawsynth_core::stable_hash;

use crate::{Money, Position, QuantError};

const LOT_MAGIC: &[u8; 5] = b"LSQL1";
/// Fixed-width money segment written by [`Money::canonical_bytes`].
const LOT_MONEY_LEN: usize = 24;
const LOT_HEADER_BYTES: usize = 5 + LOT_MONEY_LEN;

/// An executed lot: a [`Position`] acquired at a known per-unit entry price.
///
/// A lot is the smallest unit of profit-and-loss accounting. Both its cost basis
/// and its mark-to-market profit delegate to [`Money`]'s overflow-checked integer
/// algebra, so P&L is exact: no rounding, no binary floating point, and no silent
/// wrapping. The entry price and any mark price must share a currency; a mismatch
/// is rejected rather than silently converted.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Lot {
    position: Position,
    entry_price: Money,
}

impl Lot {
    pub fn new(position: Position, entry_price: Money) -> Self {
        Self { position, entry_price }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub const fn entry_price(&self) -> Money {
        self.entry_price
    }

    /// Signed cost basis: the position's market value at the entry price
    /// (exactly `entry_price * quantity`), overflow-checked.
    pub fn entry_value(&self) -> Result<Money, QuantError> {
        self.position.market_value(self.entry_price)
    }

    /// Signed mark-to-market value of the lot at the current `mark` price.
    pub fn market_value(&self, mark: Money) -> Result<Money, QuantError> {
        self.position.market_value(mark)
    }

    /// Unrealized profit and loss at `mark`: exactly `quantity * (mark - entry)`.
    ///
    /// The per-unit price move is taken with [`Money::checked_sub`], so a currency
    /// mismatch between `mark` and the entry price is rejected rather than
    /// converted. The signed quantity makes the result correct in both
    /// directions: a long profits when the mark rises, a short profits when it
    /// falls. Overflow surfaces as an error instead of wrapping.
    pub fn unrealized_pnl(&self, mark: Money) -> Result<Money, QuantError> {
        let per_unit = mark.checked_sub(self.entry_price)?;
        per_unit.checked_mul(i128::from(self.position.quantity()))
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let position = self.position.canonical_bytes();
        let money = self.entry_price.canonical_bytes();
        let mut bytes = Vec::with_capacity(LOT_HEADER_BYTES + position.len());
        bytes.extend_from_slice(LOT_MAGIC);
        bytes.extend_from_slice(&money);
        bytes.extend_from_slice(&position);
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, QuantError> {
        if bytes.len() < LOT_HEADER_BYTES {
            return Err(QuantError::InvalidEncoding("lot encoding is truncated"));
        }
        if &bytes[..5] != LOT_MAGIC {
            return Err(QuantError::InvalidEncoding("unsupported lot encoding version"));
        }
        let entry_price = Money::from_canonical_bytes(&bytes[5..LOT_HEADER_BYTES])?;
        let position = Position::from_canonical_bytes(&bytes[LOT_HEADER_BYTES..])?;
        Ok(Self { position, entry_price })
    }

    pub fn stable_fingerprint(&self) -> u64 {
        stable_hash(self.canonical_bytes())
    }
}
