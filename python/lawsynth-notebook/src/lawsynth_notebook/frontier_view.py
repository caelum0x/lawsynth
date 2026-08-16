"""Candidate-frontier validation and rendering."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any

from .display import RenderedArtifact
from .errors import ArtifactValidationError
from .serialization import finite_number
from .templates import panel, table


def normalize_frontier(frontier: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
    if not isinstance(frontier, Sequence) or isinstance(frontier, (str, bytes)):
        raise ArtifactValidationError("frontier must be a list")
    normalized: list[dict[str, Any]] = []
    ids: set[str] = set()
    for candidate in frontier:
        if not isinstance(candidate, Mapping) or not isinstance(candidate.get("id"), str) or not candidate["id"]:
            raise ArtifactValidationError("frontier candidates require a string id")
        identifier = candidate["id"]
        if identifier in ids:
            raise ArtifactValidationError(f"duplicate candidate id {identifier!r}")
        ids.add(identifier)
        normalized.append({"id": identifier, "score": finite_number(candidate.get("score"), f"candidate {identifier} score"), "complexity": finite_number(candidate.get("complexity"), f"candidate {identifier} complexity"), "equation": str(candidate.get("equation", ""))})
    return sorted(normalized, key=lambda item: (item["score"], item["complexity"], item["id"]))


def render_frontier(frontier: Sequence[Mapping[str, Any]], theme: str = "light") -> RenderedArtifact:
    normalized = normalize_frontier(frontier)
    rows = [(item["id"], item["score"], item["complexity"], item["equation"]) for item in normalized]
    return RenderedArtifact("Candidate frontier", panel("Candidate frontier", table(["id", "score", "complexity", "equation"], rows), theme), {"frontier": normalized})
