"""Domain constraints for directed association hypotheses."""

from dataclasses import dataclass

from .errors import ValidationError
from .graph import DependencyEdge


@dataclass(frozen=True, slots=True, order=True)
class EdgeAssumption:
    source: str
    target: str

    def __post_init__(self) -> None:
        if not self.source.isidentifier() or not self.target.isidentifier() or self.source == self.target:
            raise ValidationError("assumption endpoints must be distinct identifiers")


@dataclass(frozen=True, slots=True)
class DependencyAssumptions:
    required: frozenset[EdgeAssumption] = frozenset()
    forbidden: frozenset[EdgeAssumption] = frozenset()

    def __post_init__(self) -> None:
        if self.required & self.forbidden:
            raise ValidationError("an edge cannot be both required and forbidden")

    def permits(self, edge: DependencyEdge) -> bool:
        return EdgeAssumption(edge.source, edge.target) not in self.forbidden
