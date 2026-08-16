"""Delta Lake snapshot reader with projection and partition pruning."""

from __future__ import annotations

from collections.abc import Iterable, Sequence
from typing import Any

from ._optional import dependency
from .arrow import records_from_arrow
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import ConfigurationError, QueryError, SnapshotNotFoundError


def _partition_filters(value: Any) -> Sequence[tuple[str, str, Any]] | None:
    if value is None:
        return None
    if not isinstance(value, (list, tuple)):
        raise ConfigurationError("Delta partition_filters must be a sequence")
    filters: list[tuple[str, str, Any]] = []
    for item in value:
        if not isinstance(item, (list, tuple)) or len(item) != 3:
            raise ConfigurationError("each Delta partition filter needs field, operator, value")
        field, operator, expected = item
        if operator not in {"=", "!=", ">", ">=", "<", "<=", "in", "not in"}:
            raise ConfigurationError(f"unsupported Delta partition operator: {operator}")
        filters.append((str(field), str(operator), expected))
    return filters


class DeltaConnector(BaseConnector):
    capabilities = ConnectorCapabilities(
        read=True,
        snapshots=True,
        predicates=True,
        projections=True,
    )

    def _table(self, request: ReadRequest) -> Any:
        deltalake = dependency("deltalake", extra="delta", connector="delta")
        version: int | None = None
        raw_version = request.snapshot or request.options.get("version")
        if raw_version is not None:
            try:
                version = int(raw_version)
            except (TypeError, ValueError) as exc:
                raise ConfigurationError("Delta version must be an integer") from exc
        try:
            return deltalake.DeltaTable(request.resource, version=version)
        except Exception as exc:
            if version is not None:
                raise SnapshotNotFoundError(
                    f"Delta snapshot does not exist: {version}",
                    connector=self.config.name,
                ) from exc
            raise QueryError("Delta table could not be opened") from exc

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        table = self._table(request)
        filters = _partition_filters(request.options.get("partition_filters"))
        try:
            arrow_table = table.to_pyarrow_table(
                columns=list(request.columns) or None,
                partitions=filters,
            )
        except Exception as exc:
            raise QueryError(
                "Delta snapshot scan failed", connector=self.config.name
            ) from exc
        yield from records_from_arrow(
            arrow_table,
            offset=request.offset,
            limit=request.limit,
        )

    def _inspect(self, resource: str) -> ResourceInfo:
        table = self._table(ReadRequest(resource))
        metadata = table.metadata()
        files = tuple(table.file_uris()) if hasattr(table, "file_uris") else ()
        return ResourceInfo(
            resource,
            True,
            kind="delta-table",
            snapshot=str(table.version()),
            metadata={
                "name": metadata.name,
                "description": metadata.description,
                "partition_columns": tuple(metadata.partition_columns),
                "file_count": len(files),
                "schema": str(table.schema()),
            },
        )
