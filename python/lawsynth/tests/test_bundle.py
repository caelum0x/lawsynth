"""Round-trip tests for the native deterministic world bundle format."""

from pathlib import Path
from tempfile import TemporaryDirectory

from lawsynth.bundle import load, save
from lawsynth.equation import Equation
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


def test_bundle_round_trip_preserves_executable_equations():
    if not _native_extension_available():
        return

    world = build_world(
        (Variable("x"),),
        {"rate": 1.5},
        (Equation("x", "rate"),),
    )
    with TemporaryDirectory() as directory:
        path = Path(directory) / "constant_growth.lsworld"
        save(world, path)
        assert path.is_file() and path.stat().st_size > 0
        restored = load(path)

    assert restored.equations() == {"x": "rate"}
    trajectory = restored.simulate({"x": 0.0}, end=1.0, step=0.25)
    assert trajectory.time == [0.0, 0.25, 0.5, 0.75, 1.0]
    assert abs(trajectory.values["x"][-1] - 1.5) < 1e-12
