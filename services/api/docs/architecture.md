# Architecture

`lawsynth-api` is a thin, dependency-free transport layer. It does not
re-implement projects, datasets, worlds, runs, simulations, artifacts, events,
or idempotency; those live in the `lawsynth-server` Python package and are
reached through one call: `lawsynth_server.Application.dispatch(request)`.

## Layers

- **WSGI adapter (`app.py`).** `WsgiApplication` parses the WSGI `environ` into a
  plain request dict (method, path, query, headers, JSON body), enforces
  transport-level limits, dispatches to the domain, then serializes the response
  with safe headers. It owns no business rules — only translation and admission.
- **Lifespan (`lifespan.py`).** `ApiLifespan` constructs and owns the single
  domain `Application` (and its SQLite database + object storage) for the process
  and closes it exactly once. Because the domain repositories are process-local,
  one process holds one catalog.
- **Event boundary (`events.py`).** `EventBus` is an in-process, tenant-scoped,
  bounded ring buffer. It defines the SSE delivery semantics the Rust value
  contract (`lawsynth-api-types`) deliberately leaves to the service: per-scope
  strictly increasing sequences, bounded retention, cursor resume via
  `Last-Event-ID`.
- **Settings (`settings.py`).** `ApiSettings` is the only place environment
  variables are read. The domain core never reads the environment, so a request
  handler cannot silently change tenant or storage configuration after startup.

## Request flow

`environ` -> `_request()` (validate method/path/query/headers/body, negotiate
`X-Api-Version`) -> either the SSE branch, the `501` worker marker, or
`Application.dispatch()`. On a successful run/artifact mutation the adapter emits
a matching streamed event from the domain's own outcome (`_emit_lifecycle`), so
no run/artifact state is duplicated; idempotent replays are skipped to avoid
double-emitting.

## Boundaries

The adapter is synchronous WSGI: there is no server push, no background worker,
and no distributed coordination. Deployment concerns (TLS, load balancing,
multiple replicas) belong to the WSGI server and the gateway in front of it, not
to this process.
