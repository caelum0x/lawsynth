"""Credential providers that never expose secrets in logs or repr output."""

from __future__ import annotations

import os
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from typing import Protocol, runtime_checkable

from .errors import CredentialError


class SecretValue:
    """A small redacting wrapper around sensitive text."""

    __slots__ = ("__value",)

    def __init__(self, value: str) -> None:
        if not value:
            raise CredentialError("secret value cannot be empty")
        self.__value = value

    def reveal(self) -> str:
        """Return the raw value at the narrow driver boundary."""
        return self.__value

    def __repr__(self) -> str:
        return "SecretValue('[REDACTED]')"

    def __str__(self) -> str:
        return "[REDACTED]"


@runtime_checkable
class CredentialProvider(Protocol):
    """Resolve a named secret without requiring a global credential store."""

    def get(self, name: str) -> SecretValue | None:
        """Return a secret when this provider owns the requested name."""


@dataclass(frozen=True, slots=True)
class StaticCredentialProvider:
    """Explicit credentials useful for dependency injection and notebooks."""

    values: Mapping[str, SecretValue]

    @classmethod
    def from_strings(cls, values: Mapping[str, str]) -> StaticCredentialProvider:
        return cls({name: SecretValue(value) for name, value in values.items()})

    def get(self, name: str) -> SecretValue | None:
        return self.values.get(name)


@dataclass(frozen=True, slots=True)
class EnvironmentCredentialProvider:
    """Read credentials from a constrained environment-variable namespace."""

    prefix: str = "LAWSYNTH_CONNECTOR_"

    def __post_init__(self) -> None:
        if not self.prefix or not self.prefix.replace("_", "A").isalnum():
            raise CredentialError("credential environment prefix is invalid")

    def get(self, name: str) -> SecretValue | None:
        if not name or not name.replace("-", "_").isalnum():
            raise CredentialError(f"invalid credential name: {name!r}")
        key = f"{self.prefix}{name.upper().replace('-', '_')}"
        value = os.environ.get(key)
        return SecretValue(value) if value else None


@dataclass(frozen=True, slots=True)
class CredentialChain:
    """Resolve credentials in a deterministic, caller-controlled order."""

    providers: Sequence[CredentialProvider]

    def get(self, name: str) -> SecretValue | None:
        for provider in self.providers:
            value = provider.get(name)
            if value is not None:
                return value
        return None

    def require(self, name: str, *, connector: str) -> SecretValue:
        value = self.get(name)
        if value is None:
            raise CredentialError(
                f"required credential {name!r} is unavailable",
                connector=connector,
                details={"credential": name},
            )
        return value


EMPTY_CREDENTIALS = CredentialChain(())
