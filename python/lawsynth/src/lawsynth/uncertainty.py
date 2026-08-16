"""Typed uncertainty intervals used by downstream reporting clients."""

from dataclasses import dataclass
from math import isfinite

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class Interval:
    lower: float
    upper: float
    confidence: float = 0.95

    def __post_init__(self) -> None:
        if not all(isfinite(value) for value in (self.lower, self.upper, self.confidence)) or self.lower > self.upper or not 0.0 < self.confidence < 1.0:
            raise ValidationError("invalid uncertainty interval")

    def contains(self, value: float) -> bool:
        return self.lower <= value <= self.upper
