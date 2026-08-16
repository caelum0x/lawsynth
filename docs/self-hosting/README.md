# Self-hosting the local server core

`lawsynth-server` is a dependency-light Python domain core for local use and integration tests. It uses Python's standard-library SQLite adapter and atomic filesystem object storage. `Application.dispatch()` receives an in-process request dictionary; it is not an HTTP listener and `lawsynth serve` intentionally returns an unsupported-operation error.

Install the package in a Python 3.11+ environment, set `PYTHONPATH=src` while developing, and run `python -m pytest -q tests` from `python/lawsynth-server`. Configure a SQLite URL, an object root on a local filesystem, and explicit local bearer tokens. See [architecture](architecture.md) and [authentication](authentication.md) before exposing any adapter beyond a trusted machine.

Production HTTP/ASGI hosting, OAuth/OIDC verification, PostgreSQL, object-store signing, queues, worker execution, and distributed event delivery are deliberately unavailable here. Supply reviewed deployment-specific adapters for those concerns; do not infer support from the presence of domain routes.
