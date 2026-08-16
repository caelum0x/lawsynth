"""Scored executable-law discovery candidates."""

from dataclasses import dataclass
from math import isfinite

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class CandidateMetrics:
    mean_squared_error: float
    complexity: int

    def __post_init__(self) -> None:
        if not isfinite(self.mean_squared_error) or self.mean_squared_error < 0 or self.complexity < 0:
            raise ValidationError("candidate metrics must be finite and non-negative")

    def dominates(self, other: "CandidateMetrics") -> bool:
        return ((self.mean_squared_error <= other.mean_squared_error and self.complexity <= other.complexity) and (self.mean_squared_error < other.mean_squared_error or self.complexity < other.complexity))
