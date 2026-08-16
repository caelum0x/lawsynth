"""Transparent equation metadata retained alongside native World expressions."""

from dataclasses import dataclass

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class Equation:
    target: str
    expression: str

    def __post_init__(self) -> None:
        if not self.target.isidentifier():
            raise ValidationError("equation target must be an identifier")
        if not self.expression.strip():
            raise ValidationError("equation expression cannot be empty")
