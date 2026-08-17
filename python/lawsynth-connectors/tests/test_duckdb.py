"""DuckDB connector: registration always; real queries when duckdb present."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import DataValidationError, DependencyUnavailableError

from .conftest import records_of


def test_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="duckdb"))
    caps = connector.capabilities
    assert caps.read and caps.predicates and caps.projections and caps.transactions


def test_missing_duckdb_is_explicit_not_a_fallback() -> None:
    connector = registry.create(ConnectorConfig(name="duckdb"))
    try:
        import duckdb  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(DependencyUnavailableError) as raised:
                connector.read(ReadRequest(":memory:", options={"query": "SELECT 1"}))
        assert raised.value.details["dependency"] == "duckdb"
    else:  # pragma: no cover - exercised only when duckdb is installed
        pytest.skip("duckdb installed; covered by query test")


def test_in_memory_query() -> None:
    pytest.importorskip("duckdb")
    connector = registry.create(ConnectorConfig(name="duckdb"))
    with connector:
        rows = records_of(
            connector.read(
                ReadRequest(":memory:", options={"query": "SELECT 1 AS a, 2 AS b"})
            )
        )
    assert rows == [{"a": 1, "b": 2}]


def test_write_query_rejected_even_with_driver() -> None:
    pytest.importorskip("duckdb")
    connector = registry.create(ConnectorConfig(name="duckdb"))
    with connector:
        with pytest.raises(DataValidationError):
            connector.read(ReadRequest(":memory:", options={"query": "CREATE TABLE t (a int)"}))
