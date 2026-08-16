# Missingness

The `Dataset` boundary is finite and dense. `NaN`, positive infinity, and
negative infinity are rejected for both timestamps and numeric values. There
is no in-dataset null bitmap or missing-value sentinel.

`lawsynth-profile::profile_missingness` is available for nullable source data,
and `profile_f64_missingness` treats non-finite floating values as missing.
Those functions are pre-ingestion diagnostics; profiling an already valid
`Dataset` reports zero missing values by construction.

Preprocessing includes explicit imputation utilities, but source missingness
must be handled before the final dataset passed to discovery. Leading and
trailing gaps are not silently invented by the data contract.
