"""SSO/OIDC admission seam for the gateway.

The gateway is the public entry; the API is never internet-exposed.  The
hosted-platform spec (``specs/hosted-platform/README.md`` -> "Authentication
(SSO)") requires the gateway to accept a documented SSO flow, *exchange it for a
tenant-scoped principal*, and pass an authenticated principal to the API, while
rejecting unauthenticated or cross-tenant requests before they reach the backend.

Honest boundary -- what is REAL vs what is a SEAM
    This module implements a **self-contained, HMAC-signed identity assertion**
    that is validated offline and exchanged for a machine principal (a backend
    bearer token the API already recognises).  The following are REAL, exercised
    enforcement:

      * cryptographic verification of the assertion (constant-time HMAC-SHA256
        over the exact signed bytes), so a tampered ``org``/``sub``/``scopes`` is
        rejected;
      * the token *exchange* and tenant binding: a valid assertion for tenant T
        yields T's backend credential and no other's;
      * rejection of unauthenticated (missing/invalid assertion -> 401) and
        cross-tenant / unprovisioned-tenant (-> 403) requests at the edge.

    What is a SEAM (deliberately not implemented here): the upstream OIDC/SAML
    protocol itself -- discovery, the authorization-code exchange, JWKS rotation.
    A production IdP plugs in by terminating that flow at the edge and minting the
    signed assertion this module verifies, or by replacing :meth:`SsoAuthenticator.
    _verify` with real JWKS signature verification.  The assertion format and the
    tenant-binding exchange stay identical, so nothing downstream changes.

Assertion wire format (header ``X-Lawsynth-Assertion``)
    ``<payload_b64url>.<hex_hmac_sha256(secret, payload_b64url)>`` where the
    payload is compact JSON ``{"sub": ..., "org": ..., "scopes": [...]}``.  The
    signature covers the base64url text exactly as transmitted, so verification
    never depends on re-canonicalising JSON.
"""

from __future__ import annotations

import base64
import hashlib
import hmac
import json
from dataclasses import dataclass
from typing import Mapping, Sequence

ASSERTION_HEADER = "X-Lawsynth-Assertion"
TENANT_HEADER = "X-Lawsynth-Tenant"


@dataclass(frozen=True, slots=True)
class Principal:
    """A tenant-scoped principal resolved from a verified identity assertion."""

    subject: str
    organization_id: str
    scopes: tuple[str, ...]


class SsoError(Exception):
    """A rejection at the SSO seam, carrying the edge status/code to surface."""

    def __init__(self, status: int, code: str, message: str) -> None:
        super().__init__(message)
        self.status = status
        self.code = code
        self.message = message


def _b64url_encode(raw: bytes) -> str:
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def _b64url_decode(text: str) -> bytes:
    padding = "=" * (-len(text) % 4)
    return base64.urlsafe_b64decode(text + padding)


class SsoAuthenticator:
    """Verifies signed identity assertions and exchanges them for principals.

    Constructed with an HMAC ``secret`` and a ``tenants`` trust map from
    ``organization_id`` to the backend bearer token that identifies that tenant to
    the API.  The map is the provisioning boundary: an assertion for a tenant not
    in the map cannot be exchanged, which is how the seam refuses cross-tenant and
    unprovisioned access.
    """

    def __init__(self, secret: str, tenants: Mapping[str, str]) -> None:
        if not isinstance(secret, str) or len(secret) < 16:
            raise ValueError("SSO secret must be at least 16 characters")
        if not tenants or any(not isinstance(k, str) or not isinstance(v, str) or not k or not v for k, v in tenants.items()):
            raise ValueError("SSO tenants must map organization ids to backend tokens")
        self._secret = secret.encode("utf-8")
        self._tenants = dict(tenants)

    def issue(self, subject: str, organization_id: str, scopes: Sequence[str]) -> str:
        """Mint an assertion for ``organization_id`` (the shape a real IdP emits).

        Provided so tests and an upstream IdP adapter produce assertions in the
        exact format :meth:`exchange` verifies; the gateway itself only verifies.
        """

        payload = json.dumps(
            {"sub": subject, "org": organization_id, "scopes": list(scopes)},
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
        encoded = _b64url_encode(payload)
        signature = self._sign(encoded)
        return f"{encoded}.{signature}"

    def exchange(self, assertion: str | None) -> tuple[Principal, str]:
        """Verify ``assertion`` and return ``(principal, backend_token)``.

        Raises :class:`SsoError` for a missing/invalid assertion (401) or an
        assertion bound to a tenant this gateway cannot provision (403).
        """

        if not assertion:
            raise SsoError(401, "authentication_required", "an identity assertion is required")
        claims = self._verify(assertion)
        organization_id = claims.get("org")
        subject = claims.get("sub")
        if not isinstance(organization_id, str) or not organization_id or not isinstance(subject, str) or not subject:
            raise SsoError(401, "invalid_assertion", "assertion is missing a subject or tenant")
        token = self._tenants.get(organization_id)
        if token is None:
            raise SsoError(403, "tenant_forbidden", f"no principal is provisioned for tenant {organization_id!r}")
        raw_scopes = claims.get("scopes")
        scopes = tuple(scope for scope in raw_scopes if isinstance(scope, str)) if isinstance(raw_scopes, list) else ()
        return Principal(subject=subject, organization_id=organization_id, scopes=scopes), token

    def _verify(self, assertion: str) -> Mapping[str, object]:
        encoded, separator, signature = assertion.partition(".")
        if not separator or not encoded or not signature:
            raise SsoError(401, "invalid_assertion", "assertion is malformed")
        expected = self._sign(encoded)
        if not hmac.compare_digest(signature, expected):
            raise SsoError(401, "invalid_assertion", "assertion signature is invalid")
        try:
            claims = json.loads(_b64url_decode(encoded))
        except (ValueError, json.JSONDecodeError) as error:
            raise SsoError(401, "invalid_assertion", "assertion payload is not valid JSON") from error
        if not isinstance(claims, dict):
            raise SsoError(401, "invalid_assertion", "assertion payload must be an object")
        return claims

    def _sign(self, encoded: str) -> str:
        return hmac.new(self._secret, encoded.encode("ascii"), hashlib.sha256).hexdigest()
