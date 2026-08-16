# `lawsynth inspect`

```
lawsynth inspect model.lsworld
```

Inspects exactly one validated world bundle and emits counts for states, variables, and parameters. It is deliberately a summary command: it does not print equations, checksums, provenance, units, or arbitrary archive contents. Use the Rust or Python bundle APIs when an application needs to load the executable world.
