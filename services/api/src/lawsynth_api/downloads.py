"""Response construction for the transport's read/stream paths.

Two download shapes are built here:

* the Server-Sent Events response for ``GET /v1/events`` -- request parsing
  (``Accept`` negotiation, ``Last-Event-ID`` cursor), the response headers, and
  the framed body generator (composed from :func:`events.render_frame`); and
* content-addressed artifact download decoration -- deriving a strong ``ETag``
  from the stored sha256 so caches and clients can revalidate a download.

Keeping this out of ``app.py`` means the dispatch loop only decides *which*
response to build, not how a stream or a download is framed.
"""

from __future__ import annotations

from typing import Iterable, Mapping

from .events import ApiEvent, render_frame
from .middleware import RequestProblem


def wants_sse(headers: Mapping[str, str]) -> bool:
    """True when the client negotiated ``text/event-stream``."""

    accept = headers.get("Accept", "")
    return isinstance(accept, str) and "text/event-stream" in accept


def last_event_id(headers: Mapping[str, str]) -> int:
    """Parse the ``Last-Event-ID`` resume cursor (0 when absent).

    A present but non-numeric value is a client error, mirroring the strict
    parsing the SSE contract requires.
    """

    raw = None
    for name, value in headers.items():
        if name.lower() == "last-event-id":
            raw = value
            break
    if raw is None or raw == "":
        return 0
    if not raw.isascii() or not raw.isdecimal():
        raise RequestProblem(400, "invalid_last_event_id", "Last-Event-ID must be a non-negative integer")
    return int(raw)


def sse_headers(request_id: str) -> dict[str, str]:
    """The response headers that keep an SSE body unbuffered and uncached."""

    return {
        "Content-Type": "text/event-stream; charset=utf-8",
        "Cache-Control": "no-cache",
        "Connection": "keep-alive",
        "X-Accel-Buffering": "no",
        "X-Content-Type-Options": "nosniff",
        "X-Request-ID": request_id,
    }


def open_stream(events: list[ApiEvent], request_id: str, start_response) -> Iterable[bytes]:
    """Start the SSE response and return its framed body generator.

    A leading comment keeps the stream valid (and non-empty) even when no events
    match, so polling clients always receive a 200 body.
    """

    start_response("200 OK", list(sse_headers(request_id).items()))

    def body() -> Iterable[bytes]:
        yield f": stream open ({len(events)} event(s))\n\n".encode("utf-8")
        for event in events:
            yield render_frame(event)

    return body()


def artifact_download_headers(body: object) -> dict[str, str]:
    """Derive cache-revalidation headers for a served artifact body.

    Returns a strong ``ETag`` built from the content's sha256 when the served
    body carries one; otherwise an empty mapping (so the caller merges nothing).
    The header is additive -- it never alters the JSON download payload.
    """

    if not isinstance(body, Mapping):
        return {}
    sha256 = body.get("sha256")
    if not isinstance(sha256, str) or len(sha256) != 64:
        return {}
    return {"ETag": f'"{sha256}"'}
