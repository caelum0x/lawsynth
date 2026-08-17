"""xarray connector: registration always; real ingestion when xarray present."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import ConfigurationError, DependencyUnavailableError

from .conftest import records_of


def test_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="xarray"))
    caps = connector.capabilities
    assert caps.read and caps.projections


def test_resolve_without_data_raises_configuration_error() -> None:
    connector = registry.create(ConnectorConfig(name="xarray"))
    with connector:
        with pytest.raises(ConfigurationError):
            connector.read(ReadRequest("unknown"))


def test_missing_xarray_degrades_when_data_present() -> None:
    connector = registry.create(
        ConnectorConfig(name="xarray", options={"data": object()})
    )
    try:
        import xarray  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(DependencyUnavailableError) as raised:
                connector.read(ReadRequest("x"))
        assert raised.value.details["dependency"] == "xarray"
    else:  # pragma: no cover - exercised only when xarray is installed
        pytest.skip("xarray installed; covered by dataset test")


def test_reads_dataset_records() -> None:
    xr = pytest.importorskip("xarray")
    dataset = xr.Dataset(
        {"value": ("t", [1.0, 2.0])},
        coords={"t": [0, 1]},
    )
    connector = registry.create(
        ConnectorConfig(name="xarray", options={"datasets": {"d": dataset}})
    )
    with connector:
        rows = records_of(connector.read(ReadRequest("d")))
    assert [row["value"] for row in rows] == [1.0, 2.0]
