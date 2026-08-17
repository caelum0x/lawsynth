"""Shared WSGI test harness (mirrors the style in ``test_wsgi.py``).

Provides the temp-sqlite ``ServerSettings`` app factory, the ``request`` helper
that drives the WSGI callable, and the bearer/idempotency header builder.  The
new per-module test files reuse these so every test exercises the real
transport, not a mock.
"""

from __future__ import annotations

import io
import json
from typing import Any

from lawsynth_api import ApiSettings, create_wsgi_app
from lawsynth_server.settings import Settings as ServerSettings

TOKEN = "0123456789abcdef0123456789abcdef"
TOKEN_GLOBEX = "fedcba9876543210fedcba9876543210"


def make_app(tmp_path, *, scopes=("read", "write"), extra_tokens=None, max_bytes=1024):
    tokens = {TOKEN: ("acme", frozenset(scopes))}
    if extra_tokens:
        tokens.update(extra_tokens)
    server = ServerSettings(
        database_url=f"sqlite:///{tmp_path / 'metadata.sqlite3'}",
        object_root=tmp_path / "objects",
        tokens=tokens,
        max_upload_bytes=max_bytes,
    )
    return create_wsgi_app(ApiSettings(server=server, environment="test", max_request_bytes=max_bytes))


def request(app, method: str, path: str, *, body: object | None = None, headers: dict[str, str] | None = None, query: str = "") -> tuple[int, dict[str, str], dict[str, Any] | None]:
    raw = b"" if body is None else json.dumps(body).encode("utf-8")
    environ: dict[str, object] = {
        "REQUEST_METHOD": method,
        "PATH_INFO": path,
        "QUERY_STRING": query,
        "CONTENT_LENGTH": str(len(raw)),
        "wsgi.input": io.BytesIO(raw),
    }
    if raw:
        environ["CONTENT_TYPE"] = "application/json"
    for name, value in (headers or {}).items():
        environ[f"HTTP_{name.upper().replace('-', '_')}"] = value
    captured: dict[str, object] = {}

    def start_response(status: str, response_headers: list[tuple[str, str]]) -> None:
        captured["status"], captured["headers"] = status, dict(response_headers)

    payload = b"".join(app(environ, start_response))
    decoded = None if not payload else json.loads(payload)
    return int(str(captured["status"]).split(" ", 1)[0]), captured["headers"], decoded


def auth(*, token: str = TOKEN, key: str | None = None) -> dict[str, str]:
    result = {"Authorization": f"Bearer {token}"}
    if key:
        result["Idempotency-Key"] = key
    return result
