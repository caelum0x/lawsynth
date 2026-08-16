"""Validated half-open regime schedules."""

from dataclasses import dataclass
from math import isfinite

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class RegimeInterval:
    name: str
    start: float
    end: float

    def __post_init__(self) -> None:
        if not self.name.isidentifier() or not isfinite(self.start) or not isfinite(self.end) or self.end <= self.start:
            raise ValidationError("regime intervals must be finite and increasing")


@dataclass(frozen=True, slots=True)
class RegimeSchedule:
    intervals: tuple[RegimeInterval, ...]

    def __post_init__(self) -> None:
        ordered = tuple(sorted(self.intervals, key=lambda item: (item.start, item.end, item.name)))
        if any(left.end > right.start for left, right in zip(ordered, ordered[1:])):
            raise ValidationError("regime intervals cannot overlap")
        object.__setattr__(self, "intervals", ordered)

    def active_at(self, time: float) -> RegimeInterval | None:
        return next((item for item in self.intervals if item.start <= time < item.end), None)
