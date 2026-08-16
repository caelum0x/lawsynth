# Plugins

No plugin registry, dynamic loader, subprocess protocol, WASM host, signature
policy, or plugin permission model is implemented in the active engine crates.
Accordingly, no third-party algorithm is authorized merely by being placed near
the repository or named in configuration.

An application that extends LawSynth must isolate executable extensions using
its own process or runtime boundary, validate its input/output schema, assign
resource limits, authenticate provenance, and make failure handling explicit.
Those controls are deployment work, not current LawSynth behavior.
