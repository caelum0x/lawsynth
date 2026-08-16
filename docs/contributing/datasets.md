# Dataset contributions

The local core's dataset boundary is finite numeric columns sampled on a
strictly increasing finite time axis. Preserve original observations and
return validation errors instead of imputing, sorting, or coercing input
silently.

CLI ingestion currently reads simple comma-separated text with one header row;
it does not parse quoted values, missing values, categorical data, or page
encoded data files. Normalize those inputs outside the CLI or implement the
full parser and its validation/tests.

When adding fixtures, include the generating equation or source, units,
sampling interval, noise model if any, and expected tolerances. Keep fixtures
small enough for tests and deterministic enough for regression review. A
Parquet input path is only valid for the implemented uncompressed flat numeric
PLAIN subset; unsupported encodings must continue to fail explicitly.
