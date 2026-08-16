"""Append-only, organization-scoped domain event journal."""

from __future__ import annotations

from dataclasses import asdict, dataclass
from datetime import UTC, datetime
from threading import RLock
from uuid import uuid4


@dataclass(frozen=True, slots=True)
class Event:
    event_id: str
    organization_id: str
    topic: str
    payload: dict[str, object]
    occurred_at: str


class EventJournal:
    def __init__(self) -> None:
        self._events: list[Event] = []
        self._lock = RLock()

    def append(self, organization_id: str, topic: str, payload: dict[str, object]) -> Event:
        event = Event(str(uuid4()), organization_id, topic, dict(payload), datetime.now(UTC).isoformat())
        with self._lock:
            self._events.append(event)
        return event

    def list(self, organization_id: str, *, after: str | None = None) -> list[dict[str, object]]:
        with self._lock:
            events = [event for event in self._events if event.organization_id == organization_id]
        if after:
            seen = next((i for i, event in enumerate(events) if event.event_id == after), None)
            events = events[(seen + 1) if seen is not None else 0 :]
        return [asdict(event) for event in events]
