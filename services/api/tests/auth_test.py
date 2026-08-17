"""Unit tests for the bearer-token authentication adapter."""

from __future__ import annotations

import pytest

from lawsynth_api.auth import ApiAuthenticator
from lawsynth_api.middleware import RequestProblem
from lawsynth_server.auth import Principal, TokenAuthenticator

TOKEN = "0123456789abcdef0123456789abcdef"


def _authenticator() -> ApiAuthenticator:
    return ApiAuthenticator(TokenAuthenticator({TOKEN: ("acme", frozenset({"read", "write"}))}))


def test_authenticate_extracts_principal_from_header():
    principal = _authenticator().authenticate({"Authorization": f"Bearer {TOKEN}"})
    assert isinstance(principal, Principal)
    assert principal.organization_id == "acme"
    assert principal.scopes == frozenset({"read", "write"})


def test_authenticate_raises_domain_error_on_missing_header():
    from lawsynth_server.errors import AuthenticationError

    with pytest.raises(AuthenticationError):
        _authenticator().authenticate({})


def test_authenticate_or_problem_maps_failure_to_401():
    with pytest.raises(RequestProblem) as excinfo:
        _authenticator().authenticate_or_problem({"Authorization": "Bearer wrong-token-000000000"})
    assert excinfo.value.status == 401
    assert excinfo.value.code == "authentication_required"


def test_or_problem_returns_principal_on_success():
    principal = _authenticator().authenticate_or_problem({"Authorization": f"Bearer {TOKEN}"})
    assert principal.organization_id == "acme"


def test_silent_returns_none_on_failure_and_principal_on_success():
    authenticator = _authenticator()
    assert authenticator.silent({}) is None
    assert authenticator.silent({"Authorization": f"Bearer {TOKEN}"}).organization_id == "acme"
