"""Equation inspection without evaluating expressions."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any

from .display import RenderedArtifact
from .errors import ArtifactValidationError
from .templates import panel, table


def equation_rows(equations: Mapping[str, str] | Sequence[Mapping[str, Any]]) -> list[tuple[str, str]]:
    if isinstance(equations, Mapping):
        rows = list(equations.items())
    elif isinstance(equations, Sequence) and not isinstance(equations, (str, bytes)):
        rows = [(item.get("target"), item.get("expression")) for item in equations if isinstance(item, Mapping)]
        if len(rows) != len(equations):
            raise ArtifactValidationError("equation entries must be objects")
    else:
        raise ArtifactValidationError("equations must be an object or list")
    if not rows or any(not isinstance(target, str) or not target or not isinstance(expr, str) or not expr for target, expr in rows):
        raise ArtifactValidationError("each equation needs a non-empty target and expression")
    return sorted(rows)


def render_equations(equations: Mapping[str, str] | Sequence[Mapping[str, Any]], theme: str = "light") -> RenderedArtifact:
    rows = equation_rows(equations)
    return RenderedArtifact("Equations", panel("Equations", table(["target", "expression"], rows), theme), {"equations": dict(rows)})
