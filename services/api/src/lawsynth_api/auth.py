"""Bearer-token authentication at the WSGI boundary.

The domain service (``lawsynth_server.auth.TokenAuthenticator``) owns credential
verification; this adapter extracts the ``Authorization`` header from a parsed
request and translates the domain's :class:`AuthenticationError` into a
transport :class:`RequestProblem`.  Keeping the extraction here means the WSGI
layer never re-implements token comparison and the streaming/lifecycle paths
share one authentication entry point.
"""

from __future__ import annotations

from typing import Mapping

from lawsynth_server.auth import Principal, TokenAuthenticator
from lawsynth_server.errors import AuthenticationError, AuthorizationError

from .middleware import RequestProblem


class ApiAuthenticator:
    """Header-aware facade over the domain :class:`TokenAuthenticator`."""

    def __init__(self, authenticator: TokenAuthenticator) -> None:
        self._authenticator = authenticator

    def authenticate(self, headers: Mapping[str, str]) -> Principal:
        """Resolve the caller's principal or raise the domain's own error."""

        return self._authenticator.authenticate(headers.get("Authorization"))

    def authenticate_or_problem(self, headers: Mapping[str, str]) -> Principal:
        """Resolve the principal, mapping a failure to a 401 transport problem."""

        try:
            return self.authenticate(headers)
        except AuthenticationError as error:
            raise RequestProblem(401, error.code, error.message) from error

    def silent(self, headers: Mapping[str, str]) -> Principal | None:
        """Resolve the principal, returning ``None`` when authn/authz fails.

        Used by best-effort side paths (lifecycle event emission) that must
        never turn an authentication failure into a request failure, because the
        primary domain dispatch has already decided the response.
        """

        try:
            return self.authenticate(headers)
        except (AuthenticationError, AuthorizationError):
            return None
