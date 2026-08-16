# Reproducibility contract

LawSynth makes deterministic inputs and artifacts possible; it does not
automatically make every invocation reproducible. The implemented foundation
is `EngineConfig` (engine version, `Seed`, and `ResourceLimits`), deterministic
World construction, and deterministic stored-ZIP bundle writing. A complete
reproduction record must be assembled by the caller.

The sub-specifications distinguish information the code currently carries from
metadata that an experiment owner must preserve externally. They do not imply a
database, provenance service, environment capture tool, citation generator, or
one-command reproducibility verifier.
