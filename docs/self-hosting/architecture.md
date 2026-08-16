# Local architecture

`create_app(Settings(...))` constructs an in-process `Application`. Middleware validates request shape and translates domain errors; bearer tokens resolve to organization-scoped principals; repositories persist projects, datasets, worlds, and runs through SQLite; artifacts use content-addressed files; idempotency records replayable write outcomes; and events are appended per organization.

The only implemented persistence choices are SQLite (`:memory:` or `sqlite:///path`) and a local directory. The object store writes a temporary file, flushes it, fsyncs it, then atomically replaces the content-addressed target. This gives useful local crash behavior on a filesystem that honors those operations; it is not a distributed-storage protocol.

An HTTP adapter may translate requests into `Application.dispatch()`, but it must own request-size limits, TLS, trusted-proxy handling, connection lifecycle, security headers, and process supervision. None are supplied by this repository.
