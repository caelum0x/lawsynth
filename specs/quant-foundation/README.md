# Quant foundation boundary

Status: QR0 implementation slice 1. This specification covers the exact money,
observation-identity, single-position valuation, exact mark-to-market
profit-and-loss, and single-currency multi-instrument portfolio-aggregation
primitives compiled in `lawsynth-quant`. Trading calendars, corporate actions,
market-data tables, realized-P&L and lot-matched fill accounting, leakage checks,
fixture licensing, and complete experiment manifests remain unimplemented and
MUST NOT be claimed from this initial slice.

## Money

`Currency` is a versioned closed set for the initial end-of-day research
boundary: USD, EUR, TRY, GBP, CHF, and JPY. Unknown codes are rejected rather
than guessed. Each currency defines its ISO-style minor-unit exponent: JPY uses
zero and the others use two.

`Money` stores a signed `i128` count of minor units plus one `Currency`. It never
uses binary floating point. Addition and subtraction MUST reject currency
mismatches and arithmetic overflow. Negation, absolute value, and exact integer
scaling (for example a quantity of units) preserve the currency and MUST likewise
reject overflow rather than wrapping, including the single `i128::MIN` boundary.
Scaling is by an integer factor only: it introduces no rounding and no fractional
minor units. Callers are responsible for supplying a separate, sourced FX
conversion policy; the crate does not silently convert.

Canonical money bytes are:

```text
"LSQM1" | 3-byte currency code | signed i128 minor units (big endian)
```

Readers reject unknown versions, unsupported currencies, and any trailing or
missing bytes.

## Observation identity

`UtcTimestamp` is a signed Unix millisecond count. It represents an already
resolved UTC instant: local wall time, daylight-saving rules, exchange sessions,
and observation cutoffs are deliberately not inferred.

`ObservationKey` combines a validated portable instrument identifier, one UTC
timestamp, and a sequence number. The sequence makes multiple events at one
millisecond explicit. Ordering is lexicographic by instrument, timestamp, then
sequence.

Canonical observation bytes are:

```text
"LSQO1" | u16 instrument-byte length (big endian) | instrument UTF-8
         | signed i64 Unix milliseconds (big endian) | u32 sequence (big endian)
```

Instrument identifiers use the existing `lawsynth-core::Identifier` grammar and
are limited to 65,535 bytes by this encoding. Decoders reject malformed UTF-8,
invalid identifiers, length mismatches, unknown versions, and trailing bytes.

## Position

`Direction` is the net sign of a holding: long, short, or flat. `Position`
combines a validated portable instrument identifier with a signed `i64` unit
count. It never uses binary floating point.

Valuation reuses the exact `Money` integer algebra rather than defining a second
one. At a per-unit `Money` price:

- `market_value` is exactly `price * quantity` (overflow-checked scaling).
- `notional` is its absolute magnitude, ignoring sign.
- `establish_cash_flow` is its negation: going long is a cash outflow, going
  short is an inflow.

`combine` nets two holdings of the same instrument by adding signed quantities;
it rejects a differing instrument and rejects quantity overflow. `reverse`
returns the offsetting position that flattens this one, rejecting the single
`i64::MIN` boundary rather than wrapping. No FX conversion, price sourcing, or
rounding is introduced; multi-instrument portfolio aggregation is out of scope.

Canonical position bytes are:

```text
"LSQP1" | u16 instrument-byte length (big endian) | instrument UTF-8
         | signed i64 quantity (big endian)
```

Decoders reject malformed UTF-8, invalid identifiers, length mismatches, unknown
versions, and trailing bytes.

## Lot and mark-to-market P&L

A `Lot` is one executed `Position` paired with a per-unit `Money` entry price: the
smallest unit of profit-and-loss accounting. Both its cost basis and its mark
profit reuse the exact `Money` integer algebra rather than defining a second one,
so P&L introduces no rounding, no binary floating point, and no silent wrapping.

At a per-unit `Money` mark price:

- `entry_value` is the signed cost basis, exactly `entry_price * quantity`.
- `market_value` is the signed current value, exactly `mark * quantity`.
- `unrealized_pnl` is exactly `quantity * (mark - entry_price)`. The per-unit
  price move is taken with overflow-checked subtraction, so a currency mismatch
  between the mark and the entry price is rejected rather than converted. The
  signed quantity makes the result correct in both directions: a long profits
  when the mark rises, a short profits when it falls.

Realized P&L, average-cost or lot-matched accumulation across multiple fills, FX
conversion, financing, and fees are out of scope for this slice; multi-instrument
aggregation is provided by `Portfolio` below.

Canonical lot bytes are:

```text
"LSQL1" | 24-byte entry-price money segment ("LSQM1"...) | position bytes ("LSQP1"...)
```

The fixed-width entry price precedes the variable-length position so the decoder
can split the two without a separate length prefix. Decoders reject unknown
versions, truncated input, and any malformed money or position segment.

## Portfolio aggregation

A `Portfolio` is an ordered book of `Lot`s that all report in one currency: the
first multi-instrument aggregation in this foundation. Every total reuses the
exact `Money` integer algebra rather than defining a second one, so a portfolio
figure introduces no rounding, no binary floating point, and no silent wrapping.

Construction fixes the single reporting currency. `with_lot` returns a new
portfolio with the lot appended (order preserved), and `from_lots` builds one
from an ordered sequence. A lot whose entry price is not in the portfolio
currency is rejected rather than silently converted; there is no FX in this
slice.

Mark-to-market lookups are supplied by the caller as a per-instrument resolver,
so the portfolio never sources or infers a price. An instrument the resolver
cannot price is rejected rather than defaulted to zero. Against those marks:

- `cost_basis` is the exact sum of every lot's entry value; an empty portfolio is
  exactly zero in its currency.
- `market_value` is the exact sum of every lot's `mark * quantity`.
- `gross_notional` is the exact sum of every lot's absolute exposure, so long and
  short magnitudes add rather than cancel.
- `unrealized_pnl` is the exact sum of every lot's `quantity * (mark - entry)`. A
  currency mismatch between a mark and its lot is rejected, and any summation or
  scaling overflow surfaces as an error rather than wrapping.

Canonical portfolio bytes are:

```text
"LSQF1" | 3-byte currency code | u32 lot count (big endian)
        | repeated { u32 lot-byte length (big endian) | lot bytes ("LSQL1"...) }
```

Each lot carries a length prefix so the variable-length lots are self-delimiting.
Decoders reject unknown versions, unsupported currencies, a truncated header,
truncated length prefixes or lot segments, a lot count larger than the payload, a
lot whose currency differs from the portfolio, and any trailing bytes. Marks are
transient inputs and are not part of the encoded state. Realized P&L,
lot-matched fill accounting, FX conversion, financing, and fees remain out of
scope.

## Determinism and non-goals

Encoding is independent of locale, machine endianness, wall clock, and hash-map
order. `stable_fingerprint` is the existing deterministic FNV-1a identifier for
fixtures and in-process comparisons; it is not a cryptographic checksum and
MUST NOT replace SHA-256 in governed experiment artifacts.

This slice does not define prices, returns, asset identifiers, trading days,
day-count conventions, FX rates, rounding policy, realized-P&L or lot-matched fill
accounting, or live market connectivity. Portfolio aggregation is limited to
single-currency valuation totals over the lots a caller supplies.
