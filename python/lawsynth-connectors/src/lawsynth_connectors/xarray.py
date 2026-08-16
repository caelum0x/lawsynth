"""xarray Dataset and DataArray normalization."""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from typing import Any

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import ConfigurationError, DataValidationError


def records_from_xarray(
    value: Any,
    *,
    variables: Sequence[str] = (),
    offset: int = 0,
    limit: int | None = None,
    drop_missing: bool = False,
) -> Iterable[Record]:
    xr = dependency("xarray", extra="dataframes", connector="xarray")
    if isinstance(value, xr.DataArray):
        dataset = value.to_dataset(name=value.name or "value")
    elif isinstance(value, xr.Dataset):
        dataset = value
    else:
        raise DataValidationError("expected an xarray Dataset or DataArray")
    if variables:
        missing = sorted(set(variables) - set(dataset.data_vars))
        if missing:
            raise DataValidationError(f"xarray variables do not exist: {missing}")
        dataset = dataset[list(variables)]
    frame = dataset.to_dataframe()
    if drop_missing:
        frame = frame.dropna(how="all")
    stop = None if limit is None else offset + limit
    yield from frame.reset_index().iloc[offset:stop].to_dict(orient="records")


class XarrayConnector(BaseConnector):
    capabilities = ConnectorCapabilities(read=True, projections=True)

    def _resolve(self, request: ReadRequest) -> Any:
        data = request.options.get("data", self.config.options.get("data"))
        datasets = self.config.options.get("datasets")
        if data is None and isinstance(datasets, Mapping):
            data = datasets.get(request.resource)
        if data is None:
            raise ConfigurationError(f"no xarray data configured for {request.resource!r}")
        return data

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        drop_missing = request.options.get("drop_missing", False)
        if not isinstance(drop_missing, bool):
            raise ConfigurationError("drop_missing must be boolean")
        return records_from_xarray(
            self._resolve(request),
            variables=request.columns,
            offset=request.offset,
            limit=request.limit,
            drop_missing=drop_missing,
        )

    def _inspect(self, resource: str) -> ResourceInfo:
        xr = dependency("xarray", extra="dataframes", connector="xarray")
        value = self._resolve(ReadRequest(resource))
        dataset = value.to_dataset(name=value.name or "value") if isinstance(value, xr.DataArray) else value
        if not isinstance(dataset, xr.Dataset):
            raise DataValidationError("configured value is not xarray data")
        return ResourceInfo(
            resource,
            True,
            metadata={
                "dimensions": dict(dataset.sizes),
                "variables": tuple(dataset.data_vars),
                "coordinates": tuple(dataset.coords),
            },
        )
