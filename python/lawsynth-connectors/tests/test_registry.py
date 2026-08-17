"""Thread-safe registry with lazy built-in resolution."""

from __future__ import annotations

import pytest

from lawsynth_connectors import BaseConnector, ConnectorConfig
from lawsynth_connectors.base import ConnectorCapabilities, ReadRequest, Record
from lawsynth_connectors.errors import ConfigurationError
from lawsynth_connectors.registry import _BUILTINS, ConnectorRegistry, registry


class _Dummy(BaseConnector):
    capabilities = ConnectorCapabilities(read=True)

    def _read_records(self, request: ReadRequest) -> list[Record]:
        return [{"ok": 1}]


def _dummy_factory(config: ConnectorConfig, **kw: object) -> BaseConnector:
    return _Dummy(config, **kw)


def test_builtins_cover_expected_connectors() -> None:
    assert set(_BUILTINS) >= {
        "arrow",
        "delta",
        "duckdb",
        "filesystem",
        "http",
        "iceberg",
        "kafka",
        "pandas",
        "polars",
        "postgres",
        "s3",
        "sqlite",
        "xarray",
    }


def test_names_include_builtins() -> None:
    fresh = ConnectorRegistry()
    names = fresh.names()
    assert "filesystem" in names
    assert names == tuple(sorted(names))


def test_register_create_and_capabilities() -> None:
    fresh = ConnectorRegistry()
    fresh.register("dummy", _dummy_factory)
    connector = fresh.create(ConnectorConfig(name="dummy"))
    assert isinstance(connector, _Dummy)
    assert connector.capabilities.read is True


def test_register_rejects_duplicate_without_replace() -> None:
    fresh = ConnectorRegistry()
    fresh.register("dummy", _dummy_factory)
    with pytest.raises(ConfigurationError):
        fresh.register("dummy", _dummy_factory)
    fresh.register("dummy", _dummy_factory, replace=True)  # replace allowed


def test_aliases_resolve_to_canonical() -> None:
    fresh = ConnectorRegistry()
    fresh.register("dummy", _dummy_factory, aliases=["dummy-alias"])
    connector = fresh.create(ConnectorConfig(name="dummy_alias"))
    assert isinstance(connector, _Dummy)


def test_unregister_removes_factory_and_aliases() -> None:
    fresh = ConnectorRegistry()
    fresh.register("dummy", _dummy_factory, aliases=["da"])
    fresh.unregister("dummy")
    with pytest.raises(ConfigurationError):
        fresh.create(ConnectorConfig(name="dummy"))
    with pytest.raises(ConfigurationError):
        fresh.unregister("dummy")


def test_create_unknown_connector_raises() -> None:
    fresh = ConnectorRegistry()
    with pytest.raises(ConfigurationError):
        fresh.create(ConnectorConfig(name="doesnotexist"))


def test_factory_returning_non_connector_rejected() -> None:
    fresh = ConnectorRegistry()
    fresh.register("bad", lambda config, **kw: object())  # type: ignore[arg-type,return-value]
    with pytest.raises(ConfigurationError):
        fresh.create(ConnectorConfig(name="bad"))


def test_lazy_builtin_import_produces_correct_class() -> None:
    fresh = ConnectorRegistry()
    connector = fresh.create(ConnectorConfig(name="filesystem", options={"root": "."}))
    assert type(connector).__name__ == "FilesystemConnector"


def test_global_singleton_resolves_builtin() -> None:
    connector = registry.create(ConnectorConfig(name="filesystem", options={"root": "."}))
    assert type(connector).__name__ == "FilesystemConnector"
