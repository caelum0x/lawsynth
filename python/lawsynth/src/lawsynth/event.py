"""Typed event markers for trajectories and scenarios."""

from dataclasses import dataclass
from math import isfinite

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class Event:
    name: str
    time: float
    direction: str = "any"

    def __post_init__(self) -> None:
        if not self.name.isidentifier() or not isfinite(self.time) or self.direction not in {"any", "rising", "falling"}:
            raise ValidationError("invalid event")
