"""Performance-specific recorded sample analysis."""
from collections.abc import Iterable
from .dataset import Observation
from .baseline import Change, compare
from .config import BenchmarkConfig

TIME_UNITS = frozenset({"ns", "us", "ms", "s"})

def time_observations(rows: Iterable[Observation]) -> list[Observation]:
    return [row for row in rows if row.unit in TIME_UNITS]

def regressions(baseline: Iterable[Observation], candidate: Iterable[Observation], config: BenchmarkConfig = BenchmarkConfig()) -> list[Change]:
    return [change for change in compare(time_observations(baseline), time_observations(candidate), config) if change.regression]
