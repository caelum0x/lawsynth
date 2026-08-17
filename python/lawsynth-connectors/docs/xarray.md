# xarray connector

`XarrayConnector` (`lawsynth_connectors.xarray`) normalizes an xarray `Dataset` or
`DataArray` into flat records for the common batch pipeline. xarray is optional and
ships under the `dataframes` extra; using the connector without it raises
`DependencyUnavailableError`.

## Sources and resolution

The value is resolved from the request options `data`, then the connector's
configured `data`, then a `datasets` mapping keyed by resource name. A missing
value raises `ConfigurationError`. A `DataArray` is promoted to a single-variable
`Dataset` (named `value` when the array is unnamed).

## Normalization

`records_from_xarray` selects the requested variables (rejecting unknown ones with
`DataValidationError`), converts the dataset to a pandas frame with
`to_dataframe()`, optionally drops all-missing rows when `drop_missing=True`,
resets the index so coordinate dimensions become columns, and yields
`offset`/`limit`-bounded `dict` records. Because coordinates are flattened into
columns, a multi-dimensional dataset becomes a rectangular record stream suitable
for LawSynth ingestion.

```python
from lawsynth_connectors import ConnectorConfig, ReadRequest
from lawsynth_connectors.xarray import XarrayConnector

connector = XarrayConnector(ConnectorConfig(name="xarray", options={"data": dataset}))
with connector:
    rows = connector.read_all(ReadRequest("grid", columns=["temperature"]))
```

`drop_missing` must be a boolean. Capabilities: `read` and `projections`. Records
flow through the standard bounded `DataBatch` pipeline with fingerprints.
