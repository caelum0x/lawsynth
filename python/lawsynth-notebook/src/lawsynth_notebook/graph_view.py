"""Deterministic dependency-graph summary."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any

from .display import RenderedArtifact
from .errors import ArtifactValidationError
from .templates import panel, table


def normalize_graph(graph: Mapping[str, Any]) -> dict[str, list[str]]:
    if not isinstance(graph, Mapping):
        raise ArtifactValidationError("graph must map nodes to dependency lists")
    result: dict[str, list[str]] = {}
    for node, deps in graph.items():
        if not isinstance(node, str) or not isinstance(deps, Sequence) or isinstance(deps, (str, bytes)) or not all(isinstance(dep, str) for dep in deps):
            raise ArtifactValidationError("graph nodes and dependencies must be strings")
        result[node] = sorted(set(deps))
    unknown = {dep for deps in result.values() for dep in deps if dep not in result}
    if unknown:
        raise ArtifactValidationError(f"graph has undeclared dependencies: {sorted(unknown)}")
    return dict(sorted(result.items()))


def render_graph(graph: Mapping[str, Any], theme: str = "light") -> RenderedArtifact:
    normalized = normalize_graph(graph)
    rows = [(node, ", ".join(deps) or "—") for node, deps in normalized.items()]
    return RenderedArtifact("Dependencies", panel("Dependencies", table(["node", "depends on"], rows), theme), {"graph": normalized})
