"""Scope authorization at the WSGI boundary.

Tenant isolation is enforced inside the domain repositories (every call is keyed
by ``organization_id``); this module owns the *scope* half of the decision for
the paths the WSGI layer handles directly (SSE streaming).  It wraps the domain
``require_scope`` so that an :class:`AuthorizationError` becomes a 403 transport
:class:`RequestProblem`, keeping the scope vocabulary in one place.
"""

from __future__ import annotations

from lawsynth_server.auth import Principal, require_scope as _require_scope
from lawsynth_server.errors import AuthorizationError

from .middleware import RequestProblem

# The scope vocabulary recognized by token grants (settings validate the set).
READ = "read"
WRITE = "write"
ADMIN = "admin"
SCOPES = frozenset({READ, WRITE, ADMIN})


def require_scope(principal: Principal, scope: str) -> None:
    """Assert ``principal`` holds ``scope`` (or ``admin``); raise the domain error."""

    _require_scope(principal, scope)


def require_scope_or_problem(principal: Principal, scope: str) -> None:
    """Assert ``scope``, mapping a failure to a 403 transport problem."""

    try:
        _require_scope(principal, scope)
    except AuthorizationError as error:
        raise RequestProblem(403, error.code, error.message) from error
