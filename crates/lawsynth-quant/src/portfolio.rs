use std::str::FromStr;

use lawsynth_core::{Identifier, stable_hash};

use crate::{Currency, Lot, Money, QuantError};

const PORTFOLIO_MAGIC: &[u8; 5] = b"LSQF1";
/// Magic (5) + 3-byte currency code + u32 lot count.
const PORTFOLIO_HEADER_BYTES: usize = 5 + 3 + 4;
/// Big-endian byte width of each lot's length prefix inside the encoding.
const LOT_LENGTH_PREFIX_BYTES: usize = 4;

/// An ordered book of [`Lot`]s that all report in one currency.
///
/// A portfolio is the first multi-instrument aggregation in the quant
/// foundation: it sums cost basis, mark-to-market value, gross notional, and
/// unrealized profit across every lot it holds. All aggregation delegates to
/// [`Money`]'s overflow-checked integer algebra, so a portfolio total is exact —
/// no rounding, no binary floating point, and no silent wrapping.
///
/// Every lot must report in the portfolio's single currency; a differing lot is
/// rejected rather than silently converted. Mark-to-market lookups are supplied
/// by the caller per instrument, so the portfolio never sources or infers a
/// price. Realized profit, average-cost or lot-matched accumulation across
/// fills, FX conversion, financing, and fees remain out of scope.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Portfolio {
    currency: Currency,
    lots: Vec<Lot>,
}

impl Portfolio {
    /// An empty portfolio reporting in `currency`. Its cost basis and totals are
    /// exactly zero in that currency until lots are added.
    pub const fn new(currency: Currency) -> Self {
        Self { currency, lots: Vec::new() }
    }

    /// Build a portfolio from an ordered list of lots, rejecting any lot whose
    /// entry price is not in `currency` rather than converting it.
    pub fn from_lots(
        currency: Currency,
        lots: impl IntoIterator<Item = Lot>,
    ) -> Result<Self, QuantError> {
        let mut portfolio = Self::new(currency);
        for lot in lots {
            portfolio = portfolio.with_lot(lot)?;
        }
        Ok(portfolio)
    }

    /// Return a new portfolio with `lot` appended, preserving order. The lot's
    /// entry price must match the portfolio currency; a mismatch is rejected.
    pub fn with_lot(&self, lot: Lot) -> Result<Self, QuantError> {
        let lot_currency = lot.entry_price().currency();
        if lot_currency != self.currency {
            return Err(QuantError::CurrencyMismatch { left: self.currency, right: lot_currency });
        }
        let mut lots = self.lots.clone();
        lots.push(lot);
        Ok(Self { currency: self.currency, lots })
    }

    pub const fn currency(&self) -> Currency {
        self.currency
    }

    pub fn lots(&self) -> &[Lot] {
        &self.lots
    }

    pub fn len(&self) -> usize {
        self.lots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lots.is_empty()
    }

    /// Total signed cost basis: the exact sum of every lot's entry value
    /// (`entry_price * quantity`). An empty portfolio is exactly zero.
    pub fn cost_basis(&self) -> Result<Money, QuantError> {
        self.fold_lots(|lot| lot.entry_value())
    }

    /// Total signed mark-to-market value: the exact sum of every lot's value at
    /// its instrument's mark. `mark_for` resolves the per-unit price for an
    /// instrument; an unresolved instrument is rejected, never priced at zero.
    pub fn market_value<F>(&self, mark_for: F) -> Result<Money, QuantError>
    where
        F: Fn(&Identifier) -> Option<Money>,
    {
        self.fold_lots(|lot| lot.market_value(mark_for_lot(lot, &mark_for)?))
    }

    /// Total gross notional: the exact sum of every lot's absolute exposure,
    /// ignoring position sign. Long and short magnitudes add rather than cancel.
    pub fn gross_notional<F>(&self, mark_for: F) -> Result<Money, QuantError>
    where
        F: Fn(&Identifier) -> Option<Money>,
    {
        self.fold_lots(|lot| lot.market_value(mark_for_lot(lot, &mark_for)?)?.checked_abs())
    }

    /// Total unrealized profit and loss: the exact sum of every lot's
    /// `quantity * (mark - entry)`. A currency mismatch between a mark and its
    /// lot is rejected rather than converted, and overflow surfaces as an error.
    pub fn unrealized_pnl<F>(&self, mark_for: F) -> Result<Money, QuantError>
    where
        F: Fn(&Identifier) -> Option<Money>,
    {
        self.fold_lots(|lot| lot.unrealized_pnl(mark_for_lot(lot, &mark_for)?))
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(PORTFOLIO_HEADER_BYTES);
        bytes.extend_from_slice(PORTFOLIO_MAGIC);
        bytes.extend_from_slice(self.currency.code().as_bytes());
        bytes.extend_from_slice(&(self.lots.len() as u32).to_be_bytes());
        for lot in &self.lots {
            let lot_bytes = lot.canonical_bytes();
            bytes.extend_from_slice(&(lot_bytes.len() as u32).to_be_bytes());
            bytes.extend_from_slice(&lot_bytes);
        }
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, QuantError> {
        if bytes.len() < PORTFOLIO_HEADER_BYTES {
            return Err(QuantError::InvalidEncoding("portfolio encoding is truncated"));
        }
        if &bytes[..5] != PORTFOLIO_MAGIC {
            return Err(QuantError::InvalidEncoding("unsupported portfolio encoding version"));
        }
        let code = std::str::from_utf8(&bytes[5..8])
            .map_err(|_| QuantError::InvalidEncoding("portfolio currency code is not UTF-8"))?;
        let currency = Currency::from_str(code)?;
        let count = u32::from_be_bytes(
            bytes[8..PORTFOLIO_HEADER_BYTES]
                .try_into()
                .map_err(|_| QuantError::InvalidEncoding("invalid portfolio lot count"))?,
        );

        let mut lots = Vec::with_capacity(count as usize);
        let mut offset = PORTFOLIO_HEADER_BYTES;
        for _ in 0..count {
            let prefix_end = offset
                .checked_add(LOT_LENGTH_PREFIX_BYTES)
                .filter(|end| *end <= bytes.len())
                .ok_or(QuantError::InvalidEncoding("portfolio lot length prefix is truncated"))?;
            let lot_len = u32::from_be_bytes(
                bytes[offset..prefix_end]
                    .try_into()
                    .map_err(|_| QuantError::InvalidEncoding("invalid portfolio lot length"))?,
            ) as usize;
            let lot_end = prefix_end
                .checked_add(lot_len)
                .filter(|end| *end <= bytes.len())
                .ok_or(QuantError::InvalidEncoding("portfolio lot segment is truncated"))?;
            let lot = Lot::from_canonical_bytes(&bytes[prefix_end..lot_end])?;
            let lot_currency = lot.entry_price().currency();
            if lot_currency != currency {
                return Err(QuantError::CurrencyMismatch { left: currency, right: lot_currency });
            }
            lots.push(lot);
            offset = lot_end;
        }
        if offset != bytes.len() {
            return Err(QuantError::InvalidEncoding("portfolio has trailing bytes"));
        }
        Ok(Self { currency, lots })
    }

    pub fn stable_fingerprint(&self) -> u64 {
        stable_hash(self.canonical_bytes())
    }

    /// Fold every lot into a running total seeded with zero in the portfolio
    /// currency, using the exact overflow-checked `Money` addition.
    fn fold_lots<F>(&self, mut value: F) -> Result<Money, QuantError>
    where
        F: FnMut(&Lot) -> Result<Money, QuantError>,
    {
        let mut total = Money::from_minor_units(self.currency, 0);
        for lot in &self.lots {
            total = total.checked_add(value(lot)?)?;
        }
        Ok(total)
    }
}

/// Resolve the per-unit mark for a lot's instrument, rejecting an unpriced
/// instrument rather than defaulting it to zero.
fn mark_for_lot<F>(lot: &Lot, mark_for: &F) -> Result<Money, QuantError>
where
    F: Fn(&Identifier) -> Option<Money>,
{
    let instrument = lot.position().instrument();
    mark_for(instrument)
        .ok_or_else(|| QuantError::MissingMark { instrument: instrument.as_str().to_owned() })
}
