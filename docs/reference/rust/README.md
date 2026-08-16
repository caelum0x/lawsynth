# Rust crate reference

LawSynth is organized as small crates with explicit contracts: `lawsynth-core` provides foundational validation and deterministic metadata; `data`, `expr`, `world`, `sim`, `discovery`, and `bundle` implement the executable workflow. The CLI and PyO3 extension bind that workflow for applications.

Each crate exposes a deliberately bounded offline API. Features described as unsupported in these pages are rejected or absent, not emulated by placeholder behavior.
