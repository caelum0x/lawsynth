# Artifact operations

Bundle operations are synchronous function calls. `write_world` and
`write_discrete_world` create a deterministic stored-ZIP archive; `read_world`
and `read_discrete_world` validate it before returning a validated World IR.
They do not emit an artifact-created, artifact-read, upload, download, or
deletion event.

Consumers needing auditable artifact lifecycle records must create them outside
the engine and bind them to the exact archive bytes or SHA-256 checksum. The
checksum file inside a bundle detects corruption but cannot authenticate an
attacker who can replace both bytes and checksum records.
