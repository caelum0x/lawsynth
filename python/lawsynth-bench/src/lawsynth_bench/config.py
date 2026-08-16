"""Configuration governing benchmark comparison."""
from dataclasses import dataclass
from .errors import SchemaError

@dataclass(frozen=True, slots=True)
class BenchmarkConfig:
    regression_ratio: float = 1.05
    significance_floor: float = 0.0
    lower_is_better: bool = True

    def __post_init__(self) -> None:
        if self.regression_ratio < 1:
            raise SchemaError("regression_ratio must be at least 1")
        if self.significance_floor < 0:
            raise SchemaError("significance_floor must be non-negative")
