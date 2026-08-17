"""Tests for the benchmark site generator.

Synthetic benchmark cases are written to a temporary directory so the suite is
deterministic and offline.  The real ``benchmarks/`` tree is loaded when present
to confirm the loader tolerates the checked-in layout.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from charts import status_bar_chart  # noqa: E402
from compare import PASS, REGRESSION, classify, summarize  # noqa: E402
from main import build_site  # noqa: E402
from publish import render_site  # noqa: E402
from results import load_case, load_results  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[3]
REAL_BENCHMARKS = REPO_ROOT / "benchmarks"


def _write_case(
    root: Path,
    benchmark_id: str,
    *,
    capability: str = "supported",
    expected: str = "passed",
    observed: str | None = None,
) -> Path:
    directory = root / benchmark_id
    directory.mkdir(parents=True)
    (directory / "benchmark.toml").write_text(
        f'id = "{benchmark_id}"\ntitle = "Case {benchmark_id}"\n\n'
        f'[capability]\nstatus = "{capability}"\nreason = "test"\n',
        encoding="utf-8",
    )
    (directory / "expected.json").write_text(
        json.dumps({"benchmark": benchmark_id, "expected_status": expected}),
        encoding="utf-8",
    )
    if observed is not None:
        (directory / "score.json").write_text(
            json.dumps({"status": observed, "passed": observed == "passed"}),
            encoding="utf-8",
        )
    return directory


def test_load_case(tmp_path: Path) -> None:
    directory = _write_case(tmp_path, "dynamics/ode-small", observed="passed")
    result = load_case(directory)
    assert result.benchmark_id == "dynamics/ode-small"
    assert result.category == "dynamics"
    assert result.capability == "supported"
    assert result.observed_status == "passed"


def test_classify_pass_and_regression(tmp_path: Path) -> None:
    passing = load_case(_write_case(tmp_path, "a/pass", observed="passed"))
    regressed = load_case(_write_case(tmp_path, "a/regress", observed="failed"))
    assert classify(passing).status == PASS
    assert classify(regressed).status == REGRESSION


def test_summarize_counts(tmp_path: Path) -> None:
    _write_case(tmp_path, "a/one", observed="passed")
    _write_case(tmp_path, "a/two", observed="failed")
    _write_case(tmp_path, "a/three")  # no run -> pending
    summary = summarize(load_results(tmp_path))
    assert summary.total == 3
    assert summary.counts[PASS] == 1
    assert summary.has_problems  # the failure


def test_render_site_contains_table_and_chart(tmp_path: Path) -> None:
    _write_case(tmp_path, "a/one", observed="passed")
    results = load_results(tmp_path)
    summary = summarize(results)
    site = render_site(results, summary)
    assert "<table>" in site.html
    assert "<svg" in site.html
    assert "a/one" in site.html
    payload = json.loads(site.results_json)
    assert payload["benchmarks"][0]["verdict"] == PASS


def test_chart_is_deterministic(tmp_path: Path) -> None:
    _write_case(tmp_path, "a/one", observed="passed")
    summary = summarize(load_results(tmp_path))
    assert status_bar_chart(summary) == status_bar_chart(summary)


def test_build_site_writes_files(tmp_path: Path) -> None:
    source = tmp_path / "benchmarks"
    _write_case(source, "a/one", observed="passed")
    out = tmp_path / "site"
    ok = build_site(source, out)
    assert ok
    assert (out / "index.html").is_file()
    assert (out / "results.json").is_file()


@pytest.mark.skipif(not REAL_BENCHMARKS.is_dir(), reason="benchmarks/ not present")
def test_loads_real_benchmarks() -> None:
    results = load_results(REAL_BENCHMARKS)
    assert results  # at least one checked-in case
    summary = summarize(results)
    site = render_site(results, summary)
    assert "<table>" in site.html
