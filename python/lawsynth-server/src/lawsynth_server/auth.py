"""Bearer-token authentication for deployments that provision local tokens.

External OAuth/OIDC is deliberately outside this package: verification must be
performed by a deployment-specific identity adapter, not guessed locally.
"""

from __future__ import annotations

import hmac
from dataclasses import dataclass
from typing import Mapping

from .errors import AuthenticationError, AuthorizationError


@dataclass(frozen=True, slots=True)
class Principal:
    subject: str
    organization_id: str
    scopes: frozenset[str]


class TokenAuthenticator:
    def __init__(self, tokens: Mapping[str, tuple[str, frozenset[str]]]) -> None:
        self._tokens = dict(tokens)

    def authenticate(self, header: str | None) -> Principal:
        if not header or not header.startswith("Bearer "):
            raise AuthenticationError("Bearer authentication is required")
        token = header[7:]
        for candidate, (organization_id, scopes) in self._tokens.items():
            if hmac.compare_digest(token, candidate):
                return Principal(subject=f"token:{token[:8]}", organization_id=organization_id, scopes=scopes)
        raise AuthenticationError("invalid bearer token")


def require_scope(principal: Principal, scope: str) -> None:
    if scope not in principal.scopes and "admin" not in principal.scopes:
        raise AuthorizationError("missing required scope", details={"scope": scope})
