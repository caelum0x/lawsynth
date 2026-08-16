"""Event-log validation shared by event and regime views."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any

from .errors import ArtifactValidationError
from .serialization import finite_number


def normalize_events(events: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
    if not isinstance(events, Sequence) or isinstance(events, (str, bytes)):
        raise ArtifactValidationError("events must be a list")
    normalized: list[dict[str, Any]] = []
    for event in events:
        if not isinstance(event, Mapping) or not isinstance(event.get("kind"), str) or not event["kind"]:
            raise ArtifactValidationError("each event requires a non-empty kind")
        normalized.append({"time": finite_number(event.get("time"), "event time"), "kind": event["kind"], "detail": str(event.get("detail", ""))})
    return sorted(normalized, key=lambda item: (item["time"], item["kind"], item["detail"]))
