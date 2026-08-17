"""The ``artifacts`` resource: route classification and SSE lifecycle projection.

An artifact write is content-addressed (sha256) and projects onto the streaming
contract as a single ``artifact_created`` event.  Because an artifact can be
associated with a run, the projection forwards an optional ``run_id`` so a
client tailing a run also observes its artifacts.  Download-response decoration
(ETag from the content hash) lives in :mod:`downloads`.
"""

from __future__ import annotations

import json
from typing import Mapping, Sequence

from .events import EventKind

SEGMENT = "artifacts"


def lifecycle_events(method: str, body: Mapping[str, object]) -> list[tuple[EventKind, str, str | None]]:
    """Return the streaming events implied by a successful artifact write.

    A ``POST`` to the collection always yields one ``artifact_created`` event; a
    read (or any other method) yields none.
    """

    if method != "POST":
        return []
    run_id = body.get("run_id") if isinstance(body.get("run_id"), str) else None
    payload = json.dumps({"id": body.get("id"), "sha256": body.get("sha256")}, separators=(",", ":"))
    return [(EventKind.ARTIFACT_CREATED, payload, run_id)]


def classify(method: str, parts: Sequence[str]) -> str:
    """Return a stable telemetry label for an ``artifacts`` route."""

    if len(parts) == 1 and method == "POST":
        return "artifacts.create"
    if len(parts) == 2 and method == "GET":
        return "artifacts.download"
    return "artifacts.other"
