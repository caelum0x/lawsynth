"""Immutable, validated benchmark observations and JSON ingestion."""
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Mapping
import json, math
from .errors import SchemaError

@dataclass(frozen=True, slots=True)
class Observation:
    problem: str
    implementation: str
    metric: str
    value: float
    unit: str = ""
    run_id: str = ""
    labels: Mapping[str, str] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not all((self.problem.strip(), self.implementation.strip(), self.metric.strip())):
            raise SchemaError("problem, implementation, and metric are required")
        if not math.isfinite(self.value):
            raise SchemaError("observation value must be finite")

    @classmethod
    def from_dict(cls, value: Mapping[str, object]) -> "Observation":
        try:
            return cls(str(value["problem"]), str(value["implementation"]), str(value["metric"]),
                       float(value["value"]), str(value.get("unit", "")), str(value.get("run_id", "")),
                       {str(k): str(v) for k, v in dict(value.get("labels", {})).items()})
        except (KeyError, TypeError, ValueError) as exc:
            raise SchemaError("invalid observation") from exc

    def to_dict(self) -> dict[str, object]:
        return {"problem": self.problem, "implementation": self.implementation, "metric": self.metric,
                "value": self.value, "unit": self.unit, "run_id": self.run_id,
                "labels": dict(sorted(self.labels.items()))}

def load_observations(path: str | Path) -> list[Observation]:
    """Load recorded observations; this never executes LawSynth or a benchmark."""
    try:
        payload = json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise SchemaError(f"cannot read benchmark data: {path}") from exc
    rows = payload.get("observations", payload) if isinstance(payload, dict) else payload
    if not isinstance(rows, list):
        raise SchemaError("benchmark data must be an observations list")
    return [Observation.from_dict(row) for row in rows]

def dump_observations(rows: Iterable[Observation]) -> dict[str, object]:
    return {"schema": "lawsynth-bench/v1", "observations": [row.to_dict() for row in rows]}
