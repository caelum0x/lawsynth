# lawsynth-store

`lawsynth-store` provides deterministic `MemoryStore` and filesystem-backed `LocalStore` implementations of the synchronous `ObjectStore` contract. Object keys reject absolute paths, traversal, and platform separators; local writes use a same-directory temporary file followed by rename.

It also provides bounded LRU caching, strict multipart assembly, and reachability-based garbage collection. Checksums use FNV-1a for accidental-corruption detection only, not authenticity.

`S3Store` validates S3-compatible endpoint metadata and constructs object URLs, but deliberately does not make network calls: the crate has no HTTP, TLS, credentials, or request-signing dependency. An application must supply that transport boundary rather than treating a configuration object as remote storage.
