"""Benchmark suite orchestration: generate, execute, score, and report.

The runner walks every ``benchmark.toml`` under ``benchmarks/`` and dispatches
each case to the correct execution path:

* native supported cases run through the compiled ``lawsynth`` CLI binary;
* the public Python SDK boundary case runs through the installed package;
* dynamics/equation ``capability-boundary`` cases generate their deterministic
  dataset and are reported honestly as boundaries;
* causal/regime/uncertainty contract cases assert an unavailable public API.

Only real ``supported`` regressions make the suite fail.
"""

from __future__ import annotations

import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from _common import read_config, repository_root, write_dataset
from _engine import EngineUnavailable, ensure_binary, run_native_case
from _families import is_family_executed, run_family_case
from _scoring import (
    BOUNDARY,
    FAILED,
    PASSED,
    REGRESSION,
    boundary_result,
    classify_native,
    declared_status,
)

# Statuses that make the whole suite fail with a non-zero exit code.
FAILING_STATUSES = frozenset({REGRESSION, FAILED})


@dataclass(frozen=True)
class CaseOutcome:
    case_id: str
    family: str
    declared: str
    result: dict[str, Any]

    @property
    def status(self) -> str:
        return str(self.result["status"])


def discover_cases(benchmarks_dir: Path) -> list[Path]:
    """Return every benchmark case directory, sorted for determinism."""
    return sorted(
        (toml.parent for toml in benchmarks_dir.rglob("benchmark.toml")),
        key=lambda path: str(path),
    )


def case_id(benchmarks_dir: Path, case_dir: Path) -> str:
    return case_dir.relative_to(benchmarks_dir).as_posix()


def _is_contract_case(config: dict[str, Any]) -> bool:
    return "family" in config and config.get("status") == "capability_boundary"


def _workflow(config: dict[str, Any]) -> str:
    workload = config.get("workload")
    return str(workload["workflow"]) if workload else "native-cli"


def _run_python_sdk(case_dir: Path) -> dict[str, Any]:
    from _performance import run_workload, score_workload

    result = run_workload(case_dir, case_dir / ".benchmark-run")
    scored = score_workload(case_dir, result)
    status = PASSED if scored["passed"] else REGRESSION
    return {
        "status": status,
        "passed": scored["passed"],
        "operation": result.get("operation"),
        "elapsed_ns": result.get("elapsed_ns"),
    }


def _run_contract(case_dir: Path) -> dict[str, Any]:
    from _capability_contract import execute

    outcome = execute(case_dir)
    return {
        "status": BOUNDARY,
        "passed": False,
        "reason": outcome["error"],
        "feature": outcome["feature"],
        "rows": outcome["rows"],
        "missing_public_operations": outcome["missing_public_operations"],
    }


def run_case(
    benchmarks_dir: Path,
    case_dir: Path,
    workdir: Path,
    binary: Path | None,
) -> CaseOutcome:
    """Execute a single benchmark case and return its classified outcome."""
    config = read_config(case_dir)
    identifier = case_id(benchmarks_dir, case_dir)
    family = identifier.split("/", 1)[0]

    if is_family_executed(config):
        if binary is None:
            return CaseOutcome(
                identifier,
                family,
                "executed",
                {"status": FAILED, "passed": False, "reason": "CLI binary unavailable"},
            )
        return CaseOutcome(identifier, family, "executed", run_family_case(case_dir, workdir, binary))

    if _is_contract_case(config):
        return CaseOutcome(identifier, family, "capability_boundary", _run_contract(case_dir))

    declared = declared_status(case_dir)

    if declared != "supported":
        reason = str(config["capability"]["reason"])
        dataset = write_dataset(case_dir, workdir)  # prove deterministic generation
        return CaseOutcome(identifier, family, declared, boundary_result(case_dir, reason, dataset))

    workflow = _workflow(config)
    if workflow == "python-sdk":
        return CaseOutcome(identifier, family, declared, _run_python_sdk(case_dir))

    if binary is None:
        return CaseOutcome(
            identifier,
            family,
            declared,
            {"status": FAILED, "passed": False, "reason": "CLI binary unavailable"},
        )
    run = run_native_case(case_dir, workdir, binary)
    return CaseOutcome(identifier, family, declared, classify_native(case_dir, run))


def run_suite(
    benchmarks_dir: Path,
    workroot: Path,
    *,
    family_filter: str | None = None,
    allow_build: bool = False,
) -> dict[str, Any]:
    """Run every (optionally filtered) case and build a deterministic summary."""
    root = repository_root(benchmarks_dir)
    cases = discover_cases(benchmarks_dir)
    if family_filter:
        cases = [c for c in cases if case_id(benchmarks_dir, c).startswith(f"{family_filter}/")]

    binary: Path | None
    binary_note: str
    try:
        binary = ensure_binary(root, allow_build=allow_build)
        try:
            binary_note = str(binary.relative_to(root))
        except ValueError:
            binary_note = str(binary)
    except EngineUnavailable as error:
        binary = None
        binary_note = f"unavailable: {error}"

    outcomes: list[CaseOutcome] = []
    for case_dir in cases:
        workdir = workroot / case_id(benchmarks_dir, case_dir).replace("/", "__")
        outcomes.append(run_case(benchmarks_dir, case_dir, workdir, binary))

    return _summarize(outcomes, binary_note)


def _summarize(outcomes: list[CaseOutcome], binary_note: str) -> dict[str, Any]:
    families: dict[str, dict[str, int]] = {}
    cases_report: list[dict[str, Any]] = []
    counts: dict[str, int] = {}
    for outcome in sorted(outcomes, key=lambda o: o.case_id):
        counts[outcome.status] = counts.get(outcome.status, 0) + 1
        family = families.setdefault(outcome.family, {})
        family[outcome.status] = family.get(outcome.status, 0) + 1
        cases_report.append(
            {
                "id": outcome.case_id,
                "family": outcome.family,
                "declared": outcome.declared,
                "status": outcome.status,
                "result": outcome.result,
            }
        )
    regressions = [o.case_id for o in outcomes if o.status in FAILING_STATUSES]
    return {
        "binary": binary_note,
        "total": len(outcomes),
        "counts": counts,
        "families": families,
        "regressions": sorted(regressions),
        "passed_suite": not regressions,
        "cases": cases_report,
    }


def render_text(summary: dict[str, Any]) -> str:
    """Render a deterministic human-readable report."""
    lines = ["LawSynth benchmark suite", f"binary: {summary['binary']}", ""]
    for family in sorted(summary["families"]):
        breakdown = summary["families"][family]
        parts = ", ".join(f"{status}={breakdown[status]}" for status in sorted(breakdown))
        lines.append(f"[{family}] {parts}")
    lines.append("")
    for case in summary["cases"]:
        marker = {PASSED: "PASS", BOUNDARY: "BNDRY", REGRESSION: "REGR", FAILED: "FAIL"}.get(
            case["status"], case["status"].upper()
        )
        detail = ""
        vector = case["result"].get("score_vector")
        if vector:
            error = vector.get("trajectory_error")
            fit = vector.get("fit_train")
            detail = f"  mse={fit} traj_err={error}"
        lines.append(f"  {marker:<5} {case['id']}{detail}")
    lines.append("")
    total = summary["total"]
    counts = ", ".join(f"{status}={summary['counts'][status]}" for status in sorted(summary["counts"]))
    lines.append(f"total={total} ({counts})")
    lines.append("SUITE PASS" if summary["passed_suite"] else f"SUITE FAIL: {summary['regressions']}")
    return "\n".join(lines) + "\n"


def write_reports(summary: dict[str, Any], json_path: Path | None) -> None:
    if json_path is not None:
        json_path.parent.mkdir(parents=True, exist_ok=True)
        json_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    sys.stdout.write(render_text(summary))
