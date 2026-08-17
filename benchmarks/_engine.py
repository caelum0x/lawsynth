"""Drive the real LawSynth CLI binary for benchmark execution.

Unlike :mod:`_common`, which shells out through ``cargo run``, this module
invokes the compiled ``lawsynth`` binary directly.  Building once and reusing
the binary keeps every benchmark invocation fast, offline, and deterministic.

The engine only *executes* the native product.  It never fabricates a
candidate, a bundle, or a trajectory: a case either runs through the public
CLI or is honestly reported at its capability boundary by the caller.
"""

from __future__ import annotations

import os
import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from _common import read_config, repository_root, series, write_dataset

_SUMMARY_PATTERN = re.compile(r"mse=([0-9eE.+-]+),\s*complexity=([0-9]+)")


class EngineUnavailable(RuntimeError):
    """Raised when the compiled CLI binary cannot be located or built."""


@dataclass(frozen=True)
class NativeRun:
    """Structured record of one native CLI benchmark invocation."""

    returncode: int
    inspect_returncode: int | None = None
    simulate_returncode: int | None = None
    mean_squared_error: float | None = None
    complexity: int | None = None
    trajectory: dict[str, list[float]] = field(default_factory=dict)
    trajectory_time: list[float] = field(default_factory=list)
    stdout: str = ""
    stderr: str = ""
    inspect_stdout: str = ""
    inspect_stderr: str = ""
    simulate_stderr: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "returncode": self.returncode,
            "inspect_returncode": self.inspect_returncode,
            "simulate_returncode": self.simulate_returncode,
            "mean_squared_error": self.mean_squared_error,
            "complexity": self.complexity,
            "trajectory_samples": len(self.trajectory_time),
        }


def binary_candidates(root: Path) -> list[Path]:
    """Return the search order used to locate the compiled CLI binary."""
    override = os.environ.get("LAWSYNTH_BIN")
    ordered: list[Path] = []
    if override:
        ordered.append(Path(override))
    ordered.extend(
        [
            root / "target" / "debug" / "lawsynth",
            root / "target" / "release" / "lawsynth",
        ]
    )
    return ordered


def locate_binary(root: Path) -> Path | None:
    """Return the first existing CLI binary, or ``None`` when absent."""
    for candidate in binary_candidates(root):
        if candidate.is_file() and os.access(candidate, os.X_OK):
            return candidate
    return None


def build_binary(root: Path) -> Path:
    """Build ``lawsynth-cli`` offline and return the produced binary path."""
    completed = subprocess.run(
        ["cargo", "build", "--offline", "-p", "lawsynth-cli"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise EngineUnavailable(
            "failed to build lawsynth-cli offline:\n" + completed.stderr.strip()
        )
    located = locate_binary(root)
    if located is None:
        raise EngineUnavailable("cargo build succeeded but no binary was produced")
    return located


def ensure_binary(root: Path, *, allow_build: bool = False) -> Path:
    """Return an executable CLI binary, optionally building it once."""
    located = locate_binary(root)
    if located is not None:
        return located
    if allow_build:
        return build_binary(root)
    raise EngineUnavailable(
        "no compiled lawsynth binary found; run `cargo build -p lawsynth-cli` "
        "or set LAWSYNTH_BIN to the binary path"
    )


def _run(binary: Path, args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(binary), *args], cwd=cwd, text=True, capture_output=True, check=False
    )


def _parse_summary(text: str) -> tuple[float | None, int | None]:
    match = _SUMMARY_PATTERN.search(text)
    if match is None:
        return None, None
    return float(match.group(1)), int(match.group(2))


def _parse_trajectory(text: str) -> tuple[list[float], dict[str, list[float]]]:
    lines = [line for line in text.splitlines() if line.strip()]
    if not lines:
        return [], {}
    header = lines[0].split(",")
    columns: dict[str, list[float]] = {name: [] for name in header}
    for line in lines[1:]:
        cells = line.split(",")
        if len(cells) != len(header):
            continue
        for name, cell in zip(header, cells, strict=True):
            columns[name].append(float(cell))
    times = columns.pop("time", [])
    return times, columns


def _discovery_args(dataset: Path, output: Path, discovery: dict[str, Any]) -> list[str]:
    args = [
        "discover",
        str(dataset),
        "--time",
        "time",
        "--state",
        ",".join(discovery["states"]),
        "--output",
        str(output),
        "--degree",
        str(discovery.get("degree", 2)),
        "--threshold",
        str(discovery.get("threshold", 0.05)),
    ]
    if discovery.get("trigonometric"):
        args.append("--trigonometric")
    if discovery.get("rational"):
        args.append("--rational")
    return args


def run_native_case(case_dir: Path, workdir: Path, binary: Path) -> NativeRun:
    """Generate the dataset and run discover/inspect/simulate via the binary."""
    config = read_config(case_dir)
    root = repository_root(case_dir)
    dataset = write_dataset(case_dir, workdir)
    world = workdir / "world.lsworld"

    discover = _run(
        binary, _discovery_args(dataset, world, config["discovery"]), root
    )
    mse, complexity = _parse_summary(discover.stdout)
    if discover.returncode != 0:
        return NativeRun(
            returncode=discover.returncode,
            stdout=discover.stdout,
            stderr=discover.stderr,
        )

    inspect = _run(binary, ["inspect", str(world)], root)

    simulate_rc: int | None = None
    simulate_stderr = ""
    times: list[float] = []
    trajectory: dict[str, list[float]] = {}
    simulation = config.get("simulation")
    if simulation is not None and inspect.returncode == 0:
        args = [
            "simulate",
            str(world),
            "--start",
            str(simulation["start"]),
            "--end",
            str(simulation["end"]),
            "--step",
            str(simulation["step"]),
        ]
        for name, value in simulation["initial"].items():
            args.extend(["--initial", f"{name}={value}"])
        simulated = _run(binary, args, root)
        simulate_rc = simulated.returncode
        simulate_stderr = simulated.stderr
        if simulated.returncode == 0:
            times, trajectory = _parse_trajectory(simulated.stdout)

    return NativeRun(
        returncode=discover.returncode,
        inspect_returncode=inspect.returncode,
        simulate_returncode=simulate_rc,
        mean_squared_error=mse,
        complexity=complexity,
        trajectory=trajectory,
        trajectory_time=times,
        stdout=discover.stdout,
        stderr=discover.stderr,
        inspect_stdout=inspect.stdout,
        inspect_stderr=inspect.stderr,
        simulate_stderr=simulate_stderr,
    )


def ground_truth_trajectory(
    case_dir: Path, times: list[float]
) -> dict[str, list[float]]:
    """Evaluate the deterministic generator on the simulation time grid."""
    if not times:
        return {}
    generation = read_config(case_dir)["generation"]
    return series(str(generation["kind"]), list(times), dict(generation.get("parameters", {})))
