# Quant foundation boundary

Status: QR0 implementation slice 1. This specification covers the exact money,
observation-identity, and single-position valuation primitives compiled in
`lawsynth-quant`. Trading calendars, corporate actions, market-data tables,
multi-position portfolio accounting, leakage checks, fixture licensing, and
complete experiment manifests remain unimplemented and MUST NOT be claimed from
this initial slice.

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

## Determinism and non-goals

Encoding is independent of locale, machine endianness, wall clock, and hash-map
order. `stable_fingerprint` is the existing deterministic FNV-1a identifier for
fixtures and in-process comparisons; it is not a cryptographic checksum and
MUST NOT replace SHA-256 in governed experiment artifacts.

This slice does not define prices, returns, asset identifiers, trading days,
day-count conventions, FX rates, rounding policy, portfolio accounting, or live
market connectivity.
