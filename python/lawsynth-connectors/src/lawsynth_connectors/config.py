"""Immutable connector configuration and resource bounds."""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass, field
from types import MappingProxyType
from typing import Any

from .errors import ConfigurationError

_SECRET_KEYS = {
    "access_key",
    "api_key",
    "authorization",
    "password",
    "secret",
    "secret_key",
    "token",
}


@dataclass(frozen=True, slots=True)
class RetryPolicy:
    """Retry policy used only for failures marked retryable by an adapter."""

    attempts: int = 3
    initial_delay_seconds: float = 0.25
    maximum_delay_seconds: float = 8.0
    multiplier: float = 2.0
    jitter_ratio: float = 0.1

    def __post_init__(self) -> None:
        if not 1 <= self.attempts <= 10:
            raise ConfigurationError("retry attempts must be in 1..10")
        if self.initial_delay_seconds < 0:
            raise ConfigurationError("initial retry delay cannot be negative")
        if self.maximum_delay_seconds < self.initial_delay_seconds:
            raise ConfigurationError("maximum retry delay is below the initial delay")
        if self.multiplier < 1:
            raise ConfigurationError("retry multiplier must be at least one")
        if not 0 <= self.jitter_ratio <= 1:
            raise ConfigurationError("retry jitter ratio must be in 0..1")

    def delay_for(self, retry_number: int) -> float:
        if retry_number < 0:
            raise ValueError("retry number cannot be negative")
        delay = self.initial_delay_seconds * self.multiplier**retry_number
        return min(delay, self.maximum_delay_seconds)


@dataclass(frozen=True, slots=True)
class ConnectorConfig:
    """Common configuration applied uniformly to concrete connectors."""

    name: str
    batch_size: int = 1_000
    max_rows: int = 1_000_000
    max_bytes: int = 512 * 1024 * 1024
    timeout_seconds: float = 30.0
    retry: RetryPolicy = field(default_factory=RetryPolicy)
    options: Mapping[str, Any] = field(default_factory=dict)

    def __post_init__(self) -> None:
        name = self.name.strip().lower()
        if not name or not name.replace("-", "_").isidentifier():
            raise ConfigurationError(f"invalid connector name: {self.name!r}")
        if not 1 <= self.batch_size <= 1_000_000:
            raise ConfigurationError("batch_size must be in 1..1,000,000")
        if not 1 <= self.max_rows <= 1_000_000_000:
            raise ConfigurationError("max_rows must be in 1..1,000,000,000")
        if not 1 <= self.max_bytes <= 1024**4:
            raise ConfigurationError("max_bytes must be in 1 byte..1 TiB")
        if not 0 < self.timeout_seconds <= 3_600:
            raise ConfigurationError("timeout_seconds must be in (0, 3600]")

        options = dict(self.options)
        for key in options:
            normalized = key.lower().replace("-", "_")
            if normalized in _SECRET_KEYS:
                raise ConfigurationError(
                    f"secret option {key!r} must be supplied by a credential provider"
                )

        object.__setattr__(self, "name", name)
        object.__setattr__(self, "options", MappingProxyType(options))

    @classmethod
    def from_mapping(cls, values: Mapping[str, Any]) -> ConnectorConfig:
        """Parse an untrusted configuration mapping with explicit fields."""
        allowed = {
            "name",
            "batch_size",
            "max_rows",
            "max_bytes",
            "timeout_seconds",
            "retry",
            "options",
        }
        unknown = set(values) - allowed
        if unknown:
            raise ConfigurationError(
                f"unknown connector configuration fields: {sorted(unknown)}"
            )

        retry_value = values.get("retry")
        retry = (
            RetryPolicy(**dict(retry_value))
            if isinstance(retry_value, Mapping)
            else retry_value or RetryPolicy()
        )
        return cls(
            name=str(values.get("name", "")),
            batch_size=int(values.get("batch_size", 1_000)),
            max_rows=int(values.get("max_rows", 1_000_000)),
            max_bytes=int(values.get("max_bytes", 512 * 1024 * 1024)),
            timeout_seconds=float(values.get("timeout_seconds", 30.0)),
            retry=retry,
            options=dict(values.get("options", {})),
        )

    def option(
        self,
        key: str,
        expected_type: type[Any],
        *,
        default: Any = None,
        required: bool = False,
    ) -> Any:
        value = self.options.get(key, default)
        if required and value is None:
            raise ConfigurationError(
                f"connector {self.name!r} requires option {key!r}",
                connector=self.name,
            )
        if value is not None and not isinstance(value, expected_type):
            raise ConfigurationError(
                f"connector option {key!r} must be {expected_type.__name__}",
                connector=self.name,
            )
        return value
