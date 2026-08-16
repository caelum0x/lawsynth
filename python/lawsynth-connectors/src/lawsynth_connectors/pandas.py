"""pandas DataFrame ingestion without making pandas a core dependency."""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from typing import Any

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import ConfigurationError, DataValidationError


def _normalize(value: Any, pd: Any) -> Any:
    if value is pd.NA or value is pd.NaT:
        return None
    if hasattr(value, "item") and callable(value.item):
        value = value.item()
    try:
        missing = pd.isna(value)
        if isinstance(missing, bool) and missing:
            return None
    except (TypeError, ValueError):
        # Array-like extension values do not have one scalar missingness value.
        # They are retained and normalized by the dataframe row conversion.
        missing = False
    return value


def records_from_pandas(
    frame: Any,
    *,
    columns: Sequence[str] = (),
    offset: int = 0,
    limit: int | None = None,
    include_index: bool = False,
) -> Iterable[Record]:
    pd = dependency("pandas", extra="dataframes", connector="pandas")
    if not isinstance(frame, pd.DataFrame):
        raise DataValidationError("expected a pandas.DataFrame", connector="pandas")
    if frame.columns.has_duplicates:
        raise DataValidationError("pandas columns must be unique", connector="pandas")
    selected = list(columns) if columns else list(frame.columns)
    missing = [column for column in selected if column not in frame.columns]
    if missing:
        raise DataValidationError(f"pandas columns do not exist: {missing}")

    stop = None if limit is None else offset + limit
    view = frame.loc[:, selected].iloc[offset:stop]
    for index, row in zip(view.index, view.itertuples(index=False, name=None)):
        record = {
            str(column): _normalize(value, pd)
            for column, value in zip(selected, row, strict=True)
        }
        if include_index:
            record["_index"] = _normalize(index, pd)
        yield record


class PandasConnector(BaseConnector):
    capabilities = ConnectorCapabilities(read=True, projections=True)

    def _resolve(self, request: ReadRequest) -> Any:
        frame = request.options.get("data", self.config.options.get("data"))
        datasets = self.config.options.get("datasets")
        if frame is None and isinstance(datasets, Mapping):
            frame = datasets.get(request.resource)
        if frame is None:
            raise ConfigurationError(f"no pandas data configured for {request.resource!r}")
        return frame

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        include_index = request.options.get("include_index", False)
        if not isinstance(include_index, bool):
            raise ConfigurationError("include_index must be boolean")
        return records_from_pandas(
            self._resolve(request),
            columns=request.columns,
            offset=request.offset,
            limit=request.limit,
            include_index=include_index,
        )

    def _inspect(self, resource: str) -> ResourceInfo:
        frame = self._resolve(ReadRequest(resource))
        pd = dependency("pandas", extra="dataframes", connector="pandas")
        if not isinstance(frame, pd.DataFrame):
            raise DataValidationError("configured value is not a DataFrame")
        return ResourceInfo(
            resource,
            True,
            row_count=len(frame),
            metadata={
                "columns": tuple(map(str, frame.columns)),
                "dtypes": {str(key): str(value) for key, value in frame.dtypes.items()},
            },
        )
