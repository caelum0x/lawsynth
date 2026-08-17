# Security boundary

`lawsynth-gateway` is the public entry point. It is the layer that faces
untrusted clients; the `lawsynth-api` application it wraps is not intended to be
internet-exposed on its own. The gateway's job is admission and canonicalization,
not authentication — the API still enforces bearer auth, tenant isolation, and
idempotency behind it.

## Trust boundary

```
untrusted client ──▶ TLS terminator / edge ──▶ [ gateway ─▶ API ] (one process)
```

- The **gateway** admits or rejects the request and cleans it.
- The **API** authenticates the bearer token, scopes the caller to a tenant, and
  performs the operation.
- The API socket must never be published directly; only the gateway (behind TLS)
  should receive external traffic.

## What the gateway enforces

- **Path and method safety.** Absolute, normalized paths only (rejecting
  control characters, `\`, and `.`/`..`); a fixed method allowlist; routing
  limited to `api_prefix`, `/healthz`, `/readyz`.
- **Header hygiene.** Canonical header names, rejection of invalid values and
  duplicates, and stripping of hop-by-hop headers and *client-supplied*
  forwarding headers (`X-Forwarded-*`, `Forwarded`, `X-Real-IP`) so a caller
  cannot spoof its origin or the logged client identity. The gateway sets
  `X-Forwarded-For`/`-Proto` itself.
- **Resource bounds.** Body, header count, and header byte ceilings; rejection of
  chunked/transformed bodies; a bounded, LRU-capped sliding-window rate limit.
- **Exact-origin CORS.** Only origins in the allowlist are permitted; preflight
  is validated, not reflected blindly.
- **Response defense.** Fixed safe headers, an authoritative `X-Request-Id` a
  backend cannot override, recomputed `Content-Length`, and conversion of any
  malformed backend response into `502` rather than leaking it.

## What the gateway does not do

- **No authentication or authorization.** It does not read or validate bearer
  tokens; that is the API's responsibility. It passes `Authorization` through
  untouched and never logs it.
- **No TLS.** It speaks plain HTTP; terminate TLS at an upstream edge/proxy.
- **No remote upstream.** Proxying to a remote API, retries, and circuit-breaking
  are unavailable by design (`RemoteUpstreamUnavailable`); use an audited edge
  proxy if they are required.

## Deployment guidance

Terminate TLS upstream, run the gateway+API as a single non-root process, keep
the API unreachable except through the gateway, and configure `allowed_origins`
to the minimal exact set. Rate-limit and body ceilings should match the API's own
limits so the two boundaries agree.
