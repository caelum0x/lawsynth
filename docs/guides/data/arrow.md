# Apache Arrow boundary

The current CLI does not decode Arrow IPC or Flight streams. Treat Arrow as an upstream, typed interchange format and materialize the exact numeric columns that will be supplied to LawSynth. Preserve the Arrow schema and the query that produced the extraction as provenance.

Before conversion, select one numeric time field, sort it ascending, reject nulls and non-finite values, and select only numeric observed fields. Convert the result to a simple CSV for the CLI or to Python sequences for `Dataset.from_columns`. Do not rely on implicit Arrow casts: an integer time column may be represented exactly as floats, but timestamps require an explicit choice of unit and origin.

Arrow integration is not an installed LawSynth runtime feature. Code that needs zero-copy Arrow ingestion should own that adapter, validate its output against the same time/column invariants, and test the adapter independently.
