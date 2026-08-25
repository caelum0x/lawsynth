# Quant foundation changelog

## Unreleased

- Extend the exact money contract with overflow-checked negation, absolute
  value, and integer scaling, plus a zero check; no rounding or FX is introduced.
- Add a single-position contract (direction, market value, notional, cash flow,
  netting, reverse) that reuses the exact money integer algebra and its own
  deterministic versioned byte encoding; no FX or multi-position aggregation.

## 0.1.0

- Define the initial closed currency set and exact minor-unit money contract.
- Define UTC millisecond observation identity with explicit same-time sequence.
- Define deterministic versioned byte encodings and strict decoders.
