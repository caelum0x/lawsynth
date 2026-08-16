# lawsynth-bundle

The current bundle implementation writes deterministic, uncompressed ZIP
archives with the .lsworld extension.

Each archive contains:

- manifest.json — fixed format and encoding identifiers;
- world/world.bin — the initial continuous World IR encoding;
- provenance/checksums.sha256 — SHA-256 checksums for every payload.

The reader rejects compressed, multi-disk, malformed-path, checksum-invalid,
or unsupported-manifest archives. Entry order, timestamps, and metadata are
fixed, so equivalent worlds produce identical bytes.

This is the intentionally small v0 bundle subset. The production layout will
add inspectable JSON metadata, CBOR expressions, Arrow or Parquet evidence,
signatures, migrations, and ZIP64 support while retaining deterministic output.
