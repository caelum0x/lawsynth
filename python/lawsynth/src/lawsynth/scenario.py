"""A reusable simulation request bound to a native World."""

from dataclasses import dataclass, field
from typing import Mapping

from .errors import ValidationError
from .intervention import Intervention


@dataclass(frozen=True, slots=True)
class Scenario:
    initial: Mapping[str, float]
    parameters: Mapping[str, float] = field(default_factory=dict)
    inputs: Mapping[str, float] = field(default_factory=dict)
    interventions: tuple[Intervention, ...] = ()

    def __post_init__(self) -> None:
        for values in (self.initial, self.parameters, self.inputs):
            if any(not name.isidentifier() for name in values):
                raise ValidationError("scenario values need identifier keys")
        if tuple(sorted(self.interventions)) != self.interventions:
            object.__setattr__(self, "interventions", tuple(sorted(self.interventions)))
