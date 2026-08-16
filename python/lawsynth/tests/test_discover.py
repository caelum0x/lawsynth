"""Native discovery integration using a known exponential-growth trajectory."""

from math import exp

from lawsynth.config import DiscoveryConfig
from lawsynth.dataset import Dataset
from lawsynth.discover import discover


def _native_extension_available() -> bool:
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name == "lawsynth._native":
            return False
        raise
    return True


def test_discovery_returns_an_executable_world_for_known_growth_data():
    if not _native_extension_available():
        return

    time = [index * 0.01 for index in range(101)]
    dataset = Dataset.from_columns(time, {"x": [exp(2.0 * point) for point in time]})
    world = discover(
        dataset,
        ("x",),
        DiscoveryConfig(polynomial_degree=1, threshold=0.01, solver="sr3"),
    )

    equations = world.equations()
    assert set(equations) == {"x"}
    assert "x" in equations["x"]
    trajectory = world.simulate({"x": 1.0}, end=0.1, step=0.01)
    assert trajectory.time[-1] == 0.1
    assert trajectory.values["x"][-1] > 1.1
