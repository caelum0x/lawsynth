"""The ``runs`` resource: route classification and SSE lifecycle projection.

Runs are the only mutable resource whose domain state transitions are projected
onto the streaming contract (specs/service-api/streaming.md).  This module owns
that projection -- the map from a run ``status`` to an :class:`EventKind` and the
extraction of ``(id, status)`` from a dispatch response -- so ``app.py`` can emit
run events without embedding run semantics in the transport loop.
"""

from __future__ import annotations

import json
from typing import Mapping, Sequence

from .events import EventKind

SEGMENT = "runs"

# Maps a domain run ``status`` to the streaming event kind for that transition.
# The domain models no separate "progress" concept, so ``EventKind.PROGRESS``
# is part of the value contract but is not emitted by run status transitions.
STATUS_KINDS = {
    "queued": EventKind.RUN_QUEUED,
    "running": EventKind.RUN_STARTED,
    "succeeded": EventKind.RUN_SUCCEEDED,
    "failed": EventKind.RUN_FAILED,
    "cancelled": EventKind.RUN_CANCELLED,
}

# The API SSE frame carries only the run identity and status; the full record is
# retrievable from the run resource, so the stream stays small and stable.
_EMITTING_METHODS = frozenset({"POST", "PATCH"})


def lifecycle_events(method: str, body: Mapping[str, object]) -> list[tuple[EventKind, str, str | None]]:
    """Return the streaming events implied by a successful run mutation.

    Emits at most one event: a status transition for a create/update/cancel that
    carries a known ``status``.  Anything else (unknown status, missing id)
    projects to no event, matching the run status vocabulary exactly.
    """

    if method not in _EMITTING_METHODS:
        return []
    run_id = body.get("id")
    status = body.get("status")
    if not isinstance(run_id, str) or not isinstance(status, str):
        return []
    kind = STATUS_KINDS.get(status)
    if kind is None:
        return []
    payload = json.dumps({"id": run_id, "status": status}, separators=(",", ":"))
    return [(kind, payload, run_id)]


def classify(method: str, parts: Sequence[str]) -> str:
    """Return a stable telemetry label for a ``runs`` route."""

    if len(parts) == 3 and parts[2] == "cancel" and method == "POST":
        return "runs.cancel"
    if len(parts) == 3 and parts[2] == "events" and method == "GET":
        return "runs.events"
    if len(parts) == 1:
        return "runs.list" if method == "GET" else "runs.create" if method == "POST" else "runs.other"
    if len(parts) == 2:
        return {"GET": "runs.get", "PATCH": "runs.update", "DELETE": "runs.delete"}.get(method, "runs.other")
    return "runs.other"
