# Self-hosting the local server core

> **Just want it running over HTTP?** Follow the [quickstart](quickstart.md):
> bring up the API + admission gateway (SQLite metadata, filesystem object
> store), mint a token, and submit your first discovery run with `curl` or
> `lawsynth.Client`. The runnable Compose profile lives in
> [`deploy/quickstart`](../../deploy/quickstart/).

`lawsynth-server` is a dependency-light Python domain core for local use and integration tests. It uses Python's standard-library SQLite adapter and atomic filesystem object storage. `Application.dispatch()` receives an in-process request dictionary; it is not itself an HTTP listener. The HTTP surface is the separate `lawsynth-api` process (a stdlib WSGI adapter that dispatches to this core); the domain's own `lawsynth serve` intentionally returns an unsupported-operation error.

Install the package in a Python 3.11+ environment, set `PYTHONPATH=src` while developing, and run `python -m pytest -q tests` from `python/lawsynth-server`. Configure a SQLite URL, an object root on a local filesystem, and explicit local bearer tokens. See [architecture](architecture.md) and [authentication](authentication.md) before exposing any adapter beyond a trusted machine.

Production HTTP/ASGI hosting, OAuth/OIDC verification, PostgreSQL, object-store signing, queues, worker execution, and distributed event delivery are deliberately unavailable here. Supply reviewed deployment-specific adapters for those concerns; do not infer support from the presence of domain routes.
