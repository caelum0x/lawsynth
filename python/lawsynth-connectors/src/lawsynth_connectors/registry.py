"""Thread-safe connector registry with lazy built-in and entry-point loading."""

from __future__ import annotations

from collections.abc import Callable, Iterable
from importlib import import_module
from importlib.metadata import entry_points
from threading import RLock
from typing import Any

from .base import BaseConnector
from .config import ConnectorConfig
from .credentials import CredentialChain, EMPTY_CREDENTIALS
from .errors import ConfigurationError

ConnectorFactory = Callable[..., BaseConnector]

_BUILTINS = {
    "arrow": ("lawsynth_connectors.arrow", "ArrowConnector"),
    "delta": ("lawsynth_connectors.delta", "DeltaConnector"),
    "duckdb": ("lawsynth_connectors.duckdb", "DuckDBConnector"),
    "filesystem": ("lawsynth_connectors.filesystem", "FilesystemConnector"),
    "http": ("lawsynth_connectors.http", "HttpConnector"),
    "iceberg": ("lawsynth_connectors.iceberg", "IcebergConnector"),
    "kafka": ("lawsynth_connectors.kafka", "KafkaConnector"),
    "pandas": ("lawsynth_connectors.pandas", "PandasConnector"),
    "polars": ("lawsynth_connectors.polars", "PolarsConnector"),
    "postgres": ("lawsynth_connectors.postgres", "PostgresConnector"),
    "s3": ("lawsynth_connectors.s3", "S3Connector"),
    "sqlite": ("lawsynth_connectors.sql", "SQLiteConnector"),
    "xarray": ("lawsynth_connectors.xarray", "XarrayConnector"),
}


def _normalize(name: str) -> str:
    normalized = name.strip().lower().replace("_", "-")
    if not normalized or not normalized.replace("-", "a").isalnum():
        raise ConfigurationError(f"invalid connector registry name: {name!r}")
    return normalized


class ConnectorRegistry:
    """Own factories without eagerly importing their optional drivers."""

    def __init__(self) -> None:
        self._factories: dict[str, ConnectorFactory] = {}
        self._aliases: dict[str, str] = {}
        self._entry_points_loaded = False
        self._lock = RLock()

    def register(
        self,
        name: str,
        factory: ConnectorFactory,
        *,
        aliases: Iterable[str] = (),
        replace: bool = False,
    ) -> None:
        canonical = _normalize(name)
        normalized_aliases = tuple(_normalize(alias) for alias in aliases)
        with self._lock:
            if canonical in self._factories and not replace:
                raise ConfigurationError(f"connector {canonical!r} is already registered")
            self._factories[canonical] = factory
            for alias in normalized_aliases:
                if alias in self._aliases and not replace:
                    raise ConfigurationError(f"connector alias {alias!r} is already registered")
                self._aliases[alias] = canonical

    def unregister(self, name: str) -> None:
        canonical = self._aliases.get(_normalize(name), _normalize(name))
        with self._lock:
            if canonical not in self._factories:
                raise ConfigurationError(f"connector {name!r} is not registered")
            del self._factories[canonical]
            self._aliases = {
                alias: target
                for alias, target in self._aliases.items()
                if target != canonical
            }

    def create(
        self,
        config: ConnectorConfig,
        credentials: CredentialChain = EMPTY_CREDENTIALS,
    ) -> BaseConnector:
        name = _normalize(config.name)
        canonical = self._aliases.get(name, name)
        self._ensure_factory(canonical)
        with self._lock:
            factory = self._factories.get(canonical)
        if factory is None:
            raise ConfigurationError(
                f"unknown connector {name!r}; available: {', '.join(self.names())}"
            )
        connector = factory(config, credentials=credentials)
        if not isinstance(connector, BaseConnector):
            raise ConfigurationError("connector factory returned an invalid implementation")
        return connector

    def names(self) -> tuple[str, ...]:
        with self._lock:
            names = set(_BUILTINS) | set(self._factories) | set(self._aliases)
        return tuple(sorted(names))

    def load_entry_points(self) -> None:
        with self._lock:
            if self._entry_points_loaded:
                return
            self._entry_points_loaded = True
        for entry in entry_points(group="lawsynth.connectors"):
            name = _normalize(entry.name)
            with self._lock:
                already_loaded = name in self._factories
            if not already_loaded:
                self.register(name, entry.load())

    def _ensure_factory(self, name: str) -> None:
        with self._lock:
            if name in self._factories:
                return
        builtin = _BUILTINS.get(name)
        if builtin:
            module_name, attribute = builtin
            self.register(
                name,
                getattr(import_module(module_name), attribute),
                replace=True,
            )
            return
        self.load_entry_points()


registry = ConnectorRegistry()
