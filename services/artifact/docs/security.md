# Security boundary

This service trusts only its local caller. It is a deliberate, narrow boundary,
not a full multi-tenant platform.

## What the service does

- Content-addresses every object by SHA-256 and re-verifies the digest on read,
  detecting accidental corruption and counting mismatches in telemetry.
- Bounds request headers, bodies, object size, multipart parts, and total stored
  bytes against the compiled-in limits.
- Redacts by construction: no object bytes, retention values, or secrets are
  written to stderr or returned in error bodies.
- Provides an optional HMAC-SHA-256 `BundleAuthenticator` (constant-time verify)
  for bytes a caller already controls.

## What the service does not do

- No authentication or remote principals. `LocalOnlyAuthorizer` accepts only the
  `local` principal; anything else is rejected. Identity, authorization, and
  tenant isolation belong to a separately built network adapter.
- No transport encryption. `serve` speaks plain HTTP/1.1; terminate TLS and
  authenticate upstream (reverse proxy, mesh) before exposing it beyond
  localhost.
- No sandbox or OS-level quota. Limits gate admission; they do not contain
  arbitrary code or enforce disk quotas at the kernel level.
- The HMAC helper is not a signed-bundle format and does not defend against a
  malicious storage actor with write access to `root`. Use an authenticated
  storage layer where integrity against attackers is required.

## Deployment guidance

Run as a dedicated non-root user that owns `root`, bind to `127.0.0.1` unless a
front door provides authentication and TLS, and keep the HMAC secret in the
embedding process's secret manager — never in `.env.example` or `config/`.
