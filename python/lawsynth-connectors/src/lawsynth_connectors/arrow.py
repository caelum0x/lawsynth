"""Arrow Table, RecordBatch, Dataset, and IPC ingestion."""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from pathlib import Path
from typing import Any

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import ConfigurationError, DataValidationError, ResourceNotFoundError


def table_from_arrow(source: Any, *, columns: Sequence[str] = ()) -> Any:
    pa = dependency("pyarrow", extra="arrow", connector="arrow")
    if isinstance(source, pa.Table):
        table = source
    elif isinstance(source, pa.RecordBatch):
        table = pa.Table.from_batches([source])
    elif isinstance(source, (str, Path)):
        path = Path(source)
        if not path.is_file():
            raise ResourceNotFoundError(f"Arrow resource does not exist: {path}")
        with pa.memory_map(str(path), "r") as mapped:
            try:
                table = pa.ipc.open_file(mapped).read_all()
            except pa.ArrowInvalid:
                table = pa.ipc.open_stream(mapped).read_all()
    elif hasattr(source, "to_table"):
        table = source.to_table(columns=list(columns) or None)
    else:
        raise DataValidationError("unsupported Arrow source", connector="arrow")
    if len(table.column_names) != len(set(table.column_names)):
        raise DataValidationError("Arrow fields must be unique", connector="arrow")
    if columns:
        missing = sorted(set(columns) - set(table.column_names))
        if missing:
            raise DataValidationError(f"Arrow columns do not exist: {missing}")
        table = table.select(columns)
    return table


def records_from_arrow(
    source: Any,
    *,
    columns: Sequence[str] = (),
    offset: int = 0,
    limit: int | None = None,
) -> Iterable[Record]:
    table = table_from_arrow(source, columns=columns)
    length = max(0, table.num_rows - offset) if limit is None else limit
    for batch in table.slice(offset, length).to_batches():
        yield from batch.to_pylist()


class ArrowConnector(BaseConnector):
    capabilities = ConnectorCapabilities(read=True, snapshots=True, projections=True)

    def _resolve(self, request: ReadRequest) -> Any:
        source = request.options.get("data", self.config.options.get("data"))
        datasets = self.config.options.get("datasets")
        if source is None and isinstance(datasets, Mapping):
            source = datasets.get(request.resource)
        return request.resource if source is None else source

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        return records_from_arrow(
            self._resolve(request),
            columns=request.columns,
            offset=request.offset,
            limit=request.limit,
        )

    def _inspect(self, resource: str) -> ResourceInfo:
        table = table_from_arrow(self._resolve(ReadRequest(resource)))
        return ResourceInfo(
            resource,
            True,
            row_count=table.num_rows,
            byte_count=table.nbytes,
            metadata={"columns": tuple(table.column_names), "schema": str(table.schema)},
        )
