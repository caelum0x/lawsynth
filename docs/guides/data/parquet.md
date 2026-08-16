# Parquet input

`lawsynth-data` includes a real, defensive reader for a deliberately small Parquet subset: flat required numeric columns encoded as uncompressed PLAIN pages. It validates Parquet metadata and refuses unsupported encodings, compression codecs, dictionary pages, nested/repeated fields, and definition or repetition levels. That refusal is intentional; it prevents a compressed or nested file from being decoded incorrectly.

The command-line `discover` command reads CSV, not Parquet. For a general Parquet warehouse, use a mature Parquet implementation to select, validate, and export the required flat columns to CSV. If an application calls the native subset reader directly, pin test fixtures to its accepted layout and surface unsupported-codec errors to the operator.

Never label the subset reader as a general Parquet codec. A production pipeline requiring Snappy, Zstd, dictionary encoding, nullable columns, or nested schemas needs a full codec dependency at its ingestion boundary.
