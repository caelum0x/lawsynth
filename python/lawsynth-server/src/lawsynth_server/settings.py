"""Validated service configuration. Secrets are accepted but never serialized."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class Settings:
    database_url: str = ":memory:"
    object_root: Path = Path(".lawsynth-objects")
    tokens: dict[str, tuple[str, frozenset[str]]] | None = None
    max_page_size: int = 100
    max_upload_bytes: int = 64 * 1024 * 1024
    telemetry_enabled: bool = False

    def __post_init__(self) -> None:
        if not self.database_url or self.max_page_size < 1 or self.max_page_size > 1_000:
            raise ValidationError("invalid server settings")
        if self.max_upload_bytes < 1:
            raise ValidationError("max_upload_bytes must be positive")
        object.__setattr__(self, "object_root", Path(self.object_root))
        object.__setattr__(self, "tokens", self.tokens or {})
