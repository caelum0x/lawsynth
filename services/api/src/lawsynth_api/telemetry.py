"""In-process request telemetry for the WSGI transport.

This is a genuinely-new API-layer capability (the domain has its own privacy
counters; this one counts *HTTP* requests).  It records a ``(route_label,
status)`` tally for every request the transport completes and exposes an
immutable snapshot.  It stores no payloads, identifiers, or tenant data -- only
route labels and status codes -- and is safe to call from concurrent WSGI
workers.  It is intentionally not exposed over HTTP: nothing here fabricates a
domain capability; it is process introspection surfaced via ``app.readiness()``.
"""

from __future__ import annotations

from collections import Counter
from threading import RLock


class RequestTelemetry:
    """Thread-safe counters keyed by ``(route_label, status)``."""

    def __init__(self) -> None:
        self._lock = RLock()
        self._counts: Counter[tuple[str, int]] = Counter()
        self._total = 0

    def record(self, route: str, status: int) -> None:
        """Increment the tally for one completed request."""

        if not isinstance(route, str) or not route:
            route = "unknown"
        try:
            code = int(status)
        except (TypeError, ValueError):
            code = 0
        with self._lock:
            self._counts[(route, code)] += 1
            self._total += 1

    def total(self) -> int:
        """Return the number of requests recorded so far."""

        with self._lock:
            return self._total

    def snapshot(self) -> dict[str, object]:
        """Return an immutable view of the counters.

        ``by_route`` keys are ``"<label>:<status>"`` so the snapshot is a flat,
        JSON-serializable mapping.
        """

        with self._lock:
            return {
                "total": self._total,
                "by_route": {f"{route}:{status}": count for (route, status), count in self._counts.items()},
            }
