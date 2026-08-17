"""The ``datasets`` resource: route classification.

A dataset holds validated numeric observations that discovery consumes.  Like
projects it follows the standard CRUD grammar with no SSE projection; its
domain-level schema validation lives in ``lawsynth_server.datasets``.  This
module contributes the transport-facing telemetry label and an empty lifecycle
projection.
"""

from __future__ import annotations

from typing import Mapping, Sequence

from .events import EventKind

SEGMENT = "datasets"


def lifecycle_events(method: str, body: Mapping[str, object]) -> list[tuple[EventKind, str, str | None]]:
    """Datasets do not project onto the SSE contract; always empty."""

    return []


def classify(method: str, parts: Sequence[str]) -> str:
    """Return a stable telemetry label for a ``datasets`` route."""

    if len(parts) == 1:
        return "datasets.list" if method == "GET" else "datasets.create" if method == "POST" else "datasets.other"
    if len(parts) == 2:
        return {"GET": "datasets.get", "PATCH": "datasets.update", "DELETE": "datasets.delete"}.get(method, "datasets.other")
    return "datasets.other"
