"""Phase 1 Python/native conformance test for executable worlds."""

from __future__ import annotations

import os
import math
import shutil
import subprocess
import sys
import sysconfig
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


def main() -> None:
    environment = os.environ | {
        "RUSTFLAGS": "-C link-arg=-undefined -C link-arg=dynamic_lookup",
    }
    subprocess.run(
        ["cargo", "build", "-p", "lawsynth-python", "--quiet"],
        cwd=ROOT,
        env=environment,
        check=True,
    )
    suffix = sysconfig.get_config_var("EXT_SUFFIX")
    if not suffix:
        raise RuntimeError("Python extension suffix is unavailable")
    with tempfile.TemporaryDirectory() as temporary:
        package = Path(temporary) / "lawsynth"
        package.mkdir()
        shutil.copy(ROOT / "python/lawsynth/src/lawsynth/__init__.py", package / "__init__.py")
        shutil.copy(ROOT / "target/debug/lib_native.dylib", package / f"_native{suffix}")
        sys.path.insert(0, temporary)
        from lawsynth import Scenario, World, discover

        cases = [
            ("constant", "1.0", {}, 0.5),
            ("growth", "x", {}, 0.5),
            ("decay", "-(x)", {}, 0.5),
            ("quadratic", "x*x", {}, 0.5),
            ("controlled", "x+u", {}, 0.5),
        ]
        for name, equation, parameters, initial in cases:
            states = ["x"]
            expression = equation
            world = World(states, parameters, {"x": expression})
            if name == "controlled":
                world = World(states, parameters, {"x": expression}, controls=["u"])
                trajectory = Scenario(world, {"x": initial}, inputs={"u": 0.25}).simulate(end=0.1, step=0.01)
            else:
                trajectory = world.simulate({"x": initial}, end=0.1, step=0.01)
            bundle = Path(temporary) / f"{name}.lsworld"
            world.save(str(bundle))
            round_tripped = World.load(str(bundle)).simulate(
                {"x": initial},
                end=0.1,
                step=0.01,
                inputs={"u": 0.25} if name == "controlled" else None,
            )
            assert len(trajectory.time) == 11, name
            assert trajectory.values["x"][0] == initial, name
            assert all(value == value and abs(value) != float("inf") for value in trajectory.values["x"]), name
            assert trajectory.values == round_tripped.values, name

        scheduled = World(
            ["x"],
            {"rate": 1.0},
            {"x": "rate"},
        )
        intervention = Scenario(
            scheduled,
            {"x": 0.0},
            parameter_schedule=[(0.5, "rate", 3.0)],
        ).simulate(end=1.0, step=1.0)
        assert intervention.time == [0.0, 0.5, 1.0]
        assert abs(intervention.values["x"][-1] - 2.0) < 1e-12

        time = [step * 0.01 for step in range(101)]
        recovered = discover(
            time,
            {"x": [math.exp(2.0 * moment) for moment in time]},
            state=["x"],
            threshold=0.01,
            derivative_method="tvreg",
            tvreg_lambda=0.001,
            tvreg_iterations=150,
        )
        assert "x" in recovered.equations()["x"], recovered.equations()


if __name__ == "__main__":
    main()
