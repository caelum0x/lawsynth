"""Iceberg connector: registration and dependency degradation."""

from __future__ import annotations

import pytest

from lawsynth_connectors import ConnectorConfig, ReadRequest, registry
from lawsynth_connectors.errors import DependencyUnavailableError, QueryError
from lawsynth_connectors.iceberg import IcebergConnector


def test_capabilities() -> None:
    connector = registry.create(ConnectorConfig(name="iceberg"))
    caps = connector.capabilities
    assert caps.read and caps.snapshots and caps.predicates and caps.projections


def test_catalog_raises_explicit_dependency_error_when_absent() -> None:
    connector = registry.create(ConnectorConfig(name="iceberg"))
    assert isinstance(connector, IcebergConnector)
    try:
        import pyiceberg  # noqa: F401
    except ImportError:
        with pytest.raises(DependencyUnavailableError) as raised:
            connector._catalog()
        assert raised.value.details["dependency"] == "pyiceberg.catalog"
    else:  # pragma: no cover - exercised only when pyiceberg is installed
        pytest.skip("pyiceberg installed; no catalog fixture available")


def test_read_degrades_to_query_error_when_absent() -> None:
    # ``_read_records`` wraps table loading, so the missing-driver failure
    # surfaces as a QueryError rather than a raw crash.
    connector = registry.create(ConnectorConfig(name="iceberg"))
    try:
        import pyiceberg  # noqa: F401
    except ImportError:
        with connector:
            with pytest.raises(QueryError):
                connector.read(ReadRequest("db.table"))
    else:  # pragma: no cover - exercised only when pyiceberg is installed
        pytest.skip("pyiceberg installed; no catalog fixture available")
