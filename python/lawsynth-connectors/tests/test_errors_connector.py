"""Error taxonomy: redaction, serialization, and the manifest alias module."""

from __future__ import annotations

import pytest

from lawsynth_connectors import errors, errors_connector
from lawsynth_connectors.errors import (
    ConfigurationError,
    ConnectorError,
    DependencyUnavailableError,
    ResourceNotFoundError,
    SnapshotNotFoundError,
    UnsupportedCapabilityError,
)


def test_connector_error_requires_message() -> None:
    with pytest.raises(ValueError):
        ConnectorError("   ")


def test_connector_error_as_dict_includes_code_and_context() -> None:
    error = ConfigurationError("bad", connector="http", details={"field": "x"})
    payload = error.as_dict()
    assert payload["code"] == "connector_configuration"
    assert payload["message"] == "bad"
    assert payload["connector"] == "http"
    assert payload["details"] == {"field": "x"}
    assert payload["retryable"] is False


def test_sensitive_details_are_redacted() -> None:
    error = ConnectorError(
        "boom",
        details={"password": "hunter2", "api_key": "k", "authorization": "Bearer x", "row": 3},
    )
    assert error.details["password"] == "[REDACTED]"
    assert error.details["api_key"] == "[REDACTED]"
    assert error.details["authorization"] == "[REDACTED]"
    assert error.details["row"] == 3


def test_non_scalar_details_are_stringified() -> None:
    error = ConnectorError("boom", details={"payload": {"nested": 1}})
    assert error.details["payload"] == repr({"nested": 1})


def test_dependency_unavailable_error_carries_extra_and_dependency() -> None:
    error = DependencyUnavailableError("duckdb", extra="duckdb", connector="duckdb")
    assert error.details["dependency"] == "duckdb"
    assert error.details["extra"] == "duckdb"
    assert "lawsynth-connectors[duckdb]" in error.message


def test_unsupported_capability_is_dependency_alias() -> None:
    assert UnsupportedCapabilityError is DependencyUnavailableError


def test_snapshot_not_found_is_resource_not_found() -> None:
    assert issubclass(SnapshotNotFoundError, ResourceNotFoundError)


def test_manifest_alias_reexports_identical_objects() -> None:
    for name in errors_connector.__all__:
        assert getattr(errors_connector, name) is getattr(errors, name)


def test_manifest_alias_exports_full_public_taxonomy() -> None:
    expected = {
        "ConfigurationError",
        "ConnectorConnectionError",
        "ConnectorError",
        "CredentialError",
        "DataValidationError",
        "DependencyUnavailableError",
        "LimitExceededError",
        "QueryError",
        "ResourceNotFoundError",
        "SnapshotNotFoundError",
        "UnsupportedCapabilityError",
    }
    assert set(errors_connector.__all__) == expected
