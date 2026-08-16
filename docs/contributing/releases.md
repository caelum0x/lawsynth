# Release discipline

The workspace version is currently `0.1.0`. A release must preserve the
canonical World/bundle contract or carry an explicit format-version migration
and compatibility fixtures. Update `CHANGELOG.md`, package metadata, and
release notes only after the corresponding behavior and tests exist.

Before tagging, run the full Rust and Python verification matrix, relevant
conformance/scientific tests, and TypeScript package checks for changed
packages. Build the Python extension with maturin from a clean environment and
verify the artifact imports with its target interpreter.

The repository's offline Cargo setting is intentional: release automation must
use prepopulated locked dependencies or an approved dependency-fetch stage.
Never publish an artifact that was built from an unlocked dependency graph.
