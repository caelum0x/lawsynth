"""Tests for the read-only DuckDB source plugin.

Query-validation tests run without the driver. The full round-trip test skips
cleanly when the optional `duckdb` package is not installed.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from duckdb_source.plugin import DuckDBSource, _query


def test_query_accepts_select_and_with() -> None:
    assert _query("SELECT 1") == "SELECT 1"
    assert _query("  select time, x from t;  ") == "select time, x from t"
    assert _query("WITH a AS (SELECT 1) SELECT * FROM a") == "WITH a AS (SELECT 1) SELECT * FROM a"


@pytest.mark.parametrize(
    "bad",
    [
        "INSERT INTO t VALUES (1)",
        "UPDATE t SET x = 1",
        "DELETE FROM t",
        "DROP TABLE t",
        "CREATE TABLE t (x INT)",
        "COPY t TO 'x.csv'",
        "ATTACH 'other.db'",
        "INSTALL httpfs",
        "SELECT 1; DROP TABLE t",  # statement stacking
    ],
)
def test_query_rejects_mutations_and_stacking(bad: str) -> None:
    with pytest.raises(ValueError, match="read-only"):
        _query(bad)


def test_invoke_rejects_missing_database_file(tmp_path: Path) -> None:
    pytest.importorskip("duckdb")
    source = DuckDBSource()
    with pytest.raises(FileNotFoundError):
        source.invoke({"database": str(tmp_path / "nope.duckdb"), "query": "SELECT 1"})


def test_round_trip_reads_numeric_series(tmp_path: Path) -> None:
    duckdb = pytest.importorskip("duckdb")
    database = tmp_path / "observations.duckdb"
    connection = duckdb.connect(str(database))
    connection.execute("CREATE TABLE observations (time DOUBLE, x DOUBLE, y DOUBLE)")
    connection.executemany(
        "INSERT INTO observations VALUES (?, ?, ?)",
        [(0.0, 1.0, 2.0), (1.0, 1.5, 2.5), (2.0, 2.25, 3.1)],
    )
    connection.close()

    source = DuckDBSource(batch_size=2)  # forces multiple fetch batches
    result = source.invoke(
        {"database": str(database), "query": "SELECT time, x, y FROM observations ORDER BY time"}
    )
    assert result["columns"] == ["time", "x", "y"]
    assert result["row_count"] == 3
    assert result["records"][0] == {"time": 0.0, "x": 1.0, "y": 2.0}


def test_max_rows_limit_is_enforced(tmp_path: Path) -> None:
    duckdb = pytest.importorskip("duckdb")
    database = tmp_path / "big.duckdb"
    connection = duckdb.connect(str(database))
    connection.execute("CREATE TABLE t AS SELECT range AS time FROM range(100)")
    connection.close()

    source = DuckDBSource(max_rows=10, batch_size=5)
    with pytest.raises(ValueError, match="max_rows"):
        source.invoke({"database": str(database), "query": "SELECT time FROM t"})
