"""Read-only SQL helpers and the stdlib-backed SQLite connector."""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import DataValidationError, ResourceNotFoundError
from lawsynth_connectors.sql import (
    build_select,
    quote_identifier,
    require_read_only_select,
)

from .conftest import records_of


# --- pure SQL helpers -------------------------------------------------------


def test_require_read_only_select_accepts_select_and_with() -> None:
    assert require_read_only_select("SELECT 1;") == "SELECT 1"
    assert require_read_only_select("WITH t AS (SELECT 1) SELECT * FROM t").startswith("WITH")


@pytest.mark.parametrize(
    "query",
    [
        "",
        "DELETE FROM t",
        "UPDATE t SET a = 1",
        "INSERT INTO t VALUES (1)",
        "DROP TABLE t",
        "SELECT 1; SELECT 2",
    ],
)
def test_require_read_only_select_rejects_mutations_and_multi(query: str) -> None:
    with pytest.raises(DataValidationError):
        require_read_only_select(query)


def test_require_read_only_select_strips_comments() -> None:
    assert require_read_only_select("SELECT 1 -- comment") == "SELECT 1"


def test_quote_identifier_validates() -> None:
    assert quote_identifier("table_1") == '"table_1"'
    with pytest.raises(DataValidationError):
        quote_identifier("bad name")
    with pytest.raises(DataValidationError):
        quote_identifier('a"; DROP')


def test_build_select_parameterizes_filters() -> None:
    query, params = build_select(
        "obs", columns=["a", "b"], filters={"a": 1}, limit=5, offset=2
    )
    assert query == 'SELECT "a", "b" FROM "obs" WHERE "a" = :p0 LIMIT :_limit OFFSET :_offset'
    assert params == {"p0": 1, "_limit": 5, "_offset": 2}


def test_build_select_star_when_no_columns() -> None:
    query, params = build_select("obs")
    assert query == 'SELECT * FROM "obs"'
    assert params == {}


# --- SQLite connector -------------------------------------------------------


def _make_db(tmp_path: Path) -> Path:
    database = tmp_path / "obs.sqlite"
    with sqlite3.connect(database) as connection:
        connection.execute("CREATE TABLE observations (time integer, x real)")
        connection.executemany(
            "INSERT INTO observations VALUES (?, ?)", [(0, 1.0), (1, 2.5), (2, 4.0)]
        )
    return database


def test_sqlite_query_is_batched(tmp_path: Path) -> None:
    database = _make_db(tmp_path)
    connector = registry.create(ConnectorConfig(name="sqlite", batch_size=2))
    with connector:
        rows = records_of(
            connector.read(
                ReadRequest(
                    str(database),
                    options={"query": "SELECT time, x FROM observations ORDER BY time"},
                )
            )
        )
    assert rows[-1] == {"time": 2, "x": 4.0}


def test_sqlite_rejects_write_sql(tmp_path: Path) -> None:
    database = _make_db(tmp_path)
    connector = registry.create(ConnectorConfig(name="sqlite"))
    with connector:
        with pytest.raises(DataValidationError):
            connector.read(ReadRequest(str(database), options={"query": "DELETE FROM observations"}))


def test_sqlite_table_projection_via_build_select(tmp_path: Path) -> None:
    database = _make_db(tmp_path)
    connector = registry.create(ConnectorConfig(name="sqlite"))
    with connector:
        rows = records_of(
            connector.read(
                ReadRequest(str(database), columns=["time"], options={"table": "observations"})
            )
        )
    assert all(set(row) == {"time"} for row in rows)


def test_sqlite_missing_database_raises(tmp_path: Path) -> None:
    connector = registry.create(ConnectorConfig(name="sqlite"))
    with connector:
        with pytest.raises(ResourceNotFoundError):
            connector.read(
                ReadRequest(str(tmp_path / "nope.sqlite"), options={"query": "SELECT 1"})
            )


def test_sqlite_inspect_lists_tables(tmp_path: Path) -> None:
    database = _make_db(tmp_path)
    connector = registry.create(ConnectorConfig(name="sqlite"))
    with connector:
        info = connector.inspect(str(database))
    assert "observations" in info.metadata["tables"]


def test_sqlite_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="sqlite"))
    caps = connector.capabilities
    assert caps.read and caps.predicates and caps.projections and caps.transactions
