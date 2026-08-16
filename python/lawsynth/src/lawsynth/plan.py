"""Immutable, inspectable discovery plans."""

from dataclasses import dataclass

from .errors import ValidationError

STAGES = ("validate", "preprocess", "profile", "differentiate", "generate_features", "fit_laws", "score", "finalize")


@dataclass(frozen=True, slots=True)
class DiscoveryPlan:
    states: tuple[str, ...]
    stages: tuple[str, ...] = STAGES

    def __post_init__(self) -> None:
        if not self.states or len(set(self.states)) != len(self.states) or any(not state.isidentifier() for state in self.states):
            raise ValidationError("states must be distinct identifiers")
        if self.stages != STAGES:
            raise ValidationError("plan stages must retain native execution order")
