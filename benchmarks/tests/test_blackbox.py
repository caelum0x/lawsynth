"""Tests for the black-box SRBench family.

Black-box cases carry no advertised ground truth: they are scored only on
predictive trajectory R^2 and parsimony (node count), with no symbolic-recovery
expectation. Each must execute through the real CLI, score a finite R^2, and be
byte-identical on replay.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

from _engine import ensure_binary
from _srbench import read_config, run_srbench_case
from _srbench_data import write_dataset

ROOT = Path(__file__).resolve().parents[2]
BLACKBOX = ROOT / "benchmarks" / "blackbox"

CASES = ["coupled-decay", "driven-process", "three-species", "oscillatory-reactor"]


def _binary() -> Path:
    try:
        return ensure_binary(ROOT, allow_build=True)
    except Exception as exc:  # pragma: no cover - environment dependent
        pytest.skip(f"lawsynth CLI unavailable: {exc}")


def test_blackbox_dataset_is_byte_stable() -> None:
    """Black-box trajectories must be generated identically across runs."""
    case = BLACKBOX / "driven-process"
    config = read_config(case)
    with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
        ds_a = write_dataset(config, Path(a)).read_bytes()
        ds_b = write_dataset(config, Path(b)).read_bytes()
    assert ds_a == ds_b and len(ds_a) > 0


@pytest.mark.parametrize("case_name", CASES)
def test_blackbox_executes_scores_r2_and_is_deterministic(case_name: str) -> None:
    binary = _binary()
    case = BLACKBOX / case_name
    config = read_config(case)
    assert config["family"] == "blackbox"
    # Black-box protocol: no symbolic-recovery ground truth is declared.
    assert "recovery" not in config
    assert config["expect"]["symbolic_recovery"] == "none"

    with tempfile.TemporaryDirectory() as tmp:
        result = run_srbench_case(case, Path(tmp), binary, check_determinism=True)
    assert result["status"] == "passed", result
    assert result["passed"] is True
    assert result["determinism"] is True
    sv = result["score_vector"]
    # Scored purely on predictive R^2 + parsimony; no symbolic recovery claimed.
    assert sv["trajectory_r2"] is not None and sv["trajectory_r2"] >= 0.9
    assert sv["symbolic_level"] == "none"
    assert sv["complexity_nodes"] is not None


def test_blackbox_family_appears_in_full_report() -> None:
    """The reporter must pick up the black-box family end to end."""
    import srbench_report

    _binary()
    report = srbench_report.build_report("blackbox", allow_build=True)
    assert report["binary_available"] is True
    assert report["total"] == len(CASES)
    assert report["supported_regressions"] == []
    det = [r["determinism"] for r in report["rows"] if r["determinism"] is not None]
    assert det and all(det)
