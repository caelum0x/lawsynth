# `lawsynth-data`

`Dataset` combines a strictly increasing `TimeAxis` with aligned finite `NumericColumn`s. It validates schema and produces deterministic fingerprints. Windows and batches operate on validated in-memory data.

`read_parquet_numeric` is an actual decoder for a deliberately narrow Parquet subset: flat required numeric columns encoded with uncompressed PLAIN pages (`INT32`, `INT64`, `FLOAT`, or `DOUBLE`). It parses the Parquet metadata and rejects compression, dictionary encodings, repetition/definition levels, nested schemas, unsupported physical types, and other unimplemented encodings. It must not be represented as a general Parquet reader.
