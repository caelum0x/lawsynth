# Preparing numerical observations

LawSynth discovery consumes one strictly ordered time axis and one or more aligned, finite numeric columns. The supported command-line ingestion path is rectangular numeric CSV. The Python SDK accepts Python sequences through `Dataset.from_columns`; native discovery receives owned numeric arrays from that object.

Start by deciding which column is time and which observed columns are candidate states. Keep the original measurements immutable, record every cleaning step, and write the derived table separately. Discovery is an inference procedure, not a repair tool: it rejects non-finite values, unequal column lengths, and non-increasing time rather than guessing an interpretation.

```text
time,x,y
0.0,1.00,0.20
0.1,0.98,0.25
0.2,0.94,0.30
```

Use [CSV](csv.md) for the executable CLI workflow. The Arrow, pandas, Polars, and xarray pages describe safe conversion boundaries; none of those formats is silently read by the CLI. The native Parquet reader is deliberately narrower than general Parquet and is documented in [Parquet](parquet.md).
