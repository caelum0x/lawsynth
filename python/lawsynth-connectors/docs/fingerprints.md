# Fingerprints

Reproducible ingestion depends on a stable content identity that does not vary
with dict ordering, float formatting, or Python object identity.
`lawsynth_connectors.fingerprints` provides that identity.

## DatasetFingerprint

```python
DatasetFingerprint(algorithm="sha256", digest="<64 hex>", row_count=..., byte_count=...)
```

Only lowercase SHA-256 digests are accepted; construction validates the algorithm
and digest shape. Every `DataBatch` carries the fingerprint of the exact records
it holds, so two runs over the same source produce byte-identical identities.

## Canonicalization

`canonical_value` maps common scientific values to deterministic JSON data before
hashing:

- primitives (`None`, `str`, `bool`, `int`) pass through;
- non-finite floats become `{"$float": "inf"|"-inf"|"nan"}` (finite floats pass
  through);
- `Decimal`, `datetime` (normalized to UTC), `date`, `bytes` (hex), and `Path`
  (POSIX) get tagged wrappers;
- mappings are emitted with keys sorted as strings; lists and tuples keep order;
  sets and frozensets are sorted by their canonical JSON;
- dataclasses are expanded via `asdict`; objects exposing `.item()` (e.g. NumPy
  scalars) are unwrapped; anything else falls back to a typed `{"$repr", "$type"}`
  form so hashing never crashes.

## Record and file digests

`fingerprint_records(records)` hashes each record as compact, sorted-key,
UTF-8 JSON, length-prefixing every row so record boundaries cannot collide, and
returns the digest with `row_count` and `byte_count`. `fingerprint_file(path)`
streams a file in bounded chunks and returns its SHA-256 with `byte_count`.

These digests feed batch provenance, snapshot identity, and the `max_bytes`
accounting in `BaseConnector`, giving every ingested batch a verifiable, order-
independent fingerprint.
