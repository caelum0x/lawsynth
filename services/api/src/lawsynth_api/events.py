"""SSE streaming boundary: the ``ApiEvent`` value type plus an in-process bus.

This module is the Python mirror of ``crates/lawsynth-api-types/src/events.rs``
and the value contract in ``specs/service-api/streaming.md``.  The Rust side
defines only the *value* contract (fields + validation) and explicitly refuses
to define delivery semantics -- it states that "a service exposing events MUST
define those delivery semantics".  This module is where ``lawsynth_api`` defines
them, in process, for the WSGI SSE endpoint:

Scope
    Events are partitioned by an opaque scope string, ``project_id``.  In this
    deployment the streaming scope IS the caller's tenant (``organization_id``
    from the bearer token), because that is the isolation boundary a token
    grants.  Every read is scoped: a caller for scope A can never observe
    events appended under scope B.  Sequence numbers are assigned per scope and
    are strictly increasing, starting at 1 (so ``Last-Event-ID: 0`` -- the
    absence of a resume cursor -- replays everything still retained).

Retention (bounded, in-memory, single-process)
    Each scope keeps at most ``retention`` most-recent events in a ring buffer.
    When the buffer overflows, the oldest events are dropped permanently.
    There is no cross-process broker, no durable journal, and no replay of
    events older than the retained window.  A client that resumes with a
    ``Last-Event-ID`` older than the oldest retained sequence will silently
    skip the dropped events; it can detect the gap because the first delivered
    ``id`` will be greater than ``Last-Event-ID + 1``.
"""

from __future__ import annotations

import json
from collections import deque
from dataclasses import dataclass
from enum import Enum
from threading import RLock

from lawsynth_server.errors import ValidationError

# Payload bound mirrors the Rust ``payload_limit`` argument; measured in UTF-8
# bytes, matching Rust's ``payload.len()`` (byte length) check.
PAYLOAD_LIMIT_BYTES = 4096
# Default per-scope ring-buffer capacity.
DEFAULT_RETENTION = 1024


class EventKind(str, Enum):
    """Streaming event kinds.

    The first seven mirror the Rust ``EventKind`` enum
    (``crates/lawsynth-api-types``).  ``REVISION_REVIEWED`` is an additive,
    API-local audit kind for the P6 collaboration surface: an approval/review
    transition on a revision (``specs/collaboration/README.md``).  It is not run
    scoped, so it carries no ``run_id`` -- like ``ARTIFACT_CREATED``.

    ``value`` is the wire form used in the SSE ``event:`` field and in JSON.
    """

    RUN_QUEUED = "run_queued"
    RUN_STARTED = "run_started"
    PROGRESS = "progress"
    RUN_SUCCEEDED = "run_succeeded"
    RUN_FAILED = "run_failed"
    RUN_CANCELLED = "run_cancelled"
    ARTIFACT_CREATED = "artifact_created"
    REVISION_REVIEWED = "revision_reviewed"


# Kinds that describe a specific run and therefore require a ``run_id``.
RUN_SCOPED_KINDS = frozenset(
    {
        EventKind.RUN_QUEUED,
        EventKind.RUN_STARTED,
        EventKind.PROGRESS,
        EventKind.RUN_SUCCEEDED,
        EventKind.RUN_FAILED,
        EventKind.RUN_CANCELLED,
    }
)


@dataclass(frozen=True, slots=True)
class ApiEvent:
    """An immutable, validated streaming event.

    Structural validation runs in ``__post_init__`` and mirrors
    ``ApiEvent::new`` in Rust: bounded, NUL-free UTF-8 payload and a mandatory
    ``run_id`` for run-scoped kinds.  Cross-event invariants (strictly
    increasing sequence, non-decreasing time) are checked by
    :func:`validate_event_stream`, mirroring ``validate_event_stream`` in Rust.
    """

    sequence: int
    occurred_at_ms: int
    project_id: str
    run_id: str | None
    kind: EventKind
    payload: str

    def __post_init__(self) -> None:
        if not isinstance(self.sequence, int) or isinstance(self.sequence, bool) or self.sequence < 0:
            raise ValidationError("event sequence must be a non-negative integer")
        if not isinstance(self.occurred_at_ms, int) or isinstance(self.occurred_at_ms, bool) or self.occurred_at_ms < 0:
            raise ValidationError("event occurred_at_ms must be a non-negative integer")
        if not isinstance(self.project_id, str) or not self.project_id:
            raise ValidationError("event project_id is required")
        if self.run_id is not None and (not isinstance(self.run_id, str) or not self.run_id):
            raise ValidationError("event run_id must be a non-empty string when present")
        if not isinstance(self.kind, EventKind):
            raise ValidationError("event kind must be an EventKind")
        if not isinstance(self.payload, str):
            raise ValidationError("event payload must be a string")
        if "\x00" in self.payload:
            raise ValidationError("event payload must not contain NUL")
        if len(self.payload.encode("utf-8")) > PAYLOAD_LIMIT_BYTES:
            raise ValidationError(
                "event payload exceeds the maximum size",
                details={"maximum": PAYLOAD_LIMIT_BYTES},
            )
        if self.kind in RUN_SCOPED_KINDS and self.run_id is None:
            raise ValidationError("run events require a run_id")

    def to_wire(self) -> dict[str, object]:
        """Return the JSON-serializable body carried in the SSE ``data:`` line."""

        return {
            "sequence": self.sequence,
            "occurred_at_ms": self.occurred_at_ms,
            "project_id": self.project_id,
            "run_id": self.run_id,
            "kind": self.kind.value,
            "payload": self.payload,
        }


def validate_event_stream(events: list[ApiEvent]) -> None:
    """Reject a sequence that is not strictly increasing or goes back in time.

    Mirrors the Rust ``validate_event_stream`` window check.
    """

    for previous, current in zip(events, events[1:]):
        if current.sequence <= previous.sequence:
            raise ValidationError("event sequence must increase strictly")
        if current.occurred_at_ms < previous.occurred_at_ms:
            raise ValidationError("event occurred_at_ms must not go backwards")


class _Scope:
    """Per-scope ring buffer and monotonic sequence counter."""

    __slots__ = ("events", "next_sequence")

    def __init__(self, retention: int) -> None:
        self.events: deque[ApiEvent] = deque(maxlen=retention)
        self.next_sequence = 1


class EventBus:
    """Thread-safe, in-process, scope-partitioned event store and bus.

    See the module docstring for the delivery, scoping, and retention
    semantics this class guarantees.
    """

    def __init__(self, *, retention: int = DEFAULT_RETENTION) -> None:
        if not isinstance(retention, int) or isinstance(retention, bool) or retention < 1:
            raise ValidationError("event retention must be a positive integer")
        self._retention = retention
        self._lock = RLock()
        self._scopes: dict[str, _Scope] = {}

    @property
    def retention(self) -> int:
        return self._retention

    def append(
        self,
        project_id: str,
        occurred_at_ms: int,
        kind: EventKind,
        payload: str,
        *,
        run_id: str | None = None,
    ) -> ApiEvent:
        """Assign the next per-scope sequence, validate, store, and return the event.

        The oldest event is dropped if the scope's ring buffer is full.
        """

        if not isinstance(project_id, str) or not project_id:
            raise ValidationError("event project_id is required")
        with self._lock:
            scope = self._scopes.get(project_id)
            if scope is None:
                scope = _Scope(self._retention)
                self._scopes[project_id] = scope
            event = ApiEvent(
                sequence=scope.next_sequence,
                occurred_at_ms=occurred_at_ms,
                project_id=project_id,
                run_id=run_id,
                kind=kind,
                payload=payload,
            )
            scope.next_sequence += 1
            scope.events.append(event)
            return event

    def events_after(self, project_id: str, after_sequence: int) -> list[ApiEvent]:
        """Return retained events for ``project_id`` with ``sequence`` > ``after_sequence``.

        Reads are strictly scoped: an unknown or foreign scope yields ``[]``.
        """

        if not isinstance(after_sequence, int) or isinstance(after_sequence, bool) or after_sequence < 0:
            raise ValidationError("after_sequence must be a non-negative integer")
        with self._lock:
            scope = self._scopes.get(project_id)
            if scope is None:
                return []
            return [event for event in scope.events if event.sequence > after_sequence]


def render_frame(event: ApiEvent) -> bytes:
    """Serialize one event as a UTF-8 SSE frame (``id:``/``event:``/``data:``)."""

    data = json.dumps(event.to_wire(), separators=(",", ":"), allow_nan=False)
    frame = f"id: {event.sequence}\nevent: {event.kind.value}\ndata: {data}\n\n"
    return frame.encode("utf-8")
