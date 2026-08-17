# Security boundary

`lawsynth-api` is the authenticated HTTP boundary in front of the local domain
core. It authenticates callers, isolates tenants, bounds every request, and
serializes responses defensively. It is not a multi-tenant identity provider.

## What the process does

- **Bearer authentication.** Tokens are compared in constant time
  (`hmac.compare_digest`) against the grant map in `LAWSYNTH_API_TOKENS_JSON`.
  Each token resolves to one `organization_id` and a scope set.
- **Tenant isolation.** Every domain operation is scoped to the principal's
  `organization_id`; a caller for tenant A can never read or mutate tenant B's
  projects, runs, artifacts, or events. Event streams are partitioned by the same
  scope.
- **Scope enforcement.** Reads require `read`, mutations require `write`, `admin`
  implies both. Writes additionally require an `Idempotency-Key`.
- **Admission limits.** Method allowlist, absolute/normalized path checks
  (rejecting `.`/`..`/NUL/backslash), header line-break rejection, a bounded
  query (max 32 params, no duplicates), `application/json`-only bodies, and a
  `max_request_bytes` ceiling.
- **Defensive responses.** Fixed safe headers on every response
  (`Cache-Control: no-store`, `X-Content-Type-Options: nosniff`,
  `X-Api-Version`), a validated `Content-Length`, and JSON serialized with
  `allow_nan=False`. Error bodies never include stack traces or secrets.

## What the process does not do

- **No transport encryption.** The stdlib server is loopback-only; a deployment
  must terminate TLS at the gateway or reverse proxy. Do not expose the WSGI
  socket directly to the internet.
- **No external identity.** OAuth/OIDC verification is deliberately out of scope
  (`lawsynth_server/auth.py`); it belongs to a deployment-specific identity
  adapter. This process only verifies pre-provisioned local bearer tokens.
- **No distributed guarantees.** Repositories are process-local; the API does not
  claim distributed metadata, shared idempotency, or remote worker execution.
  `/v1/worker/*` is an honest `501`, and a missing native runtime is an honest
  `503 native_unavailable`.

## Deployment guidance

Run as the dedicated non-root user that owns the database and object root, put
the audited gateway (and TLS) in front, and keep `LAWSYNTH_API_TOKENS_JSON` in a
secret manager — never in `config/` or `.env.example`. Rotate any token that may
have been exposed by replacing the grant map and restarting.
