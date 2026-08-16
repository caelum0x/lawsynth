# Fingerprints

`Dataset::content_fingerprint` returns a typed `DatasetFingerprint`; `fingerprint`
returns its `u64` value. It is a deterministic FNV-1a-derived hash over the
domain tag `lawsynth.dataset.v1`, timestamp IEEE-754 bits, sorted column
identifiers, optional unit bytes, and numeric IEEE-754 bits.

Column names and unit strings are length-delimited before hashing. Timestamps
and values are fixed-width bit sequences. Consequently a change to data,
ordering of timestamps, a column name, or unit metadata changes the input
address used by profiles and discovery checkpoints.

This is reproducibility metadata, not a cryptographic integrity guarantee and
not a collision-resistant content address for hostile inputs.
