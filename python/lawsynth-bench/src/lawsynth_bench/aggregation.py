"""Deterministic grouping and summary statistics for observations."""
from dataclasses import dataclass
from collections import defaultdict
from collections.abc import Iterable
from .dataset import Observation
from .metrics import mean, median

@dataclass(frozen=True, slots=True)
class Summary:
    problem: str; implementation: str; metric: str; unit: str
    count: int; mean: float; median: float; minimum: float; maximum: float

def summarize(rows: Iterable[Observation]) -> list[Summary]:
    groups: dict[tuple[str, str, str, str], list[float]] = defaultdict(list)
    for row in rows: groups[(row.problem, row.implementation, row.metric, row.unit)].append(row.value)
    return [Summary(*key, len(values), mean(values), median(values), min(values), max(values))
            for key, values in sorted(groups.items())]
