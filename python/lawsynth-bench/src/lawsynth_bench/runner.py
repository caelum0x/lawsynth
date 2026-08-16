"""Recorded-run validation; deliberately does not execute a solver."""
from dataclasses import dataclass
from collections.abc import Iterable
from .dataset import Observation
from .environment import Environment
from .errors import SchemaError
from .reproduce import fingerprint

@dataclass(frozen=True, slots=True)
class RunArtifact:
    observations: tuple[Observation, ...]
    environment: Environment
    digest: str

def ingest(rows: Iterable[Observation], environment: Environment | None = None) -> RunArtifact:
    values = tuple(rows)
    if not values: raise SchemaError("a run must include observations")
    recorded_environment = environment or Environment.capture()
    payload = {"environment": recorded_environment.to_dict(), "observations": [row.to_dict() for row in values]}
    return RunArtifact(values, recorded_environment, fingerprint(payload))
