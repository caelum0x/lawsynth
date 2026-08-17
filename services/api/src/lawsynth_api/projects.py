"""The ``projects`` resource: route classification.

A project is a metadata container with the standard collection/item CRUD grammar
and no streaming projection -- creating or mutating a project does not emit an
SSE event in this deployment.  The module therefore contributes a stable
telemetry label and an (empty) lifecycle projection so ``app.py`` can treat every
resource uniformly.
"""

from __future__ import annotations

from typing import Mapping, Sequence

from .events import EventKind

SEGMENT = "projects"


def lifecycle_events(method: str, body: Mapping[str, object]) -> list[tuple[EventKind, str, str | None]]:
    """Projects do not project onto the SSE contract; always empty."""

    return []


def classify(method: str, parts: Sequence[str]) -> str:
    """Return a stable telemetry label for a ``projects`` route."""

    if len(parts) == 1:
        return "projects.list" if method == "GET" else "projects.create" if method == "POST" else "projects.other"
    if len(parts) == 2:
        return {"GET": "projects.get", "PATCH": "projects.update", "DELETE": "projects.delete"}.get(method, "projects.other")
    return "projects.other"
