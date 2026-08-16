# Authentication and authorization

The local core authenticates an `Authorization: Bearer <token>` header against `Settings.tokens`. A configured token maps to one organization identifier and a frozen scope set. Reads require `read`; mutations require `write`; `admin` satisfies either. Token comparison uses constant-time comparison, but token provisioning and transport security remain deployment responsibilities.

Use randomly generated tokens, store them outside the repository, rotate them by replacing the configured mapping, and terminate TLS before any untrusted network. Treat the token prefix exposed in an in-memory principal as diagnostic data only; never log complete authorization headers.

OAuth/OIDC, user accounts, password storage, API-key hashing at rest, SSO, SCIM, rate limiting, and audit export are not implemented. Do not expose the local-token scheme directly to the public internet.
