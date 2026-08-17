"""Load the checked-in scientific benchmark cases from ``benchmarks/``.

Each benchmark case is a directory containing:

* ``benchmark.toml`` — id, title, and a ``[capability]`` status
* ``expected.json`` — the expected observable status (e.g. ``passed``)
* ``baseline.json`` — the implementation baseline description

A run may additionally drop a ``score.json`` (produced by the case's own
``score.py``) recording the observed status of the latest execution.  This
module reads those declarative files; it never executes benchmarks itself, so it
is deterministic and offline.
"""

from __future__ import annotations

import json
import tomllib
from dataclasses import dataclass
from pathlib import Path

BENCHMARK_DESCRIPTOR = "benchmark.toml"


@dataclass(frozen=True)
class BenchmarkResult:
    benchmark_id: str
    title: str
    category: str
    capability: str
    expected_status: str
    observed_status: str | None

    @property
    def has_run(self) -> bool:
        return self.observed_status is not None


def _read_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def _read_json(path: Path) -> dict[str, object]:
    if not path.is_file():
        return {}
    loaded = json.loads(path.read_text(encoding="utf-8"))
    return loaded if isinstance(loaded, dict) else {}


def load_case(directory: Path) -> BenchmarkResult:
    """Load a single benchmark case from its directory."""
    directory = Path(directory)
    config = _read_toml(directory / BENCHMARK_DESCRIPTOR)
    capability = config.get("capability", {})
    capability_status = (
        str(capability["status"]) if isinstance(capability, dict) and "status" in capability
        else "unknown"
    )

    expected = _read_json(directory / "expected.json")
    expected_status = str(expected.get("expected_status", "unknown"))

    score = _read_json(directory / "score.json")
    observed_status = str(score["status"]) if "status" in score else None

    benchmark_id = str(config.get("id", directory.name))
    category = benchmark_id.split("/", 1)[0] if "/" in benchmark_id else directory.parent.name

    return BenchmarkResult(
        benchmark_id=benchmark_id,
        title=str(config.get("title", benchmark_id)),
        category=category,
        capability=capability_status,
        expected_status=expected_status,
        observed_status=observed_status,
    )


def load_results(benchmarks_dir: Path) -> list[BenchmarkResult]:
    """Load every benchmark case under ``benchmarks_dir``, sorted by id."""
    benchmarks_dir = Path(benchmarks_dir)
    if not benchmarks_dir.is_dir():
        raise FileNotFoundError(f"not a directory: {benchmarks_dir}")
    results = [
        load_case(descriptor.parent)
        for descriptor in benchmarks_dir.rglob(BENCHMARK_DESCRIPTOR)
    ]
    results.sort(key=lambda result: result.benchmark_id)
    return results
