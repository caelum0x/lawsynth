"""Classify each benchmark result against its expected status."""

from __future__ import annotations

from dataclasses import dataclass

from results import BenchmarkResult

# Status a result is assigned for the rendered site.
PASS = "pass"
FAIL = "fail"
REGRESSION = "regression"
PENDING = "pending"
BOUNDARY = "capability-boundary"


@dataclass(frozen=True)
class Verdict:
    benchmark_id: str
    status: str

    @property
    def is_problem(self) -> bool:
        return self.status in (FAIL, REGRESSION)


def classify(result: BenchmarkResult) -> Verdict:
    """Map a benchmark result to a site verdict.

    * A case the engine does not yet support is a declared capability boundary.
    * A case that has not been run this cycle is pending.
    * Otherwise the observed status is compared with the expected status: an
      agreement passes, an expected-pass that now fails is a regression, and any
      other disagreement is a failure.
    """
    if result.capability != "supported":
        return Verdict(result.benchmark_id, BOUNDARY)
    if not result.has_run:
        return Verdict(result.benchmark_id, PENDING)

    observed = result.observed_status
    expected = result.expected_status
    if observed == expected:
        return Verdict(result.benchmark_id, PASS)
    if expected == "passed" and observed in ("failed", "capability-boundary"):
        return Verdict(result.benchmark_id, REGRESSION)
    return Verdict(result.benchmark_id, FAIL)


@dataclass(frozen=True)
class Summary:
    counts: dict[str, int]
    verdicts: tuple[Verdict, ...]

    @property
    def total(self) -> int:
        return len(self.verdicts)

    @property
    def has_problems(self) -> bool:
        return any(verdict.is_problem for verdict in self.verdicts)


def summarize(results: list[BenchmarkResult]) -> Summary:
    verdicts = tuple(classify(result) for result in results)
    counts: dict[str, int] = {}
    for verdict in verdicts:
        counts[verdict.status] = counts.get(verdict.status, 0) + 1
    return Summary(counts=counts, verdicts=verdicts)
