"""pandas connector: registration always; real ingestion when pandas present."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import ConfigurationError, DependencyUnavailableError

from .conftest import records_of


def test_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="pandas"))
    caps = connector.capabilities
    assert caps.read and caps.projections
    assert caps.write is False


def test_resolve_without_data_raises_configuration_error() -> None:
    connector = registry.create(ConnectorConfig(name="pandas"))
    with connector:
        with pytest.raises(ConfigurationError):
            connector.read(ReadRequest("unknown"))


def test_missing_pandas_degrades_when_data_present() -> None:
    connector = registry.create(
        ConnectorConfig(name="pandas", options={"data": [{"a": 1}]})
    )
    try:
        import pandas  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(DependencyUnavailableError) as raised:
                connector.read(ReadRequest("x"))
        assert raised.value.details["dependency"] == "pandas"
    else:  # pragma: no cover - exercised only when pandas is installed
        pytest.skip("pandas installed; covered by frame test")


def test_reads_dataframe_with_projection() -> None:
    pd = pytest.importorskip("pandas")
    frame = pd.DataFrame({"a": [1, 2], "b": [3, 4]})
    connector = registry.create(
        ConnectorConfig(name="pandas", options={"datasets": {"t": frame}})
    )
    with connector:
        rows = records_of(connector.read(ReadRequest("t", columns=["a"])))
    assert rows == [{"a": 1}, {"a": 2}]
