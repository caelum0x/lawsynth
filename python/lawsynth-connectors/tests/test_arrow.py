"""Arrow connector: registration always; real ingestion when pyarrow present."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import DependencyUnavailableError

from .conftest import records_of


def test_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="arrow"))
    caps = connector.capabilities
    assert caps.read and caps.snapshots and caps.projections


def test_missing_pyarrow_degrades_on_read() -> None:
    connector = registry.create(ConnectorConfig(name="arrow"))
    try:
        import pyarrow  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(DependencyUnavailableError) as raised:
                connector.read(ReadRequest("dataset"))
        assert raised.value.details["dependency"] == "pyarrow"
    else:  # pragma: no cover - exercised only when pyarrow is installed
        pytest.skip("pyarrow installed; covered by table roundtrip test")


def test_table_roundtrip_and_projection() -> None:
    pa = pytest.importorskip("pyarrow")
    from lawsynth_connectors.arrow import records_from_arrow, table_from_arrow

    table = pa.table({"a": [1, 2, 3], "b": [4, 5, 6]})
    assert table_from_arrow(table, columns=["a"]).column_names == ["a"]
    rows = list(records_from_arrow(table, columns=["a"], offset=1, limit=1))
    assert rows == [{"a": 2}]


def test_connector_reads_configured_table() -> None:
    pa = pytest.importorskip("pyarrow")
    table = pa.table({"a": [1, 2], "b": [3, 4]})
    connector = registry.create(
        ConnectorConfig(name="arrow", options={"datasets": {"t": table}})
    )
    with connector:
        rows = records_of(connector.read(ReadRequest("t")))
    assert rows == [{"a": 1, "b": 3}, {"a": 2, "b": 4}]
