"""Score native benchmark runs with an architecture-§16 style vector.

The scorer never collapses a candidate to a single magic scalar.  It records
the individual measurements described in the production architecture's
"candidate score vector" and derives an honest classification that respects
each case's declared capability status:

* ``supported`` cases must execute and score; a failure is a *regression*.
* ``capability-boundary`` cases are reported as boundaries, never forced to
  pass and never counted as regressions.
"""

from __future__ import annotations

import math
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from _common import read_config
from _engine import NativeRun, ground_truth_trajectory

# Classification labels.
PASSED = "passed"
BOUNDARY = "capability-boundary"
REGRESSION = "regression"
FAILED = "failed"


@dataclass(frozen=True)
class ScoreVector:
    """Subset of the architecture §16 candidate score vector we can measure.

    Fields the public CLI does not currently expose (holdout fit, unit
    consistency, bootstrap/regime/graph stability) are recorded as ``None``
    rather than fabricated.
    """

    fit_train: float | None
    trajectory_error: float | None
    complexity_nodes: int | None
    simulation_failure_rate: float | None
    fit_holdout: float | None = None
    unit_consistency: float | None = None
    bootstrap_stability: float | None = None

    def as_dict(self) -> dict[str, Any]:
        return asdict(self)


def _finite(value: float | None) -> bool:
    return value is not None and math.isfinite(value)


def _rmse(predicted: list[float], truth: list[float]) -> float | None:
    if not predicted or len(predicted) != len(truth):
        return None
    total = 0.0
    for p, t in zip(predicted, truth, strict=True):
        if not (math.isfinite(p) and math.isfinite(t)):
            return math.inf
        total += (p - t) ** 2
    return math.sqrt(total / len(predicted))


def trajectory_error(case_dir: Path, run: NativeRun) -> float | None:
    """Root-mean-square error of the simulated states against ground truth.

    This is an *informational* recovery measurement.  Chaotic reference cases
    intentionally do not assert small long-horizon error, so classification
    never fails on the magnitude of this value; it only fails on a non-finite
    (diverged/NaN) trajectory for a case that claimed to simulate.
    """
    if run.simulate_returncode != 0 or not run.trajectory_time:
        return None
    truth = ground_truth_trajectory(case_dir, run.trajectory_time)
    if not truth:
        return None
    shared = [name for name in run.trajectory if name in truth]
    if not shared:
        return None
    errors = [
        error
        for name in shared
        if (error := _rmse(run.trajectory[name], truth[name])) is not None
    ]
    if not errors:
        return None
    if any(math.isinf(error) for error in errors):
        return math.inf
    return math.sqrt(sum(error * error for error in errors) / len(errors))


def score_vector(case_dir: Path, run: NativeRun) -> ScoreVector:
    """Assemble the measurable portion of the §16 candidate score vector."""
    failure_rate: float | None = None
    if run.simulate_returncode is not None:
        error = trajectory_error(case_dir, run)
        diverged = run.simulate_returncode != 0 or (error is not None and not math.isfinite(error))
        failure_rate = 1.0 if diverged else 0.0
    else:
        error = None
    return ScoreVector(
        fit_train=run.mean_squared_error,
        trajectory_error=error,
        complexity_nodes=run.complexity,
        simulation_failure_rate=failure_rate,
    )


def _executed_cleanly(run: NativeRun) -> bool:
    if run.returncode != 0:
        return False
    if run.inspect_returncode not in (0, None):
        return False
    if run.simulate_returncode not in (0, None):
        return False
    return True


def classify_native(case_dir: Path, run: NativeRun) -> dict[str, Any]:
    """Classify a supported native-CLI run and attach its score vector."""
    vector = score_vector(case_dir, run)
    clean = _executed_cleanly(run)
    diverged = vector.simulation_failure_rate == 1.0
    fit_ok = run.mean_squared_error is None or _finite(run.mean_squared_error)
    passed = clean and not diverged and fit_ok
    status = PASSED if passed else REGRESSION
    return {
        "status": status,
        "passed": passed,
        "returncode": run.returncode,
        "inspect_returncode": run.inspect_returncode,
        "simulate_returncode": run.simulate_returncode,
        "score_vector": vector.as_dict(),
        "stderr": None if passed else (run.stderr or run.simulate_stderr or None),
    }


def boundary_result(case_dir: Path, reason: str, generated: Path | None) -> dict[str, Any]:
    """Report a declared capability boundary honestly (never a regression)."""
    return {
        "status": BOUNDARY,
        "passed": False,
        "reason": reason,
        "dataset": str(generated) if generated is not None else None,
    }


def declared_status(case_dir: Path) -> str:
    """Return the case's declared capability status from its TOML."""
    config = read_config(case_dir)
    capability = config.get("capability")
    if capability is not None:
        return str(capability.get("status", "unknown"))
    # Contract-style cases carry a top-level ``status`` key.
    return str(config.get("status", "unknown"))
