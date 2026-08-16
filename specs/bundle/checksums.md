# Checksums

`provenance/checksums.sha256` is UTF-8 text with one line for each payload entry excluding itself:

```text
<64 lowercase SHA-256 hex characters><two ASCII spaces><entry path>\n
```

The writer emits lines in lexical entry order for `manifest.json` and `world/world.bin`. The reader requires a line for every non-checksum archive entry and no extras, rejects malformed or duplicate checksum paths, computes SHA-256 over exact entry bytes, and fails on any mismatch. The checksum file is protected by the ZIP CRC but is not self-hashed.

SHA-256 verifies accidental or storage corruption only when the archive is otherwise trusted; an attacker can replace both content and checksums. Use an external authenticated transport or a separately managed HMAC tag where threat resistance is required.
