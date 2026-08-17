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
    # The uncertainty family remains an honest SDK-surface boundary: the CLI
    # exposes no calibrated-interval or coverage output to score.
    outcome = _run_single(benchmarks_dir, cli_binary, "uncertainty/parameter-coverage")
    assert outcome.status == BOUNDARY
    assert outcome.result["missing_public_operations"]
    assert "uncertainty" in outcome.result["feature"] or "coverage" in outcome.result["feature"]


def test_causal_family_executes_and_scores(benchmarks_dir: Path, cli_binary: Path) -> None:
    # Promoted from an SDK-surface boundary: `discover --causal` now emits a
    # real dependency-hypothesis edge count that is scored end to end.
    outcome = _run_single(benchmarks_dir, cli_binary, "causal/linear")
    assert outcome.declared == "executed"
    assert outcome.status == PASSED, outcome.result
    assert outcome.result["signal"] == "dependency_edges"
    assert outcome.result["signal_value"] >= outcome.result["expected_minimum"]
    assert outcome.result["discover_returncode"] == 0
    assert outcome.result["inspect_returncode"] == 0
    vector = outcome.result["score_vector"]
    assert vector["fit_train"] is not None
    assert vector["dependency_edges"] is not None
    assert vector["frontier_size"] is not None


def test_regime_family_executes_and_scores(benchmarks_dir: Path, cli_binary: Path) -> None:
    # Promoted from an SDK-surface boundary: `discover --regimes` now emits a
    # real regime-segment count. A single change point implies >= 2 segments.
    outcome = _run_single(benchmarks_dir, cli_binary, "regime/change-point")
    assert outcome.declared == "executed"
    assert outcome.status == PASSED, outcome.result
    assert outcome.result["signal"] == "regime_segments"
    assert outcome.result["signal_value"] >= 2
    assert outcome.result["score_vector"]["regime_segments"] is not None


def test_regime_family_regression_when_signal_missing(monkeypatch, benchmarks_dir: Path) -> None:
    # A family case that stops emitting its structural signal is a real
    # regression that fails the suite, never a silently-passing boundary.
    import _families

    case_dir = benchmarks_dir / "regime/change-point"
    config = _families.read_config(case_dir)
    empty = _families.FamilyRun(discover_returncode=0, inspect_returncode=0)
    scored = _families.score_family(config, empty)
    assert scored["status"] == REGRESSION
    assert scored["passed"] is False


def test_dynamics_family_suite_passes(benchmarks_dir: Path, cli_binary: Path) -> None:
    with tempfile.TemporaryDirectory() as workroot:
        summary = run_suite(benchmarks_dir, Path(workroot), family_filter="dynamics")
    # Two supported dynamics cases must pass; boundaries must not fail the suite.
    assert summary["passed_suite"], summary["regressions"]
    assert summary["counts"].get(PASSED) == 2
    assert summary["counts"].get(BOUNDARY) == 4
    assert summary["regressions"] == []


def test_causal_family_suite_all_execute(benchmarks_dir: Path, cli_binary: Path) -> None:
    with tempfile.TemporaryDirectory() as workroot:
        summary = run_suite(benchmarks_dir, Path(workroot), family_filter="causal")
    # All five causal cases now execute-and-score; none are boundaries.
    assert summary["passed_suite"], summary["regressions"]
    assert summary["counts"].get(PASSED) == 5
    assert summary["counts"].get(BOUNDARY) is None


def test_regime_family_suite_all_execute(benchmarks_dir: Path, cli_binary: Path) -> None:
    with tempfile.TemporaryDirectory() as workroot:
        summary = run_suite(benchmarks_dir, Path(workroot), family_filter="regime")
    assert summary["passed_suite"], summary["regressions"]
    assert summary["counts"].get(PASSED) == 4
    assert summary["counts"].get(BOUNDARY) is None


def test_full_suite_executes_eighteen_and_bounds_twelve(
    benchmarks_dir: Path, cli_binary: Path
) -> None:
    # Baseline was 9 executed; causal (5) + regime (4) promotions raise it to 18.
    with tempfile.TemporaryDirectory() as workroot:
        summary = run_suite(benchmarks_dir, Path(workroot))
    assert summary["passed_suite"], summary["regressions"]
    assert summary["counts"].get(PASSED) == 18
    assert summary["counts"].get(BOUNDARY) == 12
    assert summary["total"] == 30
