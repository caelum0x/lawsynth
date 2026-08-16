## LawSynth gateway

This package is the transport-independent WSGI admission layer in front of the
local `lawsynth-api` application. It canonicalizes headers, rejects oversized
or malformed bodies, enforces an exact-origin CORS policy, creates request IDs,
applies a bounded sliding-window limit, and exposes `/healthz` and `/readyz`.

Compose it in the process hosting the API:

```python
from lawsynth_api import create_wsgi_app
from lawsynth_gateway import create_gateway

application = create_gateway(create_wsgi_app())
```

Only an in-process WSGI backend is implemented. Remote upstream proxying, TLS
termination, and retry behavior are explicitly unavailable; they must be
provided by an audited edge proxy rather than simulated by this process.
