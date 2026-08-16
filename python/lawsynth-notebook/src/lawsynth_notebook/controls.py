"""Validated declarative controls for notebook consumers."""

from __future__ import annotations

from dataclasses import dataclass

from .errors import ArtifactValidationError
from .serialization import finite_number


@dataclass(frozen=True, slots=True)
class RangeControl:
    name: str
    lower: float
    upper: float
    value: float
    step: float = 1.0

    def __post_init__(self) -> None:
        lower, upper, value, step = (finite_number(self.lower, "lower"), finite_number(self.upper, "upper"), finite_number(self.value, "value"), finite_number(self.step, "step"))
        if not self.name or lower >= upper or not lower <= value <= upper or step <= 0:
            raise ArtifactValidationError("invalid range control")
