# Provenance boundary

World IR itself stores no author, source dataset, run identity, timestamps, cryptographic signature, or model-confidence fields. Such information belongs to an enclosing bundle or application-level record. The dependency graph is inspectable derivation metadata, but it is not provenance and contains no source attribution.

The stable expression fingerprint uses FNV-1a over the expression canonical string and is appropriate only for deterministic ordering or local cache keys. It is explicitly not a cryptographic checksum. Bundle integrity uses SHA-256 checksums; an optional HMAC helper authenticates bytes only when a caller provides and protects a shared secret.
