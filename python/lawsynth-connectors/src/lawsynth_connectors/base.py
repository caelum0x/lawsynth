"""Connector lifecycle, bounded batching, and provenance contracts."""

from __future__ import annotations

import time
from collections.abc import Iterable, Iterator, Mapping, Sequence
from dataclasses import dataclass, field
from enum import Enum
from types import MappingProxyType
from typing import Any, Literal, TypeAlias

from .config import ConnectorConfig
from .credentials import CredentialChain, EMPTY_CREDENTIALS
from .errors import ConfigurationError, ConnectorError, LimitExceededError
from .fingerprints import DatasetFingerprint, fingerprint_records
from .pagination import chunked
from .validation import RecordSchema, validate_numeric_dataset, validate_records

Record: TypeAlias = Mapping[str, Any]
WriteMode = Literal["append", "replace", "error"]


class ConnectorState(str, Enum):
    NEW = "new"
    CONNECTED = "connected"
    CLOSED = "closed"
    FAILED = "failed"


@dataclass(frozen=True, slots=True)
class ConnectorCapabilities:
    read: bool = True
    write: bool = False
    snapshots: bool = False
    predicates: bool = False
    projections: bool = False
    streaming: bool = False
    transactions: bool = False


@dataclass(frozen=True, slots=True)
class ReadRequest:
    resource: str
    columns: Sequence[str] = ()
    filters: Mapping[str, Any] = field(default_factory=dict)
    limit: int | None = None
    offset: int = 0
    snapshot: str | None = None
    schema: RecordSchema | None = None
    numeric: bool = False
    time_column: str | None = None
    options: Mapping[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.resource.strip():
            raise ConfigurationError("read resource cannot be empty")
        if self.limit is not None and self.limit < 1:
            raise ConfigurationError("read limit must be positive")
        if self.offset < 0:
            raise ConfigurationError("read offset cannot be negative")
        if len(set(self.columns)) != len(self.columns):
            raise ConfigurationError("read projection contains duplicate columns")
        if self.time_column is not None and not self.time_column.strip():
            raise ConfigurationError("time_column cannot be blank")
        object.__setattr__(self, "columns", tuple(self.columns))
        object.__setattr__(self, "filters", MappingProxyType(dict(self.filters)))
        object.__setattr__(self, "options", MappingProxyType(dict(self.options)))


@dataclass(frozen=True, slots=True)
class WriteRequest:
    resource: str
    mode: WriteMode = "error"
    partition_by: Sequence[str] = ()
    options: Mapping[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.resource.strip():
            raise ConfigurationError("write resource cannot be empty")
        if self.mode not in {"append", "replace", "error"}:
            raise ConfigurationError(f"unsupported write mode: {self.mode}")
        if len(set(self.partition_by)) != len(self.partition_by):
            raise ConfigurationError("write partition fields contain duplicates")
        object.__setattr__(self, "partition_by", tuple(self.partition_by))
        object.__setattr__(self, "options", MappingProxyType(dict(self.options)))


@dataclass(frozen=True, slots=True)
class DataBatch:
    records: Sequence[Record]
    fingerprint: DatasetFingerprint
    source: str
    index: int = 0
    snapshot: Mapping[str, Any] = field(default_factory=dict)
    metadata: Mapping[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if self.index < 0:
            raise ValueError("batch index cannot be negative")
        if not self.source:
            raise ValueError("batch source cannot be empty")
        object.__setattr__(
            self,
            "records",
            tuple(MappingProxyType(dict(record)) for record in self.records),
        )
        object.__setattr__(self, "snapshot", MappingProxyType(dict(self.snapshot)))
        object.__setattr__(self, "metadata", MappingProxyType(dict(self.metadata)))

    @classmethod
    def from_records(
        cls,
        records: Sequence[Record],
        *,
        source: str,
        index: int = 0,
        snapshot: Mapping[str, Any] | None = None,
        metadata: Mapping[str, Any] | None = None,
    ) -> DataBatch:
        rows = tuple(records)
        return cls(
            records=rows,
            fingerprint=fingerprint_records(rows),
            source=source,
            index=index,
            snapshot=snapshot or {},
            metadata=metadata or {},
        )

    @property
    def row_count(self) -> int:
        return len(self.records)


@dataclass(frozen=True, slots=True)
class WriteResult:
    resource: str
    row_count: int
    batch_count: int
    fingerprint: DatasetFingerprint
    metadata: Mapping[str, Any] = field(default_factory=dict)


@dataclass(frozen=True, slots=True)
class ResourceInfo:
    resource: str
    exists: bool
    kind: str = "dataset"
    schema: RecordSchema | None = None
    row_count: int | None = None
    byte_count: int | None = None
    snapshot: str | None = None
    metadata: Mapping[str, Any] = field(default_factory=dict)


@dataclass(frozen=True, slots=True)
class HealthStatus:
    healthy: bool
    connector: str
    state: ConnectorState
    latency_seconds: float
    message: str = ""


class BaseConnector:
    """Own lifecycle, validation, limits, batching, and provenance behavior."""

    capabilities = ConnectorCapabilities()

    def __init__(
        self,
        config: ConnectorConfig,
        credentials: CredentialChain = EMPTY_CREDENTIALS,
        **kwargs: Any,
    ) -> None:
        if "credentials" in kwargs:
            credentials = kwargs.pop("credentials")
        if kwargs:
            raise TypeError(f"unexpected connector arguments: {sorted(kwargs)}")
        self.config = config
        self.credentials = credentials
        self._state = ConnectorState.NEW

    @property
    def state(self) -> ConnectorState:
        return self._state

    def connect(self) -> None:
        if self._state is ConnectorState.CONNECTED:
            return
        if self._state is ConnectorState.CLOSED:
            raise ConnectorError(
                "a closed connector cannot be reconnected",
                connector=self.config.name,
            )
        try:
            self._connect()
        except ConnectorError:
            self._state = ConnectorState.FAILED
            raise
        except Exception as exc:
            self._state = ConnectorState.FAILED
            raise ConnectorError(
                "connector initialization failed",
                connector=self.config.name,
                details={"exception_type": type(exc).__name__},
            ) from exc
        self._state = ConnectorState.CONNECTED

    def close(self) -> None:
        if self._state is ConnectorState.CLOSED:
            return
        try:
            self._close()
        finally:
            self._state = ConnectorState.CLOSED

    def read(self, request: ReadRequest) -> tuple[DataBatch, ...]:
        self._require_connected()
        return tuple(self._bounded_batches(self._read_records(request), request))

    def read_all(self, request: ReadRequest) -> tuple[Record, ...]:
        return tuple(record for batch in self.read(request) for record in batch.records)

    def _bounded_batches(
        self,
        records: Iterable[Record],
        request: ReadRequest,
        *,
        source: str | None = None,
        snapshot: Mapping[str, Any] | None = None,
    ) -> Iterator[DataBatch]:
        maximum = min(request.limit or self.config.max_rows, self.config.max_rows)
        total_rows = 0
        total_bytes = 0

        for index, values in enumerate(chunked(records, self.config.batch_size)):
            if total_rows + len(values) > maximum:
                values = values[: maximum - total_rows]
            if not values:
                break
            projected = self._project_batch(values, request.columns)
            self._validate(projected, request)
            fingerprint = fingerprint_records(projected)
            total_bytes += fingerprint.byte_count or 0
            if total_bytes > self.config.max_bytes:
                raise LimitExceededError(
                    "connector read exceeded max_bytes",
                    connector=self.config.name,
                    details={"max_bytes": self.config.max_bytes},
                )
            total_rows += len(projected)
            yield DataBatch.from_records(
                projected,
                source=source or request.resource,
                index=index,
                snapshot=snapshot,
                metadata={"connector": self.config.name, "rows_so_far": total_rows},
            )
            if total_rows >= maximum:
                break

    def _make_batches(
        self,
        records: Sequence[Record],
        request: ReadRequest,
        *,
        source: str,
        snapshot: Mapping[str, Any] | None = None,
    ) -> tuple[DataBatch, ...]:
        """Compatibility hook for adapters while they migrate to generators."""
        stop = None if request.limit is None else request.offset + request.limit
        selected = records[request.offset:stop]
        if len(selected) > self.config.max_rows:
            raise LimitExceededError(
                "source exceeded configured row limit",
                connector=self.config.name,
                details={"max_rows": self.config.max_rows},
            )
        return tuple(
            self._bounded_batches(selected, request, source=source, snapshot=snapshot)
        )

    def write(self, request: WriteRequest, records: Iterable[Record]) -> WriteResult:
        if not self.capabilities.write:
            raise ConfigurationError(
                "connector does not support writes", connector=self.config.name
            )
        self._require_connected()
        accepted: list[Record] = []
        batch_count = 0
        total_bytes = 0
        for values in chunked(records, self.config.batch_size):
            if len(accepted) + len(values) > self.config.max_rows:
                raise LimitExceededError("connector write exceeded max_rows")
            fingerprint = fingerprint_records(values)
            total_bytes += fingerprint.byte_count or 0
            if total_bytes > self.config.max_bytes:
                raise LimitExceededError("connector write exceeded max_bytes")
            self._write_records(request, values, first_batch=batch_count == 0)
            accepted.extend(values)
            batch_count += 1
        return WriteResult(
            request.resource,
            len(accepted),
            batch_count,
            fingerprint_records(accepted),
            {"connector": self.config.name},
        )

    def inspect(self, resource: str) -> ResourceInfo:
        self._require_connected()
        return self._inspect(resource)

    def health(self) -> HealthStatus:
        started = time.monotonic()
        try:
            message = self._health_message() if self._state is ConnectorState.CONNECTED else "not connected"
            return HealthStatus(
                self._state is ConnectorState.CONNECTED,
                self.config.name,
                self._state,
                time.monotonic() - started,
                message,
            )
        except Exception as exc:
            return HealthStatus(
                False,
                self.config.name,
                self._state,
                time.monotonic() - started,
                f"health check failed: {type(exc).__name__}",
            )

    def _project_batch(
        self,
        records: Sequence[Record],
        columns: Sequence[str],
    ) -> tuple[Record, ...]:
        if not columns:
            return tuple(records)
        projected: list[Record] = []
        for record in records:
            missing = set(columns) - set(record)
            if missing:
                raise ConfigurationError(f"projection columns do not exist: {sorted(missing)}")
            projected.append({column: record[column] for column in columns})
        return tuple(projected)

    def _validate(self, records: Sequence[Record], request: ReadRequest) -> None:
        if request.schema is not None:
            validate_records(records, request.schema).raise_for_errors(
                connector=self.config.name
            )
        if request.numeric:
            validate_numeric_dataset(
                records,
                time_column=request.time_column,
                connector=self.config.name,
            )

    def _require_connected(self) -> None:
        if self._state is not ConnectorState.CONNECTED:
            raise ConnectorError(
                "connector must be connected before use",
                connector=self.config.name,
                details={"state": self._state.value},
            )

    def __enter__(self) -> BaseConnector:
        self.connect()
        return self

    def __exit__(self, *_exc: object) -> None:
        self.close()

    def _connect(self) -> None:
        """Validate a stateless connector's declared capability surface.

        Stateless adapters (for example in-memory dataframe adapters) do not
        need an external handle.  They still reach this hook so a malformed
        adapter cannot silently advertise no usable operation.
        """
        if not self.capabilities.read and not self.capabilities.write:
            raise ConfigurationError(
                "connector must expose at least one of read or write",
                connector=self.config.name,
            )

    def _close(self) -> None:
        """Stateless adapters have no external handle to release.

        Stateful implementations override this hook to close their driver;
        ``close`` always transitions the public lifecycle state afterward.
        """
        return None

    def _health_message(self) -> str:
        return "ready"

    def _inspect(self, resource: str) -> ResourceInfo:
        return ResourceInfo(resource=resource, exists=True)

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        raise ConfigurationError(
            "connector does not implement reads", connector=self.config.name
        )

    def _write_records(
        self,
        request: WriteRequest,
        records: Sequence[Record],
        *,
        first_batch: bool,
    ) -> None:
        raise ConfigurationError(
            "connector does not implement writes", connector=self.config.name
        )


# Compatibility name for early plugin implementations.
Connector = BaseConnector
