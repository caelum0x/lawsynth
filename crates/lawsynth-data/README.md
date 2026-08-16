# lawsynth-data

Validated numerical time-series inputs for LawSynth discovery. A `Dataset`
couples a strictly increasing `TimeAxis` with equal-length named numeric
columns, schema metadata, windowing, deterministic fingerprints, and a bounded
native Parquet reader.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

let data = Dataset::new(
    TimeAxis::new(vec![0.0, 1.0, 2.0])?,
    vec![NumericColumn::new(Identifier::new("x")?, vec![1.0, 2.0, 4.0])],
)?;
assert_eq!(data.len(), 3);
# Ok::<(), lawsynth_data::DataError>(())
```

`read_parquet_numeric` decodes a deliberately narrow, explicit subset:
uncompressed PLAIN numeric leaf columns. It rejects unsupported codecs,
dictionary pages, nesting, and nullable/repeated encodings instead of silently
misreading them. Use a full external Parquet engine for broader formats.
