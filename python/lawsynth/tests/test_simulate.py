"""End-to-end scenario simulation through the native extension."""

from lawsynth.equation import Equation
from lawsynth.intervention import Intervention
from lawsynth.scenario import Scenario
from lawsynth.simulate import simulate
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


def test_simulation_applies_a_scheduled_parameter_at_the_exact_boundary():
    if not _native_extension_available():
        return

    world = build_world(
        (Variable("x"),),
        {"rate": 1.0},
        (Equation("x", "rate"),),
    )
    scenario = Scenario(
        initial={"x": 0.0},
        interventions=(Intervention(0.5, "rate", 3.0, "parameter"),),
    )

    trajectory = simulate(world, scenario, start=0.0, end=1.0, step=1.0)

    assert trajectory.time == (0.0, 0.5, 1.0)
    assert trajectory.column("x") == (0.0, 0.5, 2.0)
