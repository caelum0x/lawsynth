# LawSynth Server

`lawsynth-server` is the dependency-light domain core for local LawSynth server
mode. It provides organization-scoped repositories, bearer-token authorization,
idempotent writes, append-only events, SQLite transaction support, and atomic
filesystem content-addressed storage. `Application.dispatch()` is an
in-process transport boundary; deployments may adapt it to ASGI/WSGI after
their edge authentication and network policy are selected.

The package intentionally does not pretend to implement OAuth, S3 signing,
worker scheduling, or a distributed event bus. Those require deployment-owned
adapters and credentials. The implemented local contracts remain fully usable
for deterministic development and integration testing.

```python
from lawsynth_server import Settings, create_app

app = create_app(Settings(tokens={"dev-token": ("acme", frozenset({"read", "write"}))}))
response = app.dispatch({
    "method": "POST", "path": "/projects",
    "headers": {"Authorization": "Bearer dev-token", "Idempotency-Key": "create-climate"},
    "body": {"name": "climate"},
})
assert response["status"] == 201
```

Run the service core tests with:

```sh
PYTHONPATH=src python -m pytest -q tests
```
