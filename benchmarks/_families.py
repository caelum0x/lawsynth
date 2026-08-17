"""Execute the causal and regime benchmark families through the real CLI.

The ``discover`` command now exposes structural-analysis flags that emit real,
deterministic signals which were previously reported only as SDK-surface
capability boundaries:

* ``--causal`` emits a *dependency hypothesis* graph (an edge count over the
  discovered state variables);
* ``--regimes`` emits a *regime segmentation* (a segment count over the
  observation window);
* ``--pareto`` reports the size of the retained candidate frontier;
* ``--refine`` reports a local-refinement improvement when one is available.

This module drives those flags end to end against generated, deterministic
family datasets, parses each signal, and scores it against a ground-truth
derived minimum declared in the case's ``[expect]`` table.  The score is a
real (partial) execution signal, never a fabricated recovery result: the CLI
genuinely runs and genuinely produces the structural summary that is scored.
"""

from __future__ import annotations

import csv
import json
import re
import subprocess
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from _capability_contract import generate_series
from _common import read_config, repository_root

# Classification labels shared with :mod:`_scoring`.
PASSED = "passed"
REGRESSION = "regression"

_SUMMARY = re.compile(r"mse=([0-9eE.+-]+),\s*complexity=([0-9]+)")
_PARETO = re.compile(r"pareto frontier:\s*([0-9]+)\s*candidate")
_DEPENDENCY = re.compile(r"dependency hypothesis:\s*([0-9]+)\s*edge")
_REGIMES = re.compile(r"regimes:\s*([0-9]+)\s*segment")
_REFINEMENT = re.compile(r"refinement:\s*improvement=([0-9eE.+-]+),\s*iterations=([0-9]+)")

# The public discovery flag that each executed family exercises.
_FAMILY_SIGNAL = {
    "causal": ("--causal", "dependency_edges"),
    "regime": ("--regimes", "regime_segments"),
}


@dataclass(frozen=True)
class FamilyRun:
    """Structured record of one executed causal/regime discovery run."""

    discover_returncode: int
    inspect_returncode: int | None = None
    mean_squared_error: float | None = None
    complexity: int | None = None
    frontier_size: int | None = None
    dependency_edges: int | None = None
    regime_segments: int | None = None
    refinement_improvement: float | None = None
    refinement_iterations: int | None = None
    stdout: str = ""
    stderr: str = ""
    inspect_stderr: str = ""
    states: tuple[str, ...] = field(default_factory=tuple)

    def signal(self, name: str) -> int | None:
        return {
            "dependency_edges": self.dependency_edges,
            "regime_segments": self.regime_segments,
        }[name]


def is_family_executed(config: dict[str, Any]) -> bool:
    """True when a contract-family case has been promoted to real execution."""
    return (
        "family" in config
        and config.get("status") == "executed"
        and config.get("family") in _FAMILY_SIGNAL
    )


def write_family_dataset(config: dict[str, Any], workdir: Path) -> Path:
    """Generate the deterministic family dataset as a CSV the CLI can read."""
    family = str(config["family"])
    name = str(config["name"])
    states = [str(state) for state in config["execution"]["states"]]
    data = generate_series(family, name, int(config["samples"]))
    columns = ["time", *states]
    workdir.mkdir(parents=True, exist_ok=True)
    output = workdir / "observations.csv"
    with output.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(columns)
        for row in data["rows"]:
            writer.writerow([f"{float(row[column]):.12g}" for column in columns])
    return output


def _discover_args(dataset: Path, world: Path, execution: dict[str, Any], flag: str) -> list[str]:
    states = ",".join(str(state) for state in execution["states"])
    args = [
        "discover",
        str(dataset),
        "--time",
        "time",
        "--state",
        states,
        "--output",
        str(world),
        "--degree",
        str(execution.get("degree", 2)),
        "--threshold",
        str(execution.get("threshold", 0.05)),
        flag,
        "--pareto",
        "--refine",
    ]
    return args


def _parse(text: str, run_kwargs: dict[str, Any]) -> None:
    summary = _SUMMARY.search(text)
    if summary is not None:
        run_kwargs["mean_squared_error"] = float(summary.group(1))
        run_kwargs["complexity"] = int(summary.group(2))
    pareto = _PARETO.search(text)
    if pareto is not None:
        run_kwargs["frontier_size"] = int(pareto.group(1))
    dependency = _DEPENDENCY.search(text)
    if dependency is not None:
        run_kwargs["dependency_edges"] = int(dependency.group(1))
    regimes = _REGIMES.search(text)
    if regimes is not None:
        run_kwargs["regime_segments"] = int(regimes.group(1))
    refinement = _REFINEMENT.search(text)
    if refinement is not None:
        run_kwargs["refinement_improvement"] = float(refinement.group(1))
        run_kwargs["refinement_iterations"] = int(refinement.group(2))


def run_family(config: dict[str, Any], workdir: Path, binary: Path, root: Path) -> FamilyRun:
    """Run ``discover`` with the family flag and parse every emitted signal."""
    execution = config["execution"]
    flag, _ = _FAMILY_SIGNAL[str(config["family"])]
    dataset = write_family_dataset(config, workdir)
    world = workdir / "world.lsworld"
    discover = subprocess.run(
        [str(binary), *_discover_args(dataset, world, execution, flag)],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    run_kwargs: dict[str, Any] = {
        "discover_returncode": discover.returncode,
        "stdout": discover.stdout,
        "stderr": discover.stderr,
        "states": tuple(str(state) for state in execution["states"]),
    }
    _parse(discover.stdout, run_kwargs)
    if discover.returncode == 0:
        inspect = subprocess.run(
            [str(binary), "inspect", str(world)],
            cwd=root,
            text=True,
            capture_output=True,
            check=False,
        )
        run_kwargs["inspect_returncode"] = inspect.returncode
        run_kwargs["inspect_stderr"] = inspect.stderr
    return FamilyRun(**run_kwargs)


def score_family(config: dict[str, Any], run: FamilyRun) -> dict[str, Any]:
    """Score a family run against its declared, ground-truth-derived minimum."""
    _, signal_name = _FAMILY_SIGNAL[str(config["family"])]
    expect = config.get("expect", {})
    minimum = int(expect.get("minimum", 1))
    value = run.signal(signal_name)
    executed_cleanly = run.discover_returncode == 0 and run.inspect_returncode == 0
    signal_ok = value is not None and value >= minimum
    passed = executed_cleanly and signal_ok
    status = PASSED if passed else REGRESSION
    score_vector = {
        "fit_train": run.mean_squared_error,
        "trajectory_error": None,
        "complexity_nodes": run.complexity,
        "simulation_failure_rate": None,
        "frontier_size": run.frontier_size,
        "dependency_edges": run.dependency_edges,
        "regime_segments": run.regime_segments,
        "refinement_improvement": run.refinement_improvement,
        "refinement_iterations": run.refinement_iterations,
    }
    return {
        "status": status,
        "passed": passed,
        "signal": signal_name,
        "signal_value": value,
        "expected_minimum": minimum,
        "discover_returncode": run.discover_returncode,
        "inspect_returncode": run.inspect_returncode,
        "score_vector": score_vector,
        "stderr": None if passed else (run.stderr or run.inspect_stderr or None),
    }


def run_family_case(case_dir: Path, workdir: Path, binary: Path) -> dict[str, Any]:
    """Generate, execute, and score one promoted causal/regime family case."""
    config = read_config(case_dir)
    root = repository_root(case_dir)
    run = run_family(config, workdir, binary, root)
    return score_family(config, run)


def main(directory: Path, mode: str) -> int:
    """Per-case command-line front end used by the promoted family scripts."""
    from _engine import EngineUnavailable, ensure_binary

    config = read_config(directory)
    workdir = directory / ".benchmark-run"
    if mode == "generate":
        print(write_family_dataset(config, workdir))
        return 0
    root = repository_root(directory)
    try:
        binary = ensure_binary(root, allow_build=True)
    except EngineUnavailable as error:
        print(json.dumps({"status": "skipped", "reason": str(error)}, sort_keys=True))
        return 0
    result = run_family_case(directory, workdir, binary)
    print(json.dumps(result, sort_keys=True, default=str))
    if mode in {"run", "score"}:
        return 0 if result["passed"] else 2
    raise ValueError(f"unknown mode: {mode}")


if __name__ == "__main__":
    import sys

    raise SystemExit(main(Path(sys.argv[1]), sys.argv[2]))
