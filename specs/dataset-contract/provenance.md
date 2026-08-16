# Provenance

The data crate records no source URI, author, ingestion timestamp, license, or
lineage graph. Its durable data provenance primitive is the content fingerprint
described in `fingerprints.md`.

`lawsynth-profile::DatasetProfile` captures that fingerprint, sample count,
time summary, per-column population statistics, quality flags, and
missingness diagnostics. A `PreprocessPipeline` returns ordered
`AppliedTransform` reports, each carrying transform-specific input/output
fingerprints where applicable.

Callers that need source-file or laboratory provenance must store it outside
the current `Dataset` type and bind it to the returned fingerprint. The runtime
does not serialize a general provenance manifest.
