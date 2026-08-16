# Artifact preservation

The implemented `.lsworld` writer uses lexically ordered entries, stored ZIP
encoding, fixed timestamps/metadata, and deterministic binary World encoding.
It writes `manifest.json`, `world/world.bin`, and
`provenance/checksums.sha256`. Reading validates the fixed manifest, ZIP
structure, CRCs, SHA-256 entry checksums, and World invariants.

Preserve the full archive, not only a printed equation. The internal checksums
provide corruption detection, not origin authentication. The format has no
embedded dataset, run configuration, seed, citation, external attachment, or
signature entry in version 0.1.
