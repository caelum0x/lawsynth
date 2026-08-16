"""Structured benchmark report construction."""
from dataclasses import asdict
from collections.abc import Iterable
from .aggregation import summarize
from .baseline import Change
from .dataset import Observation

def build(rows: Iterable[Observation], changes: Iterable[Change] = ()) -> dict[str, object]:
    records = list(rows); deltas = list(changes)
    return {"schema": "lawsynth-bench/report-v1", "observation_count": len(records),
            "summaries": [asdict(item) for item in summarize(records)],
            "changes": [{"key": list(change.key), "baseline": change.baseline, "candidate": change.candidate,
                         "ratio": change.ratio, "regression": change.regression} for change in deltas],
            "regression_count": sum(change.regression for change in deltas)}
