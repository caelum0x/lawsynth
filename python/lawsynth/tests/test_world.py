"""Native-world construction coverage.

The Python data-model tests deliberately run without the optional extension.  This
test exercises the boundary when the extension has actually been built, while
remaining usable in source-only checkouts.
"""

from lawsynth.equation import Equation
from lawsynth.errors import ValidationError
from lawsynth.variable import Variable
from lawsynth.world import build_world


def _native_extension_available() -> bool:
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name == "lawsynth._native":
            return False
        raise
    return True


def test_build_world_creates_a_native_executable_world():
    if not _native_extension_available():
        return

    world = build_world(
        (Variable("x", "state"),),
        {"rate": 2.0},
        (Equation("x", "rate * x"),),
        (Variable("input", "control"),),
    )

    assert world.equations() == {"x": "(rate*x)"}
    trajectory = world.simulate({"x": 1.0}, end=0.5, step=0.01)
    assert trajectory.time[0] == 0.0
    assert abs(trajectory.values["x"][-1] - 2.718281828459045) < 1e-7


def test_build_world_rejects_an_incomplete_equation_schema_before_native_use():
    try:
        build_world(
            (Variable("x"), Variable("y")),
            {},
            (Equation("x", "y"),),
        )
    except ValidationError:
        pass
    else:
        raise AssertionError("an incomplete state equation schema was accepted")
