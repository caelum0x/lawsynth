"""Validated, immutable Python-side time-series datasets."""

from dataclasses import dataclass
from math import isfinite
from typing import Mapping, Sequence

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class Dataset:
    """Aligned finite numeric columns indexed by strictly increasing time."""

    time: tuple[float, ...]
    columns: Mapping[str, tuple[float, ...]]

    @classmethod
    def from_columns(cls, time: Sequence[float], columns: Mapping[str, Sequence[float]]) -> "Dataset":
        return cls(tuple(float(value) for value in time), {name: tuple(float(value) for value in values) for name, values in columns.items()})

    def __post_init__(self) -> None:
        if not self.time or any(not isfinite(value) for value in self.time):
            raise ValidationError("time must contain finite values")
        if any(right <= left for left, right in zip(self.time, self.time[1:])):
            raise ValidationError("time must be strictly increasing")
        if not self.columns:
            raise ValidationError("at least one numeric column is required")
        for name, values in self.columns.items():
            if not name or not name.isidentifier():
                raise ValidationError(f"invalid column identifier {name!r}")
            if len(values) != len(self.time):
                raise ValidationError(f"column {name!r} does not match time length")
            if any(not isfinite(value) for value in values):
                raise ValidationError(f"column {name!r} contains a non-finite value")

    def as_native_arguments(self) -> tuple[list[float], dict[str, list[float]]]:
        """Return owned containers accepted by the native discovery boundary."""
        return list(self.time), {name: list(values) for name, values in self.columns.items()}
