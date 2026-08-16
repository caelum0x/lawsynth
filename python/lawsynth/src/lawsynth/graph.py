"""Lagged association graph values returned by discovery diagnostics."""

from dataclasses import dataclass
from math import isfinite

from .errors import ValidationError


@dataclass(frozen=True, slots=True, order=True)
class DependencyEdge:
    source: str
    target: str
    lag: int
    correlation: float

    def __post_init__(self) -> None:
        if not self.source.isidentifier() or not self.target.isidentifier() or self.source == self.target:
            raise ValidationError("dependency endpoints must be distinct identifiers")
        if self.lag < 1 or not isfinite(self.correlation) or abs(self.correlation) > 1.0:
            raise ValidationError("dependency lag/correlation is invalid")


@dataclass(frozen=True, slots=True)
class DependencyGraph:
    edges: tuple[DependencyEdge, ...] = ()

    def __post_init__(self) -> None:
        if len(set(self.edges)) != len(self.edges):
            raise ValidationError("dependency graph contains duplicate edges")
