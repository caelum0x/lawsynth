# Environment capture

The repository exposes `CURRENT_ENGINE_VERSION` through `EngineConfig`; it does
not capture operating system, CPU, compiler, linker, dependency graph,
locale, floating-point mode, thread count, or environment variables at run
time. Lockfiles constrain the checked-out build when they are honored, but are
not embedded in a bundle.

For a reproducible study, store the source revision, lockfile hashes, build
command, target triple, compiler version, operating-system image, and relevant
execution settings alongside the input and output. Treat missing environment
metadata as an incomplete reproduction record.
