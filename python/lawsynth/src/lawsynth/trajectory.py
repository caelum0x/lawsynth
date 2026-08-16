"""Immutable simulation trajectory values."""

from dataclasses import dataclass
from typing import Mapping, Sequence

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class TrajectoryData:
    time: tuple[float, ...]
    values: Mapping[str, tuple[float, ...]]

    @classmethod
    def from_native(cls, trajectory: object) -> "TrajectoryData":
        return cls(tuple(trajectory.time), {name: tuple(values) for name, values in trajectory.values.items()})

    def __post_init__(self) -> None:
        if not self.time or any(right <= left for left, right in zip(self.time, self.time[1:])):
            raise ValidationError("trajectory time must be strictly increasing")
        if not self.values or any(len(series) != len(self.time) for series in self.values.values()):
            raise ValidationError("trajectory values must align with time")

    def column(self, name: str) -> Sequence[float]:
        try:
            return self.values[name]
        except KeyError as error:
            raise ValidationError(f"unknown trajectory column {name!r}") from error
