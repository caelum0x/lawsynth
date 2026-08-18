"""Tests for the SRBench-style credibility benchmark harness.

These drive the real compiled ``lawsynth`` CLI on the Strogatz ODE family. If the
binary cannot be produced they skip with an explicit message (never a silent
pass), matching the rest of the benchmark suite.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

from _engine import ensure_binary
from _srbench import read_config, run_srbench_case
from _srbench_data import write_dataset
import srbench_report

ROOT = Path(__file__).resolve().parents[2]
STROGATZ = ROOT / "benchmarks" / "strogatz"


def _binary() -> Path:
    try:
        return ensure_binary(ROOT, allow_build=True)
    except Exception as exc:  # pragma: no cover - environment dependent
        pytest.skip(f"lawsynth CLI unavailable: {exc}")


def test_strogatz_data_generation_is_byte_stable() -> None:
    """The generated observation CSV must be identical across runs (determinism)."""
    case = STROGATZ / "damped-oscillator"
    config = read_config(case)
    with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
        ds_a = write_dataset(config, Path(a)).read_bytes()
        ds_b = write_dataset(config, Path(b)).read_bytes()
    assert ds_a == ds_b and len(ds_a) > 0


@pytest.mark.parametrize("case_name", ["damped-oscillator", "lotka-volterra", "van-der-pol"])
def test_strogatz_case_executes_scores_and_is_deterministic(case_name: str) -> None:
    binary = _binary()
    case = STROGATZ / case_name
    with tempfile.TemporaryDirectory() as tmp:
        result = run_srbench_case(case, Path(tmp), binary, check_determinism=True)
    assert result["status"] in {"passed", "capability-boundary"}, result
    if result["status"] == "passed":
        assert result["passed"] is True
        # Determinism is the headline metric: byte-identical worlds on replay.
        assert result["determinism"] is True
        sv = result["score_vector"]
        assert sv["fit_train"] is not None
        assert sv["training_time_ns"] is not None


def test_srbench_report_runs_and_reports_determinism() -> None:
    _binary()  # ensure a binary exists (or skip)
    report = srbench_report.build_report("strogatz", allow_build=True)
    assert report["binary_available"] is True
    assert report["total"] >= 3
    # No supported regressions on the checked-in Strogatz cases.
    assert report["supported_regressions"] == []
    det = [r["determinism"] for r in report["rows"] if r["determinism"] is not None]
    assert det and all(det), "every executed case must be byte-identical on replay"
    print(srbench_report.render_table(report))
