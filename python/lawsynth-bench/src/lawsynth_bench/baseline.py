"""Baseline snapshots and conservative regression comparison."""
from dataclasses import dataclass
from collections.abc import Iterable
from .aggregation import Summary, summarize
from .config import BenchmarkConfig
from .dataset import Observation
from .errors import ComparisonError

@dataclass(frozen=True, slots=True)
class Change:
    key: tuple[str, str, str, str]; baseline: float; candidate: float; ratio: float; regression: bool

def compare(baseline: Iterable[Observation], candidate: Iterable[Observation], config: BenchmarkConfig = BenchmarkConfig()) -> list[Change]:
    before = {(s.problem, s.implementation, s.metric, s.unit): s.mean for s in summarize(baseline)}
    after = {(s.problem, s.implementation, s.metric, s.unit): s.mean for s in summarize(candidate)}
    if before.keys() != after.keys():
        missing = sorted(before.keys() ^ after.keys())
        raise ComparisonError(f"benchmark sets differ in {len(missing)} metric groups")
    changes = []
    for key in sorted(before):
        old, new = before[key], after[key]
        if old == 0: ratio = 1.0 if new == 0 else float("inf")
        else: ratio = new / old
        regression = (ratio >= config.regression_ratio and abs(new - old) >= config.significance_floor) if config.lower_is_better else (ratio <= 1 / config.regression_ratio and abs(new - old) >= config.significance_floor)
        changes.append(Change(key, old, new, ratio, regression))
    return changes
