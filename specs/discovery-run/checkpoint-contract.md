# Checkpoint contract

`DiscoveryCheckpoint` permits deterministic restart at the library API. A new
checkpoint stores the dataset fingerprint; the executor also records a stable
hash of the debug representation of the discovery configuration. A mismatch in
either value rejects resumption.

After each sparse law, the checkpoint stores the state identifier, canonical
printed expression, and residual sum of squares. On resumption, stored laws
are parsed and reused instead of refit. States and laws are `BTreeSet`/
`BTreeMap` ordered, so serialization is stable.

The on-disk LSCP2 text format contains magic, dataset fingerprint,
configuration fingerprint (or `-`), then `S` state or `L` law tab records.
Residuals are serialized as `f64::to_bits()` decimal integers. The loader also
accepts legacy LSCP1 state-only checkpoints. This format has no encryption,
signature, lockfile, atomic-write protocol, or CLI command-line integration.
