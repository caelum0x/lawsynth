# Dataset contract changelog

## Current implementation

- Dense finite `f64` time-series datasets with canonical identifier order.
- Deterministic fingerprints include timestamp bits, values, identifiers, and
  optional unit strings.
- Deterministic owned batches and complete sliding windows.
- A bounded local Parquet reader for uncompressed, required, PLAIN numeric
  columns; its limits are contractual, not an approximation of full Parquet.

Compatibility changes to these invariants require a new fingerprint domain
version or an explicit migration path.
