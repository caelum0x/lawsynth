"""Scheduled input and parameter overrides."""

from dataclasses import dataclass
from math import isfinite

from .errors import ValidationError


@dataclass(frozen=True, slots=True, order=True)
class Intervention:
    time: float
    target: str
    value: float
    kind: str = "parameter"

    def __post_init__(self) -> None:
        if not isfinite(self.time) or not isfinite(self.value) or not self.target.isidentifier() or self.kind not in {"parameter", "input"}:
            raise ValidationError("invalid intervention")
