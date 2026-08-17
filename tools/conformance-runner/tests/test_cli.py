"""Tests for the conformance runner.

Synthetic cases are written to a temporary directory and executed with an
injected in-process runner, so the suite is deterministic and offline (no Rust
toolchain, no subprocess).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from compare import compare  # noqa: E402
from discover import DiscoveryError, discover_cases, load_case  # noqa: E402
from execute import execute_case, extract_json  # noqa: E402
from main import run_suite  # noqa: E402


def _write_case(root: Path, case_id: str, expected: dict, mode: str = "valid") -> Path:
    directory = root / case_id
    directory.mkdir(parents=True)
    (directory / "case.toml").write_text(
        f'[case]\nid = "{case_id}"\nmode = "{mode}"\n\n'
        '[execution]\nrunner = "python3 run.py"\n',
        encoding="utf-8",
    )
    (directory / "expected.json").write_text(json.dumps(expected), encoding="utf-8")
    return directory


def _runner_emitting(payload: dict, returncode: int = 0):
    def runner(command, cwd, timeout):  # noqa: ARG001
        return returncode, json.dumps(payload), ""

    return runner


def test_discover_sorts_cases(tmp_path: Path) -> None:
    _write_case(tmp_path, "zeta", {"outcome": "accepted"})
    _write_case(tmp_path, "alpha", {"outcome": "accepted"})
    cases = discover_cases(tmp_path)
    assert [case.case_id for case in cases] == ["alpha", "zeta"]


def test_discover_filter(tmp_path: Path) -> None:
    _write_case(tmp_path, "continuous-world", {})
    _write_case(tmp_path, "discrete-world", {})
    cases = discover_cases(tmp_path, name_filter="discrete")
    assert [case.case_id for case in cases] == ["discrete-world"]


def test_load_case_missing_descriptor(tmp_path: Path) -> None:
    (tmp_path / "empty").mkdir()
    with pytest.raises(DiscoveryError):
        load_case(tmp_path / "empty")


def test_extract_json_takes_last_object() -> None:
    stdout = "building fixture\n{\"status\": \"ok\"}\n"
    assert extract_json(stdout) == {"status": "ok"}
    assert extract_json("no json here") is None


def test_compare_tolerant_of_float_noise() -> None:
    result = compare({"value": 1.0}, {"value": 1.0 + 1e-12})
    assert result.passed
    mismatch = compare({"value": 1.0}, {"value": 2.0})
    assert not mismatch.passed
    assert mismatch.differences


def test_execute_and_report_pass(tmp_path: Path) -> None:
    _write_case(tmp_path, "case-ok", {"outcome": "accepted"})
    report = run_suite(tmp_path, runner=_runner_emitting({"outcome": "accepted"}))
    assert report.ok
    assert report.passed == 1


def test_execute_and_report_fail_on_mismatch(tmp_path: Path) -> None:
    _write_case(tmp_path, "case-bad", {"outcome": "accepted"})
    report = run_suite(tmp_path, runner=_runner_emitting({"outcome": "rejected"}))
    assert not report.ok
    assert report.failed == 1


def test_execute_and_report_fail_on_nonzero_exit(tmp_path: Path) -> None:
    _write_case(tmp_path, "case-crash", {"outcome": "accepted"})
    report = run_suite(tmp_path, runner=_runner_emitting({"outcome": "accepted"}, returncode=2))
    assert not report.ok


def test_execute_case_missing_observed(tmp_path: Path) -> None:
    directory = _write_case(tmp_path, "case-silent", {"outcome": "accepted"})
    case = load_case(directory)
    result = execute_case(case, runner=lambda *_: (0, "no json output", ""))
    assert result.observed is None
