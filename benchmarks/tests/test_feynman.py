"""Tests for the Feynman SRBench family.

The Feynman family is deliberately split into two honest halves:

* **static** ``y = f(x1..xn)`` algebraic relations, which the LawSynth *dynamics*
  CLI does not address — these must be classified as ``capability-boundary`` with
  their dataset still generated deterministically (never a fabricated recovery);
* **dynamics-framed** Feynman relations (decay/cooling laws) whose governing law is
  a first-order ODE the CLI *does* discover — these must execute, simulate, score a
  high trajectory R^2, and be byte-identical on replay.

Cases that need the compiled binary skip explicitly if it cannot be built.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

from _engine import ensure_binary
from _srbench import read_config, run_srbench_case
from _srbench_data import write_dataset

ROOT = Path(__file__).resolve().parents[2]
FEYNMAN = ROOT / "benchmarks" / "feynman"

# A representative static (boundary) case and the dynamics-framed (supported) cases.
STATIC_BOUNDARY = "gravitation"
DYNAMICS_FRAMED = ["radioactive-decay", "newton-cooling", "rc-discharge"]


def _binary() -> Path:
    try:
        return ensure_binary(ROOT, allow_build=True)
    except Exception as exc:  # pragma: no cover - environment dependent
        pytest.skip(f"lawsynth CLI unavailable: {exc}")


def test_static_feynman_is_honest_capability_boundary() -> None:
    """A static y=f(x) Feynman case must be a boundary with a generated dataset.

    We assert the case is *not* fake-recovered: no discover/score vector is
    produced, it is explicitly classified as a capability boundary, and the
    deterministic dataset is nonetheless materialised (proving the generator).
    """
    case = FEYNMAN / STATIC_BOUNDARY
    config = read_config(case)
    assert config["family"] == "feynman"
    assert config["capability"]["status"] == "capability-boundary"

    with tempfile.TemporaryDirectory() as tmp:
        # No binary is needed: a boundary case never invokes the CLI.
        result = run_srbench_case(case, Path(tmp), binary=None, check_determinism=False)
        assert result["status"] == "capability-boundary"
        assert result["passed"] is False
        # Honest: no recovery / score vector is fabricated for a boundary.
        assert "score_vector" not in result
        assert result.get("determinism") is None
        # The deterministic dataset is still generated (proves the generator).
        dataset = Path(result["dataset"])
        assert dataset.is_file()
        content = dataset.read_bytes()
        assert content and content.splitlines()[0].endswith(b",y")


def test_static_feynman_dataset_is_byte_stable() -> None:
    """The generated static dataset must be identical across runs (determinism)."""
    case = FEYNMAN / STATIC_BOUNDARY
    config = read_config(case)
    with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
        ds_a = write_dataset(config, Path(a)).read_bytes()
        ds_b = write_dataset(config, Path(b)).read_bytes()
    assert ds_a == ds_b and len(ds_a) > 0


@pytest.mark.parametrize("case_name", DYNAMICS_FRAMED)
def test_dynamics_framed_feynman_executes_and_recovers(case_name: str) -> None:
    """A Feynman relation framed as an ODE must execute, score R^2, be deterministic."""
    binary = _binary()
    case = FEYNMAN / case_name
    config = read_config(case)
    assert config["capability"]["status"] == "supported"
    with tempfile.TemporaryDirectory() as tmp:
        result = run_srbench_case(case, Path(tmp), binary, check_determinism=True)
    assert result["status"] == "passed", result
    assert result["passed"] is True
    assert result["determinism"] is True
    sv = result["score_vector"]
    assert sv["trajectory_r2"] is not None and sv["trajectory_r2"] >= 0.99
    assert sv["fit_train"] is not None
