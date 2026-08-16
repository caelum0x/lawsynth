# Data identity

No dataset hash is automatically recorded by the discovery, simulation, or
bundle APIs. `stable_hash` is a 64-bit FNV-1a utility for deterministic local
keys and seed derivation; it is not collision-resistant and must not identify
scientific input or prove integrity.

Use SHA-256 (or a stronger approved external policy) over the exact input byte
stream, record the encoding and canonicalization rules, and retain a secure
copy when policy permits. If parsing transforms data, record both the source
digest and the parser/configuration used to obtain the numeric dataset.
