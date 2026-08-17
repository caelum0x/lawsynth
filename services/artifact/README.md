# LawSynth artifact service

`lawsynth-artifact-service` is a local-first, content-addressed store for LawSynth
artifacts. Objects are identified by their SHA-256 digest, stored durably through
`lawsynth-store`'s `LocalStore`, and served either as an in-process library or over
a small dependency-free HTTP/1.1 transport. Every write records its checksum,
size, content type, and optional retention; every read re-verifies integrity.

The crate is a library first (`ArtifactService`) with an optional network layer
(`ArtifactServer`). Routing is a pure function of `(service, now, request)`, so the
whole API is testable without opening a socket. No async runtime, HTTP framework,
or database engine is linked.

## Boundaries

- Trusts only the local caller. The `LocalOnlyAuthorizer` accepts a single
  `local` principal; authentication and remote principals belong to a separate,
  explicitly built adapter. See `docs/security.md`.
- Serves HTTP/1.1 only. gRPC, queues, streaming, and uploads to remote object
  stores are reported as `NetworkSurface::NotImplemented` rather than stubbed.
- Enforces compiled-in resource ceilings (`config/limits.yaml`). Limits gate
  admission; they are not OS-level quotas.
- The HMAC `BundleAuthenticator` protects bytes a caller already controls; it is
  not a signed-bundle format or a defense against a malicious storage actor.

## CLI

```
lawsynth-artifact serve <root> <addr>                 # HTTP/1.1 over the local core
lawsynth-artifact health <root>                       # print catalog + capacity summary
lawsynth-artifact gc <root> <unix-seconds> [--dry-run] # sweep expired artifacts
```

Example:

```sh
lawsynth-artifact serve ./.lawsynth/artifacts 127.0.0.1:8080
lawsynth-artifact health ./.lawsynth/artifacts
lawsynth-artifact gc ./.lawsynth/artifacts 1723900000 --dry-run
```

`serve` prints one startup line to stderr and then blocks, handling one request
per connection on a thread-per-connection model.

## HTTP surface

See `docs/api.md`. In brief: `POST /artifacts`, `GET /artifacts/{id}`,
`DELETE /artifacts/{id}`, `GET /artifacts/{id}/metadata`, the `POST /uploads` …
`PUT /uploads/{id}/parts/{n}` … `POST /uploads/{id}/complete` multipart flow,
`POST /gc`, and `GET /health`.

## Configuration

Resource limits and per-environment profiles live under `config/`
(`limits.yaml`, `development.yaml`, `test.yaml`, `staging.yaml`, `production.yaml`,
`logging.yaml`). They document the fields of `ArtifactConfig`
(`src/config.rs`); the current CLI applies the built-in defaults reproduced in
`limits.yaml`. No environment variables are read — see `.env.example`.

## Build and test

```sh
cargo build --release -p lawsynth-artifact-service
cargo test -p lawsynth-artifact-service
```

Further reading: `docs/architecture.md`, `docs/api.md`, `docs/operations.md`,
`docs/failures.md`, `docs/security.md`.
