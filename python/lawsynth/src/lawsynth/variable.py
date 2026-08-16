"""Declared World variables and their semantic roles."""

from dataclasses import dataclass

from .errors import ValidationError

_ROLES = frozenset({"state", "control", "exogenous", "observed", "latent", "derived"})


@dataclass(frozen=True, slots=True)
class Variable:
    name: str
    role: str = "state"
    unit: str | None = None

    def __post_init__(self) -> None:
        if not self.name.isidentifier():
            raise ValidationError("variable name must be a Python-compatible identifier")
        if self.role not in _ROLES:
            raise ValidationError(f"unknown variable role {self.role!r}")
        if self.unit is not None and not self.unit:
            raise ValidationError("unit cannot be empty")
