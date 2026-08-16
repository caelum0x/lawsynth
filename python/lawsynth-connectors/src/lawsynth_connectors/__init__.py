"""Bounded and reproducible data connectors for LawSynth."""

from ._version import VERSION, __version__
from .base import (
    BaseConnector,
    Connector,
    ConnectorCapabilities,
    ConnectorState,
    DataBatch,
    HealthStatus,
    ReadRequest,
    Record,
    ResourceInfo,
    WriteRequest,
    WriteResult,
)
from .config import ConnectorConfig, RetryPolicy
from .credentials import (
    CredentialChain,
    CredentialProvider,
    EnvironmentCredentialProvider,
    SecretValue,
    StaticCredentialProvider,
)
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
from .fingerprints import DatasetFingerprint, fingerprint_file, fingerprint_records
from .registry import ConnectorRegistry, registry
from .validation import FieldSpec, RecordSchema, ValidationIssue, ValidationReport

__all__ = [
    "VERSION",
    "BaseConnector",
    "ConfigurationError",
    "Connector",
    "ConnectorCapabilities",
    "ConnectorConfig",
    "ConnectorConnectionError",
    "ConnectorError",
    "ConnectorRegistry",
    "ConnectorState",
    "CredentialChain",
    "CredentialError",
    "CredentialProvider",
    "DataBatch",
    "DataValidationError",
    "DatasetFingerprint",
    "DependencyUnavailableError",
    "EnvironmentCredentialProvider",
    "FieldSpec",
    "HealthStatus",
    "LimitExceededError",
    "QueryError",
    "ReadRequest",
    "Record",
    "RecordSchema",
    "ResourceInfo",
    "ResourceNotFoundError",
    "RetryPolicy",
    "SecretValue",
    "SnapshotNotFoundError",
    "StaticCredentialProvider",
    "UnsupportedCapabilityError",
    "ValidationIssue",
    "ValidationReport",
    "WriteRequest",
    "WriteResult",
    "__version__",
    "fingerprint_file",
    "fingerprint_records",
    "registry",
]
