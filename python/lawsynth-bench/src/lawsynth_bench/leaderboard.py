"""Stable ranking of implementations from recorded values."""
from dataclasses import dataclass
from collections.abc import Iterable
from .dataset import Observation
from .aggregation import summarize

@dataclass(frozen=True, slots=True)
class LeaderboardEntry:
    rank: int; implementation: str; score: float; observations: int

def rank(rows: Iterable[Observation], metric: str, *, lower_is_better: bool = False) -> list[LeaderboardEntry]:
    selected = [r for r in rows if r.metric == metric]
    totals: dict[str, list[float]] = {}
    for row in selected: totals.setdefault(row.implementation, []).append(row.value)
    ordered = sorted(totals.items(), key=lambda item: (item[1] and (sum(item[1]) / len(item[1])), item[0]), reverse=not lower_is_better)
    return [LeaderboardEntry(i + 1, name, sum(values) / len(values), len(values)) for i, (name, values) in enumerate(ordered)]
