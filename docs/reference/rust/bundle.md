# `lawsynth-bundle`

`write_world`/`read_world` persist continuous `World`; `write_discrete_world`/`read_discrete_world` do the same for `DiscreteWorld`. A bundle is deterministic stored ZIP containing `manifest.json`, `world/world.bin`, and `provenance/checksums.sha256`. The manifest is exactly format `lawsynth-world`, version `0.1.0`, encoding `lawsynth-world-binary-v1`.

Readers reject compressed entries, ZIP64, archive comments, multi-disk archives, unsafe paths, missing/duplicate entries, CRC failures, SHA-256 mismatches, unsupported manifests, invalid units/identifiers, nonfinite values, unknown expression tags, and nesting of 128 or more. `BundleSignature` provides HMAC-SHA256 authentication and `verify_signature` uses constant-time comparison; it is shared-key authentication, not a public-key signature format.
