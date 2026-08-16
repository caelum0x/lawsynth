"""Iceberg snapshot scans with catalog and snapshot provenance."""

from __future__ import annotations

from collections.abc import Iterable, Mapping
from typing import Any

from ._optional import dependency
from .arrow import records_from_arrow
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import ConfigurationError, QueryError, SnapshotNotFoundError


class IcebergConnector(BaseConnector):
    capabilities = ConnectorCapabilities(
        read=True,
        snapshots=True,
        predicates=True,
        projections=True,
    )

    def _catalog(self) -> Any:
        catalog_module = dependency(
            "pyiceberg.catalog", extra="iceberg", connector="iceberg"
        )
        catalog_name = str(self.config.options.get("catalog_name", "default"))
        properties = self.config.options.get("catalog", {})
        if not isinstance(properties, Mapping):
            raise ConfigurationError("Iceberg catalog options must be a mapping")
        try:
            return catalog_module.load_catalog(catalog_name, **dict(properties))
        except Exception as exc:
            raise QueryError(
                "Iceberg catalog could not be loaded", connector=self.config.name
            ) from exc

    def _table(self, resource: str) -> Any:
        try:
            return self._catalog().load_table(resource)
        except Exception as exc:
            raise QueryError(
                f"Iceberg table could not be loaded: {resource}",
                connector=self.config.name,
            ) from exc

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        table = self._table(request.resource)
        scan = table.scan(
            row_filter=request.options.get("row_filter"),
            selected_fields=tuple(request.columns) or ("*",),
            limit=request.limit,
        )
        snapshot_id = request.snapshot or request.options.get("snapshot_id")
        if snapshot_id is not None:
            try:
                scan = scan.use_snapshot(int(snapshot_id))
            except Exception as exc:
                raise SnapshotNotFoundError(
                    f"Iceberg snapshot does not exist: {snapshot_id}",
                    connector=self.config.name,
                ) from exc
        try:
            arrow_table = scan.to_arrow()
        except Exception as exc:
            raise QueryError(
                "Iceberg scan failed", connector=self.config.name
            ) from exc
        yield from records_from_arrow(
            arrow_table,
            offset=request.offset,
            limit=request.limit,
        )

    def _inspect(self, resource: str) -> ResourceInfo:
        table = self._table(resource)
        snapshot = table.current_snapshot()
        return ResourceInfo(
            resource,
            True,
            kind="iceberg-table",
            snapshot=(
                str(snapshot.snapshot_id) if snapshot is not None else None
            ),
            metadata={
                "location": table.location(),
                "schema": str(table.schema()),
                "partition_spec": str(table.spec()),
                "sort_order": str(table.sort_order()),
            },
        )
