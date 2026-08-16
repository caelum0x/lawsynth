"""Small, immutable rendering policy."""

from dataclasses import dataclass

from .errors import ArtifactValidationError


@dataclass(frozen=True, slots=True)
class NotebookConfig:
    max_rows: int = 200
    max_series_points: int = 2_000
    theme: str = "light"
    include_raw_json: bool = False

    def __post_init__(self) -> None:
        if self.max_rows < 1 or self.max_series_points < 2:
            raise ArtifactValidationError("row and series limits must be positive")
        if self.theme not in {"light", "dark"}:
            raise ArtifactValidationError("theme must be 'light' or 'dark'")
