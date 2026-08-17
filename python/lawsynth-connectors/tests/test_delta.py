"""Delta connector: partition-filter parsing and dependency degradation."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.delta import _partition_filters
from lawsynth_connectors.errors import ConfigurationError, DependencyUnavailableError


def test_partition_filters_none_passthrough() -> None:
    assert _partition_filters(None) is None


def test_partition_filters_parses_triples() -> None:
    parsed = _partition_filters([("region", "=", "us"), ["year", ">=", 2020]])
    assert parsed == [("region", "=", "us"), ("year", ">=", 2020)]


def test_partition_filters_rejects_non_sequence() -> None:
    with pytest.raises(ConfigurationError):
        _partition_filters("region=us")


def test_partition_filters_rejects_bad_arity() -> None:
    with pytest.raises(ConfigurationError):
        _partition_filters([("region", "=")])


def test_partition_filters_rejects_unknown_operator() -> None:
    with pytest.raises(ConfigurationError):
        _partition_filters([("region", "~", "us")])


def test_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="delta"))
    caps = connector.capabilities
    assert caps.read and caps.snapshots and caps.predicates and caps.projections


def test_missing_deltalake_degrades_on_read() -> None:
    connector = registry.create(ConnectorConfig(name="delta"))
    try:
        import deltalake  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(DependencyUnavailableError) as raised:
                connector.read(ReadRequest("/tmp/does-not-exist"))
        assert raised.value.details["dependency"] == "deltalake"
    else:  # pragma: no cover - only when the driver is installed
        pytest.skip("deltalake installed; no table fixture available")
