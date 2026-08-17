"""The ``worlds`` resource: route classification (including the simulate action).

A world is an executable model with CRUD plus one action sub-route,
``/v1/worlds/{id}/simulate``, whose classification is delegated to
:mod:`simulations` so the two modules never disagree about what a simulate route
is.  Worlds have no SSE lifecycle projection at the transport layer.
"""

from __future__ import annotations

from typing import Mapping, Sequence

from . import simulations
from .events import EventKind

SEGMENT = "worlds"


def lifecycle_events(method: str, body: Mapping[str, object]) -> list[tuple[EventKind, str, str | None]]:
    """Worlds do not project onto the SSE contract; always empty."""

    return []


def classify(method: str, parts: Sequence[str]) -> str:
    """Return a stable telemetry label for a ``worlds`` route or its action."""

    action = simulations.classify(method, parts)
    if action is not None:
        return action
    if len(parts) == 1:
        return "worlds.list" if method == "GET" else "worlds.create" if method == "POST" else "worlds.other"
    if len(parts) == 2:
        return {"GET": "worlds.get", "PATCH": "worlds.update", "DELETE": "worlds.delete"}.get(method, "worlds.other")
    return "worlds.other"
