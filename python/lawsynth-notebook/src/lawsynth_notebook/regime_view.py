"""Read-only rendering of decoded regime segments."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any

from .display import RenderedArtifact
from .errors import ArtifactValidationError
from .serialization import finite_number
from .templates import panel, table


def normalize_regimes(regimes: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
    if not isinstance(regimes, Sequence) or isinstance(regimes, (str, bytes)):
        raise ArtifactValidationError("regimes must be a list")
    result: list[dict[str, Any]] = []
    last_end = float("-inf")
    for regime in sorted(regimes, key=lambda value: value.get("start", float("inf")) if isinstance(value, Mapping) else float("inf")):
        if not isinstance(regime, Mapping) or not isinstance(regime.get("name"), str):
            raise ArtifactValidationError("regimes need a name")
        start, end = finite_number(regime.get("start"), "regime start"), finite_number(regime.get("end"), "regime end")
        if end <= start or start < last_end:
            raise ArtifactValidationError("regime segments must be ordered and non-overlapping")
        last_end = end
        result.append({"name": regime["name"], "start": start, "end": end})
    return result


def render_regimes(regimes: Sequence[Mapping[str, Any]], theme: str = "light") -> RenderedArtifact:
    normalized = normalize_regimes(regimes)
    return RenderedArtifact("Regimes", panel("Regimes", table(["name", "start", "end"], [(item["name"], item["start"], item["end"]) for item in normalized]), theme), {"regimes": normalized})
