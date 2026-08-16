"""Polars DataFrame and LazyFrame ingestion."""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from typing import Any

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import ConfigurationError, DataValidationError


def records_from_polars(
    frame: Any,
    *,
    columns: Sequence[str] = (),
    offset: int = 0,
    limit: int | None = None,
    streaming: bool = True,
) -> Iterable[Record]:
    pl = dependency("polars", extra="dataframes", connector="polars")
    if isinstance(frame, pl.DataFrame):
        lazy = frame.lazy()
    elif isinstance(frame, pl.LazyFrame):
        lazy = frame
    else:
        raise DataValidationError("expected a Polars DataFrame or LazyFrame")
    schema = lazy.collect_schema()
    if columns:
        missing = sorted(set(columns) - set(schema.names()))
        if missing:
            raise DataValidationError(f"Polars columns do not exist: {missing}")
        lazy = lazy.select(*columns)
    collected = lazy.slice(offset, limit).collect(
        engine="streaming" if streaming else "auto"
    )
    yield from collected.iter_rows(named=True)


class PolarsConnector(BaseConnector):
    capabilities = ConnectorCapabilities(read=True, projections=True)

    def _resolve(self, request: ReadRequest) -> Any:
        frame = request.options.get("data", self.config.options.get("data"))
        datasets = self.config.options.get("datasets")
        if frame is None and isinstance(datasets, Mapping):
            frame = datasets.get(request.resource)
        if frame is None:
            raise ConfigurationError(f"no Polars data configured for {request.resource!r}")
        return frame

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        streaming = request.options.get("streaming", True)
        if not isinstance(streaming, bool):
            raise ConfigurationError("streaming must be boolean")
        return records_from_polars(
            self._resolve(request),
            columns=request.columns,
            offset=request.offset,
            limit=request.limit,
            streaming=streaming,
        )

    def _inspect(self, resource: str) -> ResourceInfo:
        pl = dependency("polars", extra="dataframes", connector="polars")
        frame = self._resolve(ReadRequest(resource))
        lazy = frame.lazy() if isinstance(frame, pl.DataFrame) else frame
        if not isinstance(lazy, pl.LazyFrame):
            raise DataValidationError("configured value is not a Polars frame")
        schema = lazy.collect_schema()
        return ResourceInfo(
            resource,
            True,
            row_count=frame.height if isinstance(frame, pl.DataFrame) else None,
            metadata={"schema": {name: str(dtype) for name, dtype in schema.items()}},
        )
