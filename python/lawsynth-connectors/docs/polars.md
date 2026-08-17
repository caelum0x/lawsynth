# Polars connector

`PolarsConnector` (`lawsynth_connectors.polars`) ingests a Polars `DataFrame` or
`LazyFrame` into the common batch pipeline. Polars is optional and ships under the
`dataframes` extra; using the connector without it raises
`DependencyUnavailableError`.

## Sources and resolution

The frame is resolved from the request options `data`, then the connector's
configured `data`, then a `datasets` mapping keyed by resource name. A missing
frame raises `ConfigurationError`. Both eager `DataFrame` and lazy `LazyFrame`
inputs are accepted; a `DataFrame` is converted to lazy internally.

## Reading

```python
from lawsynth_connectors import ConnectorConfig, ReadRequest
from lawsynth_connectors.polars import PolarsConnector

connector = PolarsConnector(ConnectorConfig(name="polars", options={"data": frame}))
with connector:
    rows = connector.read_all(ReadRequest("frame", columns=["t", "x"]))
```

`records_from_polars` collects lazily: it applies the projection with `select`,
slices by `offset`/`limit`, and collects with the streaming engine by default
(`streaming=True`), then yields named rows via `iter_rows(named=True)`. Requested
columns that do not exist are rejected with `DataValidationError`. The
`streaming` option must be a boolean.

## Capabilities

`read` and `projections`. The connector reads Polars frames into plain records
flowing through the standard bounded `DataBatch` pipeline with fingerprints; it
does not write frames or perform domain inference.
