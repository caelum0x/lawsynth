"""Small unit-expression helpers matching the native SI vocabulary."""

from dataclasses import dataclass
from re import fullmatch

from .errors import ValidationError

_UNIT = r"(?:1|m|km|s|min|kg|g)(?:\^-?\d+)?"


@dataclass(frozen=True, slots=True)
class Unit:
    expression: str

    def __post_init__(self) -> None:
        if not self.expression or fullmatch(rf"{_UNIT}(?:[*/]{_UNIT})*", self.expression) is None:
            raise ValidationError(f"invalid native unit expression {self.expression!r}")

    def __str__(self) -> str:
        return self.expression
