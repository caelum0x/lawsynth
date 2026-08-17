"""Manifest-facing alias for the canonical connector error taxonomy.

The repository manifest lists an ``errors_connector`` module.  The full error
taxonomy already lives in :mod:`lawsynth_connectors.errors`; forking it here
would create two competing sources of truth.  Instead this module is a thin,
documented re-export so the manifest path resolves while the exception classes
remain defined in exactly one place.
"""

from __future__ import annotations

from .errors import (
    ConfigurationError,
    ConnectorConnectionError,
    ConnectorError,
    CredentialError,
    DataValidationError,
    DependencyUnavailableError,
    LimitExceededError,
    QueryError,
    ResourceNotFoundError,
    SnapshotNotFoundError,
    UnsupportedCapabilityError,
)

__all__ = [
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
]
