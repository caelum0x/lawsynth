"""Local, read-only artifact access; remote server proxying is out of scope."""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any

from .errors import ArtifactValidationError, UnsupportedCapabilityError


class LocalArtifactProxy:
    """Read named, already validated artifacts without opening sockets."""
    def __init__(self, artifacts: Mapping[str, Mapping[str, Any]]) -> None:
        if not all(isinstance(name, str) and isinstance(value, Mapping) for name, value in artifacts.items()):
            raise ArtifactValidationError("proxy artifacts must be named objects")
        self._artifacts = {name: dict(value) for name, value in artifacts.items()}

    def get(self, name: str) -> dict[str, Any]:
        try:
            return dict(self._artifacts[name])
        except KeyError as error:
            raise ArtifactValidationError(f"unknown local artifact {name!r}") from error


def connect(*_: object, **__: object) -> None:
    raise UnsupportedCapabilityError("lawsynth-notebook does not implement remote server connections; use a LawSynth API client")
