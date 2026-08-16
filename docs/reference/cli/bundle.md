# CLI bundle handling

`lawsynth inspect PATH.lsworld` validates a bundle and reports either `continuous world: S states, V variables, P parameters` or `discrete world: ...`. It first attempts continuous decoding and then discrete decoding.

The command does not repair, migrate, sign, merge, or extract archives. Bundle reads validate the stored-ZIP structure, required paths, CRC-32 values, SHA-256 provenance entries, the exact v0.1 manifest, and the world binary before a summary is printed.
