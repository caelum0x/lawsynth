"""A real stateful progress recorder for local notebook workflows."""

from __future__ import annotations

from dataclasses import dataclass, field

from .errors import ArtifactValidationError


@dataclass(slots=True)
class Progress:
    total: int
    completed: int = 0
    message: str = ""
    history: list[tuple[int, str]] = field(default_factory=list)

    def __post_init__(self) -> None:
        if self.total < 1 or not 0 <= self.completed <= self.total:
            raise ArtifactValidationError("invalid progress bounds")

    def advance(self, amount: int = 1, message: str = "") -> float:
        if amount < 0 or self.completed + amount > self.total:
            raise ArtifactValidationError("progress cannot move outside bounds")
        self.completed += amount
        self.message = message
        self.history.append((self.completed, message))
        return self.fraction

    @property
    def fraction(self) -> float:
        return self.completed / self.total
