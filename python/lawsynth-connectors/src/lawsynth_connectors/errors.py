"""Public connector exceptions with redacted, machine-readable context."""

from __future__ import annotations

from collections.abc import Mapping
from types import MappingProxyType
from typing import Any

_SENSITIVE_FRAGMENTS = (
    "authorization",
    "credential",
    "password",
    "secret",
    "token",
    "api_key",
    "access_key",
)


def _safe_details(details: Mapping[str, Any] | None) -> Mapping[str, Any]:
    if details is None:
        return MappingProxyType({})

    safe: dict[str, Any] = {}
    for key, value in details.items():
        normalized = key.lower().replace("-", "_")
        if any(fragment in normalized for fragment in _SENSITIVE_FRAGMENTS):
            safe[key] = "[REDACTED]"
        elif isinstance(value, (str, int, float, bool, type(None))):
            safe[key] = value
        else:
            safe[key] = repr(value)
    return MappingProxyType(safe)


class ConnectorError(Exception):
    """Base exception surfaced by every connector implementation."""

    default_code = "connector_error"

    def __init__(
        self,
        message: str,
        *,
        connector: str | None = None,
        code: str | None = None,
        retryable: bool = False,
        details: Mapping[str, Any] | None = None,
    ) -> None:
        if not message.strip():
            raise ValueError("connector error message cannot be empty")

        self.message = message
        self.connector = connector
        self.code = code or self.default_code
        self.retryable = retryable
        self.details = _safe_details(details)
        super().__init__(message)

    def as_dict(self) -> dict[str, Any]:
        """Return a serialization-safe representation for APIs and logs."""
        return {
            "code": self.code,
            "message": self.message,
            "retryable": self.retryable,
            **({"connector": self.connector} if self.connector else {}),
            **({"details": dict(self.details)} if self.details else {}),
        }


class ConfigurationError(ConnectorError):
    default_code = "connector_configuration"


class CredentialError(ConnectorError):
    default_code = "connector_credentials"


class DependencyUnavailableError(ConnectorError):
    default_code = "connector_dependency_unavailable"

    def __init__(self, dependency: str, *, extra: str, connector: str) -> None:
        super().__init__(
            f"{connector} requires optional dependency {dependency!r}; "
            f"install lawsynth-connectors[{extra}]",
            connector=connector,
            details={"dependency": dependency, "extra": extra},
        )


# Kept as an alias so callers can use the capability vocabulary without
# having to learn the package's historical exception name.
UnsupportedCapabilityError = DependencyUnavailableError


class ConnectorConnectionError(ConnectorError):
    default_code = "connector_connection"


class QueryError(ConnectorError):
    default_code = "connector_query"


class DataValidationError(ConnectorError):
    default_code = "connector_data_validation"


class ResourceNotFoundError(ConnectorError):
    default_code = "connector_resource_not_found"


class LimitExceededError(ConnectorError):
    default_code = "connector_limit_exceeded"


class SnapshotNotFoundError(ResourceNotFoundError):
    default_code = "connector_snapshot_not_found"
