# LawSynth specifications

This directory holds LawSynth's **boundary specifications**: precise contracts for
the validated types and formats the engine implements today. Every document is
written to be honest about the current in-process scientific library and CLI. A
spec describes semantics a caller can rely on when it receives a value from the
process that produced it; it does not invent network protocols, brokers, or
platform services that are not compiled into this distribution.

Where a capability is planned but not implemented, the relevant README says so
explicitly (see `service-api`, `event-protocol`, and `security-model`).
Implementers MAY build against these contracts, but MUST publish their own
concrete transport schema, authentication, and security policy before claiming
interoperability.

The specification set is versioned as a whole in [`VERSION`](./VERSION), aligned
with the workspace crates (0.1.0). Each directory keeps a `changelog.md`.

## Contents

| Directory | Boundary |
| --- | --- |
| `world-ir` | Validated in-memory World / DiscreteWorld model consumed by the simulator and bundle codec (`lawsynth-world`). |
| `expression-language` | Grammar, typing, canonicalization, and differentiation of scalar expressions (`lawsynth-expr`). |
| `dataset-contract` | Dense numeric time-series ingestion boundary: schema, units, missingness, provenance, fingerprints (`lawsynth-data`). |
| `simulation-contract` | Solver, time-grid, initial-state, noise, and trajectory semantics for the deterministic simulator (`lawsynth-sim`). |
| `discovery-run` | Run spec, stage/candidate/score contracts, determinism, and resources for a discovery run. |
| `causal-contract` | Causal graph, identification, intervention, and sensitivity assumptions. |
| `regime-contract` | Regime states, segments, transitions, guards, and regime-selected laws. |
| `uncertainty-contract` | Parameter, structural, and trajectory uncertainty: samples, intervals, propagation, summaries. |
| `bundle` | The `.lsworld` deterministic ZIP bundle format, layout, checksums, and limits (`lawsynth-bundle`). |
| `reproducibility` | Seed plans, data/plan hashes, environment and hardware class, citation of run artifacts. |
| `event-protocol` | Semantics of the local event-shaped values (no network protocol is defined). |
| `plugin-protocol` | Plugin manifest, capabilities, permissions, lifecycle, and transport contract. |
| `security-model` | Current controls and the boundaries deployers must supply themselves. |
| `service-api` | Forward-compatible service contracts over `lawsynth-api-types` (a boundary spec, not a running service). |

## Reading a spec

Start at a directory's `README.md` for its scope and non-goals, then read the
per-topic files for normative detail. RFC 2119 keywords (MUST, SHOULD, MAY) carry
their usual meaning. A Rust struct constructed by other means than the documented
validating constructor is **not** a valid artifact under these specs.
