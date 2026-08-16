"""Construction helpers for the native executable World class."""

from collections.abc import Mapping, Sequence

from .equation import Equation
from .errors import ValidationError
from .variable import Variable


def build_world(states: Sequence[Variable], parameters: Mapping[str, float], equations: Sequence[Equation], controls: Sequence[Variable] = ()):
    """Construct a native continuous World after Python-side schema validation."""
    state_names = [variable.name for variable in states]
    control_names = [variable.name for variable in controls]
    if any(variable.role != "state" for variable in states) or any(variable.role != "control" for variable in controls):
        raise ValidationError("states and controls require matching variable roles")
    if set(state_names) & set(control_names) or set(state_names) & set(parameters):
        raise ValidationError("world identifiers must have one namespace")
    equations_by_target = {equation.target: equation.expression for equation in equations}
    if set(equations_by_target) != set(state_names):
        raise ValidationError("exactly one equation is required for each state")
    from ._native import World
    return World(state_names, dict(parameters), equations_by_target, control_names)
