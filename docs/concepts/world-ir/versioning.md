# Versioning

The initial `.lsworld` bundle writes a continuous World IR to a deterministic, uncompressed ZIP archive. It contains `manifest.json`, `world/world.bin`, and SHA-256 payload checksums. Reader validation rejects compressed, multi-disk, malformed-path, checksum-invalid, or unsupported-manifest archives. `migration_path` reports supported format transitions; the initial release has no implicit upgrader.

Pin the LawSynth release and preserve the bundle checksum when reproducing a result. Canonical archive order and fixed ZIP metadata make byte-for-byte comparison meaningful for equivalent current worlds.

Future manifest versions require an explicit reader and migration policy. Do not rely on a newer reader accepting unrecognized archives.
