"""WSGI middleware concerns extracted from the request/response boundary.

These helpers own the cross-cutting transport behavior that wraps every domain
dispatch: the JSON error envelope, protocol-version negotiation, request-header
extraction, and final response serialization (safe headers + content length).
They are deliberately free of resource knowledge so ``app.py`` can compose them
around any handler without duplicating the envelope contract.
"""

from __future__ import annotations

import json
from http import HTTPStatus
from typing import Callable, Iterable, Mapping
from uuid import uuid4

# The explicit protocol version this transport publishes on every response and
# negotiates against the optional ``X-Api-Version`` request header.  It tracks
# the ``/v1`` route prefix (specs/service-api/versioning.md).
PROTOCOL_VERSION = "1"
ACCEPTED_API_VERSIONS = frozenset({PROTOCOL_VERSION, f"v{PROTOCOL_VERSION}"})

StartResponse = Callable[[str, list[tuple[str, str]]], object]


class RequestProblem(Exception):
    """A transport-level failure that maps directly to an error envelope.

    Raised by the middleware and the upload/stream parsers before (or instead
    of) domain dispatch; ``app.py`` catches it and renders the envelope.
    """

    def __init__(self, status: int, code: str, message: str) -> None:
        self.status, self.code, self.message = status, code, message
        super().__init__(message)


def error_envelope(status: int, code: str, message: str, request_id: str) -> dict[str, object]:
    """Build the canonical JSON error body shared by every failure path."""

    return {
        "status": status,
        "headers": {"X-Request-ID": request_id},
        "body": {"error": {"code": code, "message": message, "request_id": request_id}},
    }


def negotiate_api_version(headers: Mapping[str, str]) -> None:
    """Reject a client that pins an ``X-Api-Version`` this transport cannot serve."""

    requested_version = headers.get("X-Api-Version")
    if requested_version is not None and requested_version not in ACCEPTED_API_VERSIONS:
        raise RequestProblem(
            406,
            "unsupported_api_version",
            "this endpoint only serves API protocol version 1",
        )


def extract_headers(environ: Mapping[str, object]) -> dict[str, str]:
    """Translate CGI-style ``HTTP_*`` keys into title-cased header names."""

    headers: dict[str, str] = {}
    for key, value in environ.items():
        if not isinstance(key, str) or not key.startswith("HTTP_") or not isinstance(value, str):
            continue
        name = key[5:].replace("_", "-").title()
        if "\r" in value or "\n" in value:
            raise RequestProblem(400, "invalid_header", "headers cannot contain line breaks")
        headers[name] = value
    content_type = environ.get("CONTENT_TYPE")
    if isinstance(content_type, str) and content_type:
        headers["Content-Type"] = content_type
    return headers


def finalize_response(response: Mapping[str, object], start_response: StartResponse) -> Iterable[bytes]:
    """Serialize a handler response into a WSGI body with safe, uniform headers.

    Invalid status codes and unserializable bodies collapse to a 500 envelope so
    a handler bug can never emit a malformed HTTP frame.
    """

    status = response.get("status")
    if not isinstance(status, int) or status not in HTTPStatus._value2member_map_:
        status, response = 500, error_envelope(500, "internal_error", "internal server error", str(uuid4()))
    body = response.get("body", {})
    try:
        payload = b"" if status == 204 else json.dumps(body, allow_nan=False, separators=(",", ":")).encode("utf-8")
    except (TypeError, ValueError):
        status = 500
        payload = json.dumps(
            error_envelope(500, "internal_error", "internal server error", str(uuid4()))["body"],
            separators=(",", ":"),
        ).encode("utf-8")
    response_headers = response.get("headers", {})
    safe_headers = {
        "Content-Type": "application/json; charset=utf-8",
        "Cache-Control": "no-store",
        "X-Content-Type-Options": "nosniff",
        "X-Api-Version": PROTOCOL_VERSION,
    }
    if isinstance(response_headers, Mapping):
        for name, value in response_headers.items():
            if isinstance(name, str) and isinstance(value, str) and "\r" not in name + value and "\n" not in name + value:
                safe_headers[name] = value
    safe_headers["Content-Length"] = str(len(payload))
    phrase = HTTPStatus(status).phrase
    start_response(f"{status} {phrase}", list(safe_headers.items()))
    return [payload]
