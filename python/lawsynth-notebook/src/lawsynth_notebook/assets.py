"""Safe local artifact references; no network or filesystem traversal."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .errors import ArtifactValidationError


@dataclass(frozen=True, slots=True)
class Asset:
    name: str
    media_type: str
    data: bytes

    def __post_init__(self) -> None:
        if not self.name or Path(self.name).is_absolute() or ".." in Path(self.name).parts:
            raise ArtifactValidationError("asset name must be a relative path")
        if "/" not in self.media_type:
            raise ArtifactValidationError("asset media type must be valid")

    @property
    def size(self) -> int:
        return len(self.data)
