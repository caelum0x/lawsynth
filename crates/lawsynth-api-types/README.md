# lawsynth-api-types

Transport-neutral domain values for service adapters. Constructors validate IDs,
digests, lifecycle transitions, schemas, page bounds, and ordered event streams
before work is handed to the engine. This crate intentionally contains neither
an HTTP server nor a serializer; adapters choose their own transport while
preserving these invariants.

Run its checks with `cargo test -p lawsynth-api-types`.
