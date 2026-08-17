"""Unit tests for the scope-authorization adapter."""

from __future__ import annotations

import pytest

from lawsynth_api.authorization import (
    ADMIN,
    READ,
    SCOPES,
    WRITE,
    require_scope,
    require_scope_or_problem,
)
from lawsynth_api.middleware import RequestProblem
from lawsynth_server.auth import Principal
from lawsynth_server.errors import AuthorizationError


def _principal(*scopes: str) -> Principal:
    return Principal(subject="token:test", organization_id="acme", scopes=frozenset(scopes))


def test_scope_vocabulary_is_the_expected_set():
    assert SCOPES == frozenset({READ, WRITE, ADMIN})


def test_require_scope_allows_matching_scope():
    require_scope(_principal(READ), READ)  # does not raise


def test_require_scope_allows_admin_for_any_scope():
    require_scope(_principal(ADMIN), WRITE)  # admin is a superset


def test_require_scope_rejects_missing_scope_with_domain_error():
    with pytest.raises(AuthorizationError):
        require_scope(_principal(READ), WRITE)


def test_require_scope_or_problem_maps_failure_to_403():
    with pytest.raises(RequestProblem) as excinfo:
        require_scope_or_problem(_principal(READ), WRITE)
    assert excinfo.value.status == 403
    assert excinfo.value.code == "forbidden"


def test_require_scope_or_problem_passes_when_authorized():
    require_scope_or_problem(_principal(READ, WRITE), WRITE)  # does not raise
