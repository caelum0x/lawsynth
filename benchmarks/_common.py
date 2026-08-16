"""Shared, dependency-free harness used by the checked-in scientific cases.

The harness deliberately keeps benchmark data and reported results outside the
repository.  A benchmark invocation always receives a work directory, which
makes runs repeatable and prevents generated artefacts from becoming fixtures
that hide a broken generator.
"""

from __future__ import annotations

import csv
import json
import math
import random
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any


class CapabilityBoundary(RuntimeError):
    """Raised when a case needs a public capability LawSynth does not expose."""


def repository_root(case_dir: Path) -> Path:
    """Return the repository root for a directory below ``benchmarks``."""
    for parent in (case_dir, *case_dir.parents):
        if (parent / "Cargo.toml").is_file() and (parent / "crates").is_dir():
            return parent
    raise RuntimeError(f"could not find LawSynth repository from {case_dir}")


def read_config(case_dir: Path) -> dict[str, Any]:
    with (case_dir / "benchmark.toml").open("rb") as handle:
        return tomllib.load(handle)


def deterministic_times(samples: int, step: float) -> list[float]:
    if samples < 3:
        raise ValueError("a discovery benchmark needs at least three samples")
    if not math.isfinite(step) or step <= 0:
        raise ValueError("step must be finite and positive")
    return [index * step for index in range(samples)]


def series(kind: str, times: list[float], parameters: dict[str, float]) -> dict[str, list[float]]:
    """Generate the specified deterministic observation process exactly."""
    if kind == "exponential_decay":
        rate = parameters["rate"]
        return {"x": [math.exp(-rate * time) for time in times]}
    if kind == "harmonic":
        omega = parameters["omega"]
        return {
            "x": [math.cos(omega * time) for time in times],
            "v": [-omega * math.sin(omega * time) for time in times],
        }
    if kind == "logistic_map":
        rate = parameters["rate"]
        value = parameters["initial"]
        values = [value]
        for _ in times[1:]:
            value = rate * value * (1.0 - value)
            values.append(value)
        return {"x": values}
    if kind == "lorenz":
        sigma = parameters["sigma"]
        rho = parameters["rho"]
        beta = parameters["beta"]
        x, y, z = parameters["x0"], parameters["y0"], parameters["z0"]
        step = times[1] - times[0]
        values = {"x": [x], "y": [y], "z": [z]}

        def derivative(state: tuple[float, float, float]) -> tuple[float, float, float]:
            a, b, c = state
            return (sigma * (b - a), a * (rho - c) - b, a * b - beta * c)

        def add(state: tuple[float, float, float], slope: tuple[float, float, float], factor: float) -> tuple[float, float, float]:
            return tuple(value + factor * delta for value, delta in zip(state, slope, strict=True))

        for _ in times[1:]:
            state = (x, y, z)
            k1 = derivative(state)
            k2 = derivative(add(state, k1, step / 2.0))
            k3 = derivative(add(state, k2, step / 2.0))
            k4 = derivative(add(state, k3, step))
            x, y, z = tuple(
                value + step * (a + 2.0 * b + 2.0 * c + d) / 6.0
                for value, a, b, c, d in zip(state, k1, k2, k3, k4, strict=True)
            )
            values["x"].append(x)
            values["y"].append(y)
            values["z"].append(z)
        return values
    if kind == "algebraic_polynomial":
        return {"x": [2.0 * time - 1.0 for time in times], "y": [1.0 + 2.0 * (2.0 * time - 1.0) - 0.5 * (2.0 * time - 1.0) ** 2 for time in times]}
    if kind == "algebraic_polynomial_noisy":
        generator = random.Random(int(parameters["seed"]))
        scale = parameters["noise_scale"]
        x_values = [2.0 * time - 1.0 for time in times]
        return {"x": x_values, "y": [1.0 + 2.0 * x - 0.5 * x * x + generator.gauss(0.0, scale) for x in x_values]}
    if kind == "algebraic_rational":
        return {"x": [0.02 + 1.96 * time for time in times], "y": [(0.02 + 1.96 * time) / (1.0 + 0.5 * (0.02 + 1.96 * time)) for time in times]}
    if kind == "algebraic_transcendental":
        return {"x": [2.0 * math.pi * time for time in times], "y": [math.sin(2.0 * math.pi * time) + 0.25 * math.cos(4.0 * math.pi * time) for time in times]}
    if kind == "dimensional_acceleration":
        acceleration = parameters["acceleration"]
        return {"position": [0.5 * acceleration * time * time for time in times], "velocity": [acceleration * time for time in times]}
    if kind == "delayed_feedback":
        delay = int(parameters["delay_steps"])
        gain = parameters["gain"]
        feedback = parameters["feedback"]
        values = [parameters["initial"] for _ in range(delay)]
        for index in range(delay, len(times)):
            values.append(gain * values[index - 1] + feedback * values[index - delay])
        return {"x": values}
    if kind == "stochastic_ornstein_uhlenbeck":
        generator = random.Random(int(parameters["seed"]))
        theta, sigma, value = parameters["theta"], parameters["sigma"], parameters["initial"]
        step = times[1] - times[0]
        values = [value]
        for _ in times[1:]:
            value += -theta * value * step + sigma * math.sqrt(step) * generator.gauss(0.0, 1.0)
            values.append(value)
        return {"x": values}
    if kind == "hybrid_bounce":
        value, velocity = parameters["initial"], parameters["velocity"]
        lower, upper = parameters["lower"], parameters["upper"]
        step = times[1] - times[0]
        values = [value]
        for _ in times[1:]:
            value += velocity * step
            if value >= upper:
                value, velocity = upper - (value - upper), -abs(velocity)
            elif value <= lower:
                value, velocity = lower + (lower - value), abs(velocity)
            values.append(value)
        return {"x": values}
    raise ValueError(f"unknown deterministic series '{kind}'")


def write_dataset(case_dir: Path, workdir: Path) -> Path:
    """Generate a CSV described by the case's TOML and return its path."""
    config = read_config(case_dir)
    generation = config["generation"]
    times = deterministic_times(int(generation["samples"]), float(generation["step"]))
    values = series(str(generation["kind"]), times, dict(generation.get("parameters", {})))
    output = workdir / "observations.csv"
    workdir.mkdir(parents=True, exist_ok=True)
    with output.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        names = list(values)
        writer.writerow(["time", *names])
        for index, time in enumerate(times):
            writer.writerow([f"{time:.12g}", *(f"{values[name][index]:.12g}" for name in names)])
    return output


def invoke_discovery(case_dir: Path, workdir: Path) -> dict[str, Any]:
    """Run the real LawSynth CLI against a supported continuous case."""
    config = read_config(case_dir)
    capability = config["capability"]
    if capability["status"] != "supported":
        raise CapabilityBoundary(str(capability["reason"]))
    dataset = write_dataset(case_dir, workdir)
    output = workdir / "world.lsworld"
    command = [
        "cargo", "run", "--quiet", "-p", "lawsynth-cli", "--", "discover", str(dataset),
        "--time", "time", "--state", ",".join(config["discovery"]["states"]),
        "--output", str(output), "--degree", str(config["discovery"].get("degree", 2)),
        "--threshold", str(config["discovery"].get("threshold", 0.05)),
    ]
    if config["discovery"].get("trigonometric"):
        command.append("--trigonometric")
    result = subprocess.run(command, cwd=repository_root(case_dir), text=True, capture_output=True, check=False)
    payload: dict[str, Any] = {
        "command": command,
        "returncode": result.returncode,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "world": str(output),
    }
    if result.returncode == 0:
        inspection = subprocess.run(
            ["cargo", "run", "--quiet", "-p", "lawsynth-cli", "--", "inspect", str(output)],
            cwd=repository_root(case_dir), text=True, capture_output=True, check=False,
        )
        payload.update({"inspect_returncode": inspection.returncode, "inspection": inspection.stdout, "inspect_stderr": inspection.stderr})
        simulation = config.get("simulation")
        if simulation is not None and inspection.returncode == 0:
            initial = simulation["initial"]
            simulate_command = [
                "cargo", "run", "--quiet", "-p", "lawsynth-cli", "--", "simulate", str(output),
                "--start", str(simulation["start"]), "--end", str(simulation["end"]), "--step", str(simulation["step"]),
            ]
            for name, value in initial.items():
                simulate_command.extend(["--initial", f"{name}={value}"])
            simulated = subprocess.run(
                simulate_command, cwd=repository_root(case_dir), text=True, capture_output=True, check=False,
            )
            payload.update({
                "simulate_returncode": simulated.returncode,
                "trajectory": simulated.stdout,
                "simulate_stderr": simulated.stderr,
            })
    return payload


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def score_result(case_dir: Path, result: dict[str, Any]) -> dict[str, Any]:
    """Score only observable native execution, never reconstructed predictions."""
    capability = read_config(case_dir)["capability"]
    if capability["status"] != "supported":
        return {"status": "capability-boundary", "passed": False, "reason": capability["reason"]}
    passed = result.get("returncode") == 0 and result.get("inspect_returncode") == 0
    if "simulate_returncode" in result:
        passed = passed and result["simulate_returncode"] == 0
    return {
        "status": "passed" if passed else "failed",
        "passed": passed,
        "returncode": result.get("returncode"),
        "inspect_returncode": result.get("inspect_returncode"),
        "simulate_returncode": result.get("simulate_returncode"),
    }


def script_main(case_dir: Path, mode: str) -> int:
    """Small common command-line front end used by generated artifact scripts."""
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("--workdir", type=Path, default=case_dir / ".benchmark-run")
    arguments = parser.parse_args()
    if mode == "generate":
        print(write_dataset(case_dir, arguments.workdir))
        return 0
    try:
        result = invoke_discovery(case_dir, arguments.workdir)
    except CapabilityBoundary as error:
        result = {"status": "capability-boundary", "reason": str(error)}
    if mode == "run":
        write_json(arguments.workdir / "result.json", result)
        print(json.dumps(result, sort_keys=True))
        return 0 if result.get("returncode", 1) == 0 else 2
    score = score_result(case_dir, result)
    write_json(arguments.workdir / "score.json", score)
    print(json.dumps(score, sort_keys=True))
    return 0 if score["passed"] else 2


if __name__ == "__main__":
    raise SystemExit("import this module from a benchmark artifact")
