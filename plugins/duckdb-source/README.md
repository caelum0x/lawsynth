# duckdb-source

A LawSynth `data.adapter` plugin that reads numeric time-series data from a local
[DuckDB](https://duckdb.org/) database file using **read-only, SELECT-only**
queries and streamed, bounded result fetching.

DuckDB is a fast embedded analytical engine, which makes it a convenient staging
store for the numeric tables LawSynth ingests. This plugin lets you pull a query
result straight into the canonical record form.

## Safety model

- The connection is opened with `read_only=True`.
- The query must be a single `SELECT` or `WITH` statement. Multiple statements
  (`;`) and any mutating keyword (`insert`, `update`, `delete`, `create`, `drop`,
  `alter`, `copy`, `attach`, `install`, `load`) are rejected before execution.
- Parameters are bound (sequence or mapping), never string-interpolated.
- Results are fetched in batches and capped at `max_rows`.
- The database path is resolved and must be an existing file. The declared
  `filesystem.read` capability is not a grant; the host still confines the path.

## Optional dependency

The `duckdb` driver is **optional**. Importing this package never imports duckdb.
`invoke` imports it lazily and raises a clear `RuntimeError` if it is missing:

```
RuntimeError: duckdb-source requires the duckdb package
```

Install the driver with the extra:

```bash
pip install -e "plugins/duckdb-source[duckdb]"
```

## Contract

```python
from duckdb_source.plugin import DuckDBSource

source = DuckDBSource(max_rows=1_000_000, batch_size=10_000)
result = source.invoke({
    "database": "/path/to/data.duckdb",
    "query": "SELECT time, x, y FROM observations ORDER BY time",
    "params": [],
})
# result == {"records": [...], "row_count": N, "columns": ["time", "x", "y"]}
```

The `records` are ready to feed `lawsynth.dataset.Dataset.from_columns` once you
select the time column and numeric state columns.

See [docs/usage.md](docs/usage.md) and [examples/basic.py](examples/basic.py).
