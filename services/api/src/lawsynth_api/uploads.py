"""Request-body / upload parsing with transport limits.

This is the ingest half of the transport: it reads the WSGI input stream,
enforces the configured request-size ceiling, verifies the declared media type,
and decodes a JSON object body.  It is the single place the API converts an
untrusted byte stream into a validated Python object before domain dispatch, so
every size/encoding/media-type rule lives here rather than inline in ``app.py``.
"""

from __future__ import annotations

import json
from typing import Mapping

from .middleware import RequestProblem


def parse_json_body(
    environ: Mapping[str, object],
    headers: Mapping[str, str],
    max_request_bytes: int,
) -> dict[str, object] | None:
    """Read and validate a JSON request body, honoring ``max_request_bytes``.

    Returns ``None`` for an empty body.  Raises :class:`RequestProblem` with the
    documented transport error code for every failure mode (length mismatch,
    oversize payload, wrong media type, malformed JSON, non-object JSON).
    """

    content_length = environ.get("CONTENT_LENGTH", "")
    if content_length in (None, ""):
        return None
    if not isinstance(content_length, str) or not content_length.isascii() or not content_length.isdecimal():
        raise RequestProblem(400, "invalid_content_length", "Content-Length must be a non-negative integer")
    length = int(content_length)
    if length > max_request_bytes:
        raise RequestProblem(413, "payload_too_large", "request body exceeds the configured limit")
    stream = environ.get("wsgi.input")
    if not hasattr(stream, "read"):
        raise RequestProblem(400, "missing_request_body", "WSGI input stream is missing")
    raw = stream.read(length + 1)
    if not isinstance(raw, bytes) or len(raw) != length:
        raise RequestProblem(400, "invalid_request_body", "request body does not match Content-Length")
    if not raw:
        return None
    media_type = headers.get("Content-Type", "").split(";", 1)[0].lower()
    if media_type != "application/json":
        raise RequestProblem(415, "unsupported_media_type", "request bodies must use application/json")
    try:
        body = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RequestProblem(400, "invalid_json", "request body must be valid UTF-8 JSON") from error
    if not isinstance(body, dict):
        raise RequestProblem(422, "validation_error", "request JSON body must be an object")
    return body
