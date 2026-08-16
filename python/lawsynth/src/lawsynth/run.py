"""Serializable metadata for a completed discovery or simulation execution."""

from dataclasses import dataclass
from datetime import datetime, timezone

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class RunRecord:
    identifier: str
    kind: str
    created_at: datetime
    status: str = "completed"

    @classmethod
    def completed(cls, identifier: str, kind: str) -> "RunRecord":
        return cls(identifier, kind, datetime.now(timezone.utc))

    def __post_init__(self) -> None:
        if not self.identifier or self.kind not in {"discovery", "simulation"} or self.status not in {"completed", "failed", "cancelled"}:
            raise ValidationError("invalid run record")
