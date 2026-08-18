"""Execute and score the SRBench-style credibility families through the CLI.

This module drives the *real* compiled ``lawsynth`` binary and scores the
SRBench metric set adapted to a dynamics engine:

* **symbolic recovery** — parse the discovered laws from ``explain`` and compare
  the recovered term structure against the known governing terms;
* **trajectory R^2** — simulate the discovered world and measure fit against the
  deterministically generated reference trajectory;
* **complexity** — the AST node count reported by ``discover``;
* **training time** — wall-clock of the ``discover`` invocation;
* **determinism** — run ``discover`` twice and assert byte-identical worlds, a
  metric no stochastic competitor can report.

It never fabricates a recovery result.  A ``capability-boundary`` case (static
algebraic Feynman regression the dynamics CLI does not perform) generates its
deterministic dataset and is reported honestly as a boundary.
"""

from __future__ import annotations

import math
import re
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from _srbench_data import ground_truth_trajectory, write_dataset

# Classification labels shared with the rest of the harness.
PASSED = "passed"
BOUNDARY = "capability-boundary"
REGRESSION = "regression"
FAILED = "failed"

_SUMMARY = re.compile(r"mse=([0-9eE.+-]+),\s*complexity=([0-9]+)")
_LAW = re.compile(r"^\s*d([A-Za-z_][A-Za-z0-9_]*)/dt\s*=\s*(.+?)\s*$")

# Ordering of recovery quality; a case passes when achieved >= expected.
_RECOVERY_RANK = {"none": 0, "missed": 1, "partial": 2, "core": 3, "exact": 4}


# --------------------------------------------------------------------------- #
# Parsing discovered laws into canonical term signatures.
# --------------------------------------------------------------------------- #


def _canonical_term(factors: list[str]) -> str:
    """Canonicalise a product of factors into a stable monomial signature."""
    atoms = [factor.strip() for factor in factors if factor.strip()]
    if not atoms:
        return "1"
    powers: dict[str, int] = {}
    for atom in atoms:
        powers[atom] = powers.get(atom, 0) + 1
    parts = []
    for atom in sorted(powers):
        exponent = powers[atom]
        parts.append(atom if exponent == 1 else f"{atom}^{exponent}")
    return "*".join(parts)


def _split_terms(expression: str) -> list[str]:
    """Split an RHS expression on additive boundaries (`` + ``)."""
    return [piece.strip() for piece in expression.split(" + ") if piece.strip()]


def _term_signature(term: str) -> str | None:
    """Return the canonical signature of a single ``coef * f1 * f2`` term."""
    pieces = [piece.strip() for piece in term.split("*")]
    factors: list[str] = []
    for piece in pieces:
        # Drop the leading numeric coefficient(s); keep symbolic factors.
        try:
            float(piece)
            continue
        except ValueError:
            factors.append(piece)
    if not factors:
        # A pure constant term (e.g. "1.5") contributes the bias signature,
        # unless the whole RHS is a literal zero.
        try:
            if float(term) == 0.0:
                return None
        except ValueError:
            pass
        return "1"
    return _canonical_term(factors)


def parse_laws(explain_text: str) -> dict[str, set[str]]:
    """Parse the ``explain`` output into ``{state: {term_signature, ...}}``."""
    laws: dict[str, set[str]] = {}
    for line in explain_text.splitlines():
        match = _LAW.match(line)
        if match is None:
            continue
        state, expression = match.group(1), match.group(2)
        signatures: set[str] = set()
        for term in _split_terms(expression):
            signature = _term_signature(term)
            if signature is not None:
                signatures.add(signature)
        laws[state] = signatures
    return laws


# --------------------------------------------------------------------------- #
# Symbolic recovery scoring.
# --------------------------------------------------------------------------- #


def _classify_law(recovered: set[str], expected: set[str]) -> str:
    if recovered == expected:
        return "exact"
    if expected and expected.issubset(recovered):
        return "core"
    if recovered & expected:
        return "partial"
    return "missed"


def score_recovery(
    discovered: dict[str, set[str]], expected_laws: list[dict[str, Any]]
) -> dict[str, Any]:
    """Compare discovered term structure against the known governing terms."""
    per_law: list[dict[str, Any]] = []
    worst = "exact"
    exact_count = 0
    for law in expected_laws:
        state = str(law["state"])
        expected = {str(term) for term in law["terms"]}
        recovered = discovered.get(state, set())
        level = _classify_law(recovered, expected)
        if level == "exact":
            exact_count += 1
        if _RECOVERY_RANK[level] < _RECOVERY_RANK[worst]:
            worst = level
        per_law.append(
            {
                "state": state,
                "level": level,
                "expected": sorted(expected),
                "recovered": sorted(recovered),
                "spurious": sorted(recovered - expected),
                "missing": sorted(expected - recovered),
            }
        )
    total = len(expected_laws)
    return {
        "level": worst if expected_laws else "none",
        "laws_exact": exact_count,
        "laws_total": total,
        "law_recovery_rate": (exact_count / total) if total else None,
        "system_recovered": bool(expected_laws) and worst == "exact",
        "per_law": per_law,
    }


# --------------------------------------------------------------------------- #
# Trajectory R^2.
# --------------------------------------------------------------------------- #


def _r2(predicted: list[float], truth: list[float]) -> float | None:
    length = min(len(predicted), len(truth))
    if length < 2:
        return None
    predicted, truth = predicted[:length], truth[:length]
    if any(not math.isfinite(value) for value in predicted):
        return float("-inf")
    mean = sum(truth) / length
    ss_tot = sum((value - mean) ** 2 for value in truth)
    ss_res = sum((p - t) ** 2 for p, t in zip(predicted, truth, strict=True))
    if ss_tot == 0.0:
        return 1.0 if ss_res == 0.0 else float("-inf")
    return 1.0 - ss_res / ss_tot


def _parse_trajectory(text: str) -> dict[str, list[float]]:
    lines = [line for line in text.splitlines() if line.strip()]
    if not lines:
        return {}
    header = lines[0].split(",")
    columns: dict[str, list[float]] = {name: [] for name in header}
    for line in lines[1:]:
        cells = line.split(",")
        if len(cells) != len(header):
            continue
        for name, cell in zip(header, cells, strict=True):
            try:
                columns[name].append(float(cell))
            except ValueError:
                columns[name].append(float("nan"))
    columns.pop("time", None)
    return columns


def trajectory_r2(config: dict[str, Any], trajectory: dict[str, list[float]]) -> float | None:
    """Mean R^2 of the simulated states against the generated reference."""
    if not trajectory:
        return None
    truth = ground_truth_trajectory(config)
    shared = [name for name in trajectory if name in truth]
    scores = [
        value
        for name in shared
        if (value := _r2(trajectory[name], truth[name])) is not None
    ]
    if not scores:
        return None
    if any(math.isinf(value) for value in scores):
        return float("-inf")
    return sum(scores) / len(scores)


# --------------------------------------------------------------------------- #
# CLI invocation.
# --------------------------------------------------------------------------- #


@dataclass(frozen=True)
class DiscoverRun:
    returncode: int
    stdout: str
    stderr: str
    world: Path
    elapsed_ns: int
    mean_squared_error: float | None
    complexity: int | None


def _discover_args(config: dict[str, Any], dataset: Path, world: Path) -> list[str]:
    discovery = config["discovery"]
    states = ",".join(str(name) for name in config["system"]["states"])
    args = ["discover", str(dataset), "--time", "time", "--state", states, "--output", str(world)]
    if "preset" in discovery:
        args.extend(["--preset", str(discovery["preset"])])
    args.extend(["--degree", str(discovery.get("degree", 2))])
    args.extend(["--threshold", str(discovery.get("threshold", 0.05))])
    if discovery.get("trigonometric"):
        args.append("--trigonometric")
    if discovery.get("rational"):
        args.append("--rational")
    return args


def _run(binary: Path, args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run([str(binary), *args], cwd=cwd, text=True, capture_output=True, check=False)


def run_discover(binary: Path, config: dict[str, Any], dataset: Path, world: Path, root: Path) -> DiscoverRun:
    started = time.perf_counter_ns()
    completed = _run(binary, _discover_args(config, dataset, world), root)
    elapsed = time.perf_counter_ns() - started
    mse: float | None = None
    complexity: int | None = None
    match = _SUMMARY.search(completed.stdout)
    if match is not None:
        mse, complexity = float(match.group(1)), int(match.group(2))
    return DiscoverRun(
        returncode=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
        world=world,
        elapsed_ns=elapsed,
        mean_squared_error=mse,
        complexity=complexity,
    )


def _simulation_plan(config: dict[str, Any]) -> dict[str, Any] | None:
    expect = config.get("expect", {})
    if not expect.get("simulate", True):
        return None
    system = config["system"]
    initial = {str(k): float(v) for k, v in system["initial"].items()}
    simulation = config.get("simulation")
    if simulation is not None:
        return {
            "start": float(simulation["start"]),
            "end": float(simulation["end"]),
            "step": float(simulation["step"]),
            "initial": initial,
        }
    step = float(system["step"])
    samples = int(system["samples"])
    return {"start": 0.0, "end": step * (samples - 1), "step": step, "initial": initial}


def run_simulate(binary: Path, plan: dict[str, Any], world: Path, root: Path) -> tuple[int, dict[str, list[float]], str]:
    args = [
        "simulate", str(world),
        "--start", f"{plan['start']:.12g}",
        "--end", f"{plan['end']:.12g}",
        "--step", f"{plan['step']:.12g}",
    ]
    for name, value in plan["initial"].items():
        args.extend(["--initial", f"{name}={value:.12g}"])
    completed = _run(binary, args, root)
    trajectory = _parse_trajectory(completed.stdout) if completed.returncode == 0 else {}
    return completed.returncode, trajectory, completed.stderr


# --------------------------------------------------------------------------- #
# Determinism check.
# --------------------------------------------------------------------------- #


def determinism_check(binary: Path, config: dict[str, Any], dataset: Path, workdir: Path, root: Path) -> bool:
    """Discover twice and assert the produced world bundles are byte-identical."""
    first = workdir / "determinism_a.lsworld"
    second = workdir / "determinism_b.lsworld"
    run_a = run_discover(binary, config, dataset, first, root)
    run_b = run_discover(binary, config, dataset, second, root)
    if run_a.returncode != 0 or run_b.returncode != 0:
        return False
    if not first.is_file() or not second.is_file():
        return False
    return first.read_bytes() == second.read_bytes()


# --------------------------------------------------------------------------- #
# Case orchestration.
# --------------------------------------------------------------------------- #


def repository_root(case_dir: Path) -> Path:
    for parent in (case_dir, *case_dir.parents):
        if (parent / "Cargo.toml").is_file() and (parent / "crates").is_dir():
            return parent
    raise RuntimeError(f"could not find LawSynth repository from {case_dir}")


def read_config(case_dir: Path) -> dict[str, Any]:
    import tomllib

    with (case_dir / "benchmark.toml").open("rb") as handle:
        return tomllib.load(handle)


def _boundary_outcome(config: dict[str, Any], dataset: Path) -> dict[str, Any]:
    return {
        "status": BOUNDARY,
        "passed": False,
        "family": str(config["family"]),
        "reason": str(config["capability"]["reason"]),
        "dataset": str(dataset),
        "determinism": None,
    }


def _score_vector(
    run: DiscoverRun,
    recovery: dict[str, Any] | None,
    r2: float | None,
    sim_failure: float | None,
) -> dict[str, Any]:
    return {
        "fit_train": run.mean_squared_error,
        "trajectory_r2": r2,
        "complexity_nodes": run.complexity,
        "simulation_failure_rate": sim_failure,
        "symbolic_level": recovery["level"] if recovery else "none",
        "law_recovery_rate": recovery["law_recovery_rate"] if recovery else None,
        "system_recovered": recovery["system_recovered"] if recovery else None,
        "training_time_ns": run.elapsed_ns,
    }


def run_srbench_case(
    case_dir: Path,
    workdir: Path,
    binary: Path | None,
    *,
    check_determinism: bool = True,
) -> dict[str, Any]:
    """Generate, execute, score, and classify one SRBench-family case."""
    config = read_config(case_dir)
    root = repository_root(case_dir)
    workdir.mkdir(parents=True, exist_ok=True)
    dataset = write_dataset(config, workdir)

    if config["capability"]["status"] != "supported":
        return _boundary_outcome(config, dataset)

    if binary is None:
        return {"status": FAILED, "passed": False, "reason": "CLI binary unavailable", "determinism": None}

    world = workdir / "world.lsworld"
    run = run_discover(binary, config, dataset, world, root)
    if run.returncode != 0:
        return {
            "status": REGRESSION,
            "passed": False,
            "reason": "discover failed",
            "returncode": run.returncode,
            "stderr": run.stderr or None,
            "determinism": None,
        }

    explain = _run(binary, ["explain", str(world)], root)
    discovered = parse_laws(explain.stdout) if explain.returncode == 0 else {}
    expected_laws = config.get("recovery", {}).get("law", [])
    recovery = score_recovery(discovered, expected_laws) if expected_laws else None

    plan = _simulation_plan(config)
    simulate_rc: int | None = None
    r2: float | None = None
    sim_failure: float | None = None
    simulate_stderr = ""
    if plan is not None:
        simulate_rc, trajectory, simulate_stderr = run_simulate(binary, plan, world, root)
        r2 = trajectory_r2(config, trajectory)
        diverged = simulate_rc != 0 or (r2 is not None and not math.isfinite(r2))
        sim_failure = 1.0 if diverged else 0.0

    determinism: bool | None = None
    if check_determinism:
        determinism = determinism_check(binary, config, dataset, workdir, root)

    expect = config.get("expect", {})
    passed = _passed(run, explain.returncode, simulate_rc, sim_failure, recovery, r2, expect, determinism)
    status = PASSED if passed else REGRESSION
    return {
        "status": status,
        "passed": passed,
        "family": str(config["family"]),
        "returncode": run.returncode,
        "inspect_returncode": explain.returncode,
        "simulate_returncode": simulate_rc,
        "determinism": determinism,
        "recovery": recovery,
        "score_vector": _score_vector(run, recovery, r2, sim_failure),
        "stderr": None if passed else (run.stderr or simulate_stderr or explain.stderr or None),
    }


def _passed(
    run: DiscoverRun,
    explain_rc: int,
    simulate_rc: int | None,
    sim_failure: float | None,
    recovery: dict[str, Any] | None,
    r2: float | None,
    expect: dict[str, Any],
    determinism: bool | None,
) -> bool:
    if run.returncode != 0 or explain_rc != 0:
        return False
    if run.mean_squared_error is None or not math.isfinite(run.mean_squared_error):
        return False
    if simulate_rc is not None and simulate_rc != 0:
        return False
    if sim_failure == 1.0:
        return False
    expected_level = str(expect.get("symbolic_recovery", "none"))
    if expected_level != "none":
        achieved = recovery["level"] if recovery else "none"
        if _RECOVERY_RANK.get(achieved, 0) < _RECOVERY_RANK[expected_level]:
            return False
    r2_min = expect.get("r2_min")
    if r2_min is not None and (r2 is None or r2 < float(r2_min)):
        return False
    if determinism is False:
        return False
    return True
