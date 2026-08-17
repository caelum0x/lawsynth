# Architecture

The service is layered so the network transport is an explicit addition over a
self-contained local core, never an ambient capability.

## Core (`ArtifactService`)

`ArtifactService::open(ArtifactConfig)` validates configuration and opens a
`LocalArtifactStorage` over `lawsynth-store`. The service owns:

- a content-addressed object store keyed by SHA-256 digest (`checksum.rs`,
  `object.rs`, `storage.rs`);
- an `ArtifactCatalog` that lists and counts durable metadata records
  (`database.rs`, `metadata.rs`);
- a bounded in-process read cache (`cache.rs`) sized by `cache_capacity_bytes`;
- multipart upload sessions assembled deterministically (`multipart.rs`,
  `upload.rs`);
- retention state and a deterministic garbage-collection sweep (`retention.rs`,
  `gc.rs`);
- lock-free operational counters (`telemetry.rs`).

Time is always supplied by the caller as a Unix-seconds argument, so ingestion,
retrieval, and GC are deterministic and testable without a clock.

## Transport (`ArtifactServer`)

The optional `http` module implements a blocking, thread-per-connection HTTP/1.1
server on `std::net` with no framework. It parses one bounded request, hands it
to `router::route(service, now, request)` — a pure function — and writes the
response. `ArtifactServer::with_system_clock` injects the wall clock for `main`.
`NetworkSurface` is the honest capability declaration: `Http` is real,
`NotImplemented` names transports the crate does not link.

## Storage layout

Objects and their metadata are published atomically by `LocalStore`, so a record
survives a process restart and a reader never observes a partially written
object. The digest is both the identity and the integrity check.
