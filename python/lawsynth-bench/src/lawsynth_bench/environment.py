"""Capture provenance that makes benchmark comparison auditable."""
from dataclasses import asdict, dataclass
import platform, sys
from typing import Mapping
from .errors import ComparisonError

@dataclass(frozen=True, slots=True)
class Environment:
    python: str; platform: str; machine: str; implementation: str
    @classmethod
    def capture(cls) -> "Environment":
        return cls(sys.version.split()[0], platform.platform(), platform.machine(), platform.python_implementation())
    def to_dict(self) -> dict[str, str]: return asdict(self)

def compatible(expected: Mapping[str, str], actual: Mapping[str, str]) -> bool:
    return all(actual.get(key) == value for key, value in expected.items())

def require_compatible(expected: Mapping[str, str], actual: Mapping[str, str]) -> None:
    if not compatible(expected, actual): raise ComparisonError("benchmark environments differ")
