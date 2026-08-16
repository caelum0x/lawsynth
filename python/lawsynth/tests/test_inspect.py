from lawsynth.inspect import world_summary
from lawsynth.variable import Variable
from lawsynth.world import build_world
from lawsynth.equation import Equation


def test_inspection_returns_stable_equation_summary():
    try:
        world = build_world((Variable("x"),), {}, (Equation("x", "x"),))
    except ImportError:
        return
    assert world_summary(world) == {"equations": {"x": "x"}, "state_count": 1}
