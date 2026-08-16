# Dataset contract

`lawsynth-data` accepts an in-memory, dense numeric time series: one strictly
increasing `TimeAxis` and one or more aligned `NumericColumn`s. This contract
is the boundary consumed by profiling, preprocessing, differentiation, feature
evaluation, and discovery.

The public construction path is `Dataset::new(TimeAxis, columns)`. A dataset is
immutable after construction; transforms return a new dataset plus their
reports. Columns are retained in identifier-sorted order, so schema traversal,
batching, and fingerprints do not depend on caller insertion order.

This is not a nullable table abstraction and it is not an Arrow interchange
layer. Nulls, NaNs, infinities, duplicate identifiers, empty column sets, and
misaligned rows are rejected at ingestion. The package also has a real, bounded
Parquet reader: it supports flat required `INT32`, `INT64`, `FLOAT`, and
`DOUBLE` columns in uncompressed `PLAIN` `DATA_PAGE`s. It rejects compressed,
dictionary/RLE, nullable, repeated, nested, page-v2, and nonnumeric data; it
is not a general-purpose Parquet codec.
