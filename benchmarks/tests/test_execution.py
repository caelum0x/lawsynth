"""End-to-end execution and scoring tests through the real CLI binary."""

from __future__ import annotations

import tempfile
from pathlib import Path

from _runner import run_case, run_suite
from _scoring import BOUNDARY, PASSED, REGRESSION


def _run_single(benchmarks_dir: Path, binary: Path, case: str):
    case_dir = benchmarks_dir / case
    with tempfile.TemporaryDirectory() as workdir:
        return run_case(benchmarks_dir, case_dir, Path(workdir), binary)


def test_supported_ode_small_executes_and_scores(benchmarks_dir: Path, cli_binary: Path) -> None:
    outcome = _run_single(benchmarks_dir, cli_binary, "dynamics/ode-small")
    assert outcome.declared == "supported"
    assert outcome.status == PASSED, outcome.result
    vector = outcome.result["score_vector"]
    # The engine must return real, finite measurements, not fabricated ones.
    assert vector["fit_train"] is not None
    assert vector["complexity_nodes"] is not None
    assert vector["trajectory_error"] is not None
    assert vector["simulation_failure_rate"] == 0.0
    assert outcome.result["returncode"] == 0
    assert outcome.result["inspect_returncode"] == 0


def test_supported_end_to_end_simulates(benchmarks_dir: Path, cli_binary: Path) -> None:
    outcome = _run_single(benchmarks_dir, cli_binary, "performance/end-to-end")
    assert outcome.status == PASSED, outcome.result
    assert outcome.result["simulate_returncode"] == 0
    assert outcome.result["score_vector"]["trajectory_error"] is not None


def test_capability_boundary_is_not_a_hard_failure(benchmarks_dir: Path, cli_binary: Path) -> None:
    # dynamics/delay declares a hyphenated capability-boundary; it must be
    # reported as a boundary and must still generate its deterministic dataset.
    case_dir = benchmarks_dir / "dynamics/delay"
    with tempfile.TemporaryDirectory() as workdir:
        outcome = run_case(benchmarks_dir, case_dir, Path(workdir), cli_binary)
        assert outcome.declared == "capability-boundary"
        assert outcome.status == BOUNDARY
        assert outcome.status != REGRESSION
        assert outcome.result["dataset"] is not None
        # The boundary case still produces its deterministic dataset on disk.
        assert Path(outcome.result["dataset"]).is_file()


def test_contract_boundary_reports_missing_public_api(
    benchmarks_dir: Path, cli_binary: Path
) -> None:
    outcome = _run_single(benchmarks_dir, cli_binary, "causal/linear")
    assert outcome.status == BOUNDARY
    assert outcome.result["missing_public_operations"]
    assert "causal" in outcome.result["feature"]


def test_dynamics_family_suite_passes(benchmarks_dir: Path, cli_binary: Path) -> None:
    with tempfile.TemporaryDirectory() as workroot:
        summary = run_suite(benchmarks_dir, Path(workroot), family_filter="dynamics")
    # Two supported dynamics cases must pass; boundaries must not fail the suite.
    assert summary["passed_suite"], summary["regressions"]
    assert summary["counts"].get(PASSED) == 2
    assert summary["counts"].get(BOUNDARY) == 4
    assert summary["regressions"] == []
