# Arrow connector

`ArrowConnector` (`lawsynth_connectors.arrow`) ingests Apache Arrow data into the
common batch/provenance pipeline. PyArrow is optional: importing the core package
never imports it, and using this connector without `pyarrow` raises
`DependencyUnavailableError` naming the `arrow` extra
(`pip install lawsynth-connectors[arrow]`).

## Sources

`table_from_arrow` accepts:

- a `pyarrow.Table` (used as-is);
- a `pyarrow.RecordBatch` (wrapped into a one-batch table);
- a file path to an Arrow IPC file, tried as the random-access **file** format and
  falling back to the **stream** format, read via `memory_map`;
- any object exposing `to_table(columns=...)` (e.g. a `pyarrow.dataset.Dataset`).

Duplicate field names are rejected with `DataValidationError`. A projection that
names columns not present in the table is also rejected.

## Reading

```python
from lawsynth_connectors import ConnectorConfig, ReadRequest
from lawsynth_connectors.arrow import ArrowConnector

connector = ArrowConnector(ConnectorConfig(name="arrow", options={"data": table}))
with connector:
    rows = connector.read_all(ReadRequest("in-memory", columns=["t", "x"]))
```

The source is resolved from the request options `data`, then the connector's
configured `data`, then a `datasets` mapping keyed by resource name. Rows are
produced with `records_from_arrow`, which honors `columns`, `offset`, and `limit`
and yields plain `dict` records batch-by-batch through `to_pylist()`, so the
result flows into the standard bounded `DataBatch` pipeline with fingerprints.

## Capabilities

`read`, `snapshots`, and `projections`. The connector reads Arrow into records; it
does not write Arrow and performs no LawSynth domain inference.
