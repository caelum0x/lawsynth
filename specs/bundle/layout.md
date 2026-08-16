# Archive and world layout

The archive has exactly these logical entries, ordered lexically by the writer:

1. `manifest.json`
2. `provenance/checksums.sha256`
3. `world/world.bin`

The ZIP writer uses version 2.0 local and central headers, method 0 (stored), zero DOS date/time, CRC-32 for each entry, no extras, no archive comment, and a single disk. Entry paths are UTF-8 byte strings and must be relative slash-separated paths without empty components, `.`, `..`, backslashes, or duplicates. The reader requires the central directory to end immediately before the 22-byte end record, and rejects compression, ZIP64/comments, multi-disk archives, inconsistent sizes, malformed offsets, path violations, duplicate entries, and CRC mismatch.

`world/world.bin` starts with `LSW1` for a continuous World or `LSD1` for a DiscreteWorld. It then contains: `u32le variable_count`, each variable; `u32le parameter_count`, each parameter; `u32le law_count`, each law. Counts are not semantic limits beyond host `usize` conversion, but decoding is bounds checked and world construction validates duplicates and completeness.

A string is `u16le byte_length` plus UTF-8 bytes, with an encoder maximum of 65,535 bytes. A variable is `string id`, one role byte (`0 State`, `1 Control`, `2 Exogenous`, `3 Observed`, `4 Latent`, `5 Derived`), then optional unit (`0` absent or `1` plus string). A parameter is `string id`, little-endian finite `f64`, then the same optional unit. A law is `string target` followed by the preorder expression encoding specified in the expression serialization document. The binary document must end exactly after the final law.
