"""Shared executable support for LawSynth scientific reference cases.

The functions in this module generate deterministic observations from published
ordinary differential equations and exercise the *native* Rust CLI.  They are
not a second implementation of discovery: numerical integration is used only
to make fixtures portable, while every asserted discovery and simulation result
comes from ``lawsynth`` itself.
"""

from __future__ import annotations

import csv
import json
import math
import subprocess
import tempfile
import tomllib
from pathlib import Path
from typing import Any, Callable


ROOT = Path(__file__).resolve().parents[2]


def load_case(directory: Path) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    """Load and cross-check the three declarative artifacts for a case."""
    with (directory / "case.toml").open("rb") as handle:
        case = tomllib.load(handle)
    input_data = json.loads((directory / "input.json").read_text(encoding="utf-8"))
    expected = json.loads((directory / "expected.json").read_text(encoding="utf-8"))
    identifier = directory.name
    assert case["case"]["id"] == identifier == input_data["case_id"] == expected["case_id"]
    assert case["case"]["kind"] == input_data["kind"] == expected["kind"]
    return case, input_data, expected


def native_cli(*arguments: str) -> subprocess.CompletedProcess[str]:
    """Run the public command-line interface without shell interpolation."""
    return subprocess.run(
        ["cargo", "run", "--quiet", "-p", "lawsynth-cli", "--bin", "lawsynth", "--", *arguments],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def rk4(
    derivative: Callable[[float, list[float]], list[float]], initial: list[float], stop: float, step: float
) -> tuple[list[float], list[list[float]]]:
    """Create an independent, deterministic observation fixture with RK4."""
    time = [0.0]
    rows = [initial[:]]
    while time[-1] + 1e-14 < stop:
        current_time = time[-1]
        values = rows[-1]
        h = min(step, stop - current_time)
        k1 = derivative(current_time, values)
        k2 = derivative(current_time + h / 2.0, [x + h * y / 2.0 for x, y in zip(values, k1, strict=True)])
        k3 = derivative(current_time + h / 2.0, [x + h * y / 2.0 for x, y in zip(values, k2, strict=True)])
        k4 = derivative(current_time + h, [x + h * y for x, y in zip(values, k3, strict=True)])
        rows.append([x + h * (a + 2.0 * b + 2.0 * c + d) / 6.0 for x, a, b, c, d in zip(values, k1, k2, k3, k4, strict=True)])
        time.append(current_time + h)
    return time, rows


def observations(spec: dict[str, Any]) -> tuple[list[str], list[float], list[list[float]]]:
    """Return named, finite observations for the fixture system in ``spec``."""
    system = spec["system"]
    parameters = spec["parameters"]
    initial = [float(value) for value in spec["initial"]]
    stop = float(spec["stop"])
    step = float(spec["integration_step"])
    if system == "linear_growth":
        rate = float(parameters["rate"])
        time = [float(value) for value in spec.get("sample_times", [index * step for index in range(round(stop / step) + 1)])]
        return ["x"], time, [[initial[0] * math.exp(rate * instant)] for instant in time]
    if system == "lotka_volterra":
        alpha, beta, delta, gamma = (float(parameters[name]) for name in ("alpha", "beta", "delta", "gamma"))
        time, rows = rk4(lambda _t, x: [alpha * x[0] - beta * x[0] * x[1], delta * x[0] * x[1] - gamma * x[1]], initial, stop, step)
        return ["prey", "predator"], time, rows
    if system == "lorenz":
        sigma, rho, beta = (float(parameters[name]) for name in ("sigma", "rho", "beta"))
        time, rows = rk4(lambda _t, x: [sigma * (x[1] - x[0]), x[0] * (rho - x[2]) - x[1], x[0] * x[1] - beta * x[2]], initial, stop, step)
        return ["x", "y", "z"], time, rows
    if system == "pendulum":
        damping, gravity_over_length = (float(parameters[name]) for name in ("damping", "gravity_over_length"))
        time, rows = rk4(lambda _t, x: [x[1], -damping * x[1] - gravity_over_length * math.sin(x[0])], initial, stop, step)
        return ["theta", "omega"], time, rows
    if system == "sir":
        beta, gamma = (float(parameters[name]) for name in ("beta", "gamma"))
        time, rows = rk4(lambda _t, x: [-beta * x[0] * x[1], beta * x[0] * x[1] - gamma * x[1], gamma * x[1]], initial, stop, step)
        return ["susceptible", "infected", "recovered"], time, rows
    raise ValueError(f"unknown scientific fixture system {system!r}")


def write_csv(path: Path, names: list[str], time: list[float], rows: list[list[float]], nan_indices: dict[str, list[int]] | None = None) -> None:
    """Write a CSV accepted by the public CLI, optionally injecting missing data."""
    missing = {name: set(indices) for name, indices in (nan_indices or {}).items()}
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["time", *names])
        for index, (instant, values) in enumerate(zip(time, rows, strict=True)):
            output = [format(instant, ".17g")]
            output.extend("nan" if index in missing.get(name, set()) else format(value, ".17g") for name, value in zip(names, values, strict=True))
            writer.writerow(output)


def parse_trajectory(text: str, states: list[str]) -> list[dict[str, float]]:
    """Validate that a native simulation emits a rectangular, finite CSV."""
    rows = list(csv.DictReader(text.splitlines()))
    assert rows, "native simulation emitted no rows"
    assert list(rows[0]) == ["time", *sorted(states)], f"unexpected simulation columns: {list(rows[0])}"
    parsed = [{name: float(row[name]) for name in ["time", *states]} for row in rows]
    assert all(math.isfinite(value) for row in parsed for value in row.values()), "native simulation emitted non-finite values"
    return parsed


def assert_discovery_case(directory: Path) -> None:
    """Execute discovery, inspect the produced world, and simulate it natively."""
    _case, input_data, expected = load_case(directory)
    names, time, rows = observations(input_data["fixture"])
    run = input_data["run"]
    with tempfile.TemporaryDirectory(prefix=f"lawsynth-scientific-{directory.name}-") as temporary:
        temporary_path = Path(temporary)
        source = temporary_path / "observations.csv"
        world = temporary_path / "candidate.lsworld"
        write_csv(source, names, time, rows, input_data.get("nan_indices"))
        command = ["discover", str(source), "--time", "time", "--state", ",".join(run["states"]), "--output", str(world)]
        for name, value in run.get("options", {}).items():
            command.extend([name, str(value)])
        for flag in run.get("flags", []):
            command.append(flag)
        result = native_cli(*command)
        assert result.returncode == 0, result.stderr
        assert "discovered world: mse=" in result.stdout, result.stdout
        assert world.is_file() and world.stat().st_size > 0, "native discovery created no world archive"
        inspected = native_cli("inspect", str(world))
        assert inspected.returncode == 0, inspected.stderr
        assert expected["inspect_contains"] in inspected.stdout, inspected.stdout
        initial = run["simulate_initial"]
        simulation = native_cli("simulate", str(world), *sum((["--initial", f"{name}={value}"] for name, value in initial.items()), []), "--start", "0", "--end", str(run["simulate_end"]), "--step", str(run["simulate_step"]))
        assert simulation.returncode == 0, simulation.stderr
        trajectory = parse_trajectory(simulation.stdout, run["states"])
        assert len(trajectory) >= expected["minimum_samples"]
        print(f"{directory.name}: native discovery and simulation completed ({len(trajectory)} samples)")


def assert_missing_data_boundary(directory: Path) -> None:
    """Prove that NaN observations are rejected instead of silently imputed."""
    _case, input_data, expected = load_case(directory)
    names, time, rows = observations(input_data["fixture"])
    with tempfile.TemporaryDirectory(prefix="lawsynth-scientific-missing-") as temporary:
        source = Path(temporary) / "missing.csv"
        write_csv(source, names, time, rows, input_data["nan_indices"])
        result = native_cli("discover", str(source), "--time", "time", "--state", ",".join(input_data["run"]["states"]), "--output", str(Path(temporary) / "unused.lsworld"))
        assert result.returncode != 0, "missing numeric observations were silently accepted"
        assert expected["error_contains"] in result.stderr + result.stdout
    print(f"{directory.name}: documented native missing-data boundary")


def assert_regime_boundary(directory: Path) -> None:
    """Run a switched series and show the current output remains one continuous world."""
    _case, input_data, expected = load_case(directory)
    fixture = input_data["fixture"]
    names, time, rows = observations(fixture)
    switch = int(input_data["switch_index"])
    rate_after = float(input_data["rate_after"])
    for index in range(switch, len(rows)):
        rows[index][0] = rows[switch][0] * math.exp(rate_after * (time[index] - time[switch]))
    with tempfile.TemporaryDirectory(prefix="lawsynth-scientific-regime-") as temporary:
        temporary_path = Path(temporary)
        source, world = temporary_path / "switched.csv", temporary_path / "candidate.lsworld"
        write_csv(source, names, time, rows)
        result = native_cli("discover", str(source), "--time", "time", "--state", "x", "--output", str(world), "--degree", "1")
        assert result.returncode == 0, result.stderr
        inspected = native_cli("inspect", str(world))
        assert inspected.returncode == 0 and expected["inspect_contains"] in inspected.stdout
    print(f"{directory.name}: regime changes are not represented as recovered regimes")


def assert_uncertainty_boundary(directory: Path) -> None:
    """Exercise bootstrap discovery but avoid claiming unavailable interval output."""
    _case, input_data, expected = load_case(directory)
    names, time, rows = observations(input_data["fixture"])
    with tempfile.TemporaryDirectory(prefix="lawsynth-scientific-uncertainty-") as temporary:
        temporary_path = Path(temporary)
        source, world = temporary_path / "observations.csv", temporary_path / "candidate.lsworld"
        write_csv(source, names, time, rows)
        result = native_cli("discover", str(source), "--time", "time", "--state", "x", "--output", str(world), "--degree", "1", "--bootstrap", str(input_data["bootstrap_replicates"]))
        assert result.returncode == 0, result.stderr
        assert expected["stdout_contains"] in result.stdout
        inspected = native_cli("inspect", str(world))
        assert inspected.returncode == 0 and expected["inspect_contains"] in inspected.stdout
    print(f"{directory.name}: bootstrap executes; no trajectory coverage claim is made")


def assert_noise_case(directory: Path) -> None:
    """Apply deterministic adversarial-but-finite noise and require a native model."""
    _case, input_data, expected = load_case(directory)
    names, time, rows = observations(input_data["fixture"])
    amplitude = float(input_data["noise_amplitude"])
    for index, values in enumerate(rows):
        values[0] += amplitude * (1.0 if index % 2 else -1.0)
    with tempfile.TemporaryDirectory(prefix="lawsynth-scientific-noise-") as temporary:
        temporary_path = Path(temporary)
        source, world = temporary_path / "noisy.csv", temporary_path / "candidate.lsworld"
        write_csv(source, names, time, rows)
        result = native_cli("discover", str(source), "--time", "time", "--state", "x", "--output", str(world), "--degree", "1", "--savgol-window", "5")
        assert result.returncode == 0, result.stderr
        inspected = native_cli("inspect", str(world))
        assert inspected.returncode == 0 and expected["inspect_contains"] in inspected.stdout
    print(f"{directory.name}: finite noisy observations produce a native candidate, without recovery claim")


def assert_unit_boundary(directory: Path) -> None:
    """Exercise discovery while documenting that CSV input has no unit field."""
    _case, input_data, expected = load_case(directory)
    names, time, rows = observations(input_data["fixture"])
    with tempfile.TemporaryDirectory(prefix="lawsynth-scientific-units-") as temporary:
        temporary_path = Path(temporary)
        source, world = temporary_path / "dimensioned-in-name-only.csv", temporary_path / "candidate.lsworld"
        write_csv(source, names, time, rows)
        result = native_cli("discover", str(source), "--time", "time", "--state", "x", "--output", str(world), "--degree", "1")
        assert result.returncode == 0, result.stderr
        inspected = native_cli("inspect", str(world))
        assert inspected.returncode == 0 and expected["inspect_contains"] in inspected.stdout
    print(f"{directory.name}: unit-aware CSV discovery is explicitly outside this CLI boundary")
