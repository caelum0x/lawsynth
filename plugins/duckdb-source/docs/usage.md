# Usage: duckdb-source

Read numeric time-series data from a local DuckDB file with read-only,
SELECT-only queries and bounded, streamed result fetching.

## Install

```bash
pip install -e "plugins/duckdb-source[duckdb]"
```

The `duckdb` driver is optional. Importing the package never imports duckdb;
`invoke` imports it lazily and raises `RuntimeError` if it is missing.

## Constructing the source

```python
from duckdb_source.plugin import DuckDBSource

source = DuckDBSource(max_rows=1_000_000, batch_size=10_000)
```

Both limits must be positive.

## Request

| key        | type                     | notes                                         |
|------------|--------------------------|-----------------------------------------------|
| `database` | `str`                    | path to an existing `.duckdb` file            |
| `query`    | `str`                    | one `SELECT`/`WITH` statement, no `;` stacking|
| `params`   | sequence or mapping      | bound query parameters (optional)             |

## Response

```python
{"records": [{col: value, ...}, ...], "row_count": int, "columns": [str, ...]}
```

Rows are fetched in `batch_size` chunks and the total is capped at `max_rows`.

## Query safety

The query is validated before execution:

- Must begin with `SELECT` or `WITH`.
- May not contain a `;` (no statement stacking).
- May not contain any mutating/side-effecting keyword: `insert`, `update`,
  `delete`, `create`, `drop`, `alter`, `copy`, `attach`, `install`, `load`.
- The connection is opened with `read_only=True`.

Anything else raises `ValueError`. Bind values through `params` instead of
string interpolation.

## Example

```python
source.invoke({
    "database": "/data/market.duckdb",
    "query": "SELECT time, price FROM ticks WHERE symbol = ? ORDER BY time",
    "params": ["AAPL"],
})
```

## Feeding discovery

Select a time column and numeric state columns, then build a `Dataset`:

```python
records = source.invoke({"database": db, "query": "SELECT time, x, y FROM obs ORDER BY time"})["records"]
time = tuple(r["time"] for r in records)
columns = {"x": tuple(r["x"] for r in records), "y": tuple(r["y"] for r in records)}
# lawsynth.dataset.Dataset.from_columns(time, columns)
```

## Security note

`filesystem.read` in `plugin.toml` is a declaration, not a grant. A conforming
host still confines which paths the plugin may open; see
`specs/plugin-protocol/permissions.md`.
