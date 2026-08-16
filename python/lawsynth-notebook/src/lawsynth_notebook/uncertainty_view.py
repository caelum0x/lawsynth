"""Uncertainty interval rendering with interval invariants."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any

from .display import RenderedArtifact
from .errors import ArtifactValidationError
from .serialization import finite_number
from .templates import panel, table


def normalize_intervals(intervals: Mapping[str, Mapping[str, Any]] | Sequence[Mapping[str, Any]]) -> list[dict[str, float | str]]:
    entries = ([{"name": name, **value} for name, value in intervals.items()] if isinstance(intervals, Mapping) else intervals)
    if not isinstance(entries, Sequence) or isinstance(entries, (str, bytes)):
        raise ArtifactValidationError("intervals must be an object or list")
    result = []
    for entry in entries:
        if not isinstance(entry, Mapping) or not isinstance(entry.get("name"), str):
            raise ArtifactValidationError("intervals require a name")
        lower, upper = finite_number(entry.get("lower"), "interval lower"), finite_number(entry.get("upper"), "interval upper")
        if lower > upper:
            raise ArtifactValidationError("interval lower cannot exceed upper")
        result.append({"name": entry["name"], "lower": lower, "upper": upper, "mean": finite_number(entry.get("mean", (lower + upper) / 2), "interval mean")})
    return sorted(result, key=lambda item: str(item["name"]))


def render_uncertainty(intervals: Mapping[str, Mapping[str, Any]] | Sequence[Mapping[str, Any]], theme: str = "light") -> RenderedArtifact:
    normalized = normalize_intervals(intervals)
    return RenderedArtifact("Uncertainty", panel("Uncertainty", table(["name", "lower", "mean", "upper"], [(i["name"], i["lower"], i["mean"], i["upper"]) for i in normalized]), theme), {"intervals": normalized})
