"""WSGI gateway that admits requests before an in-process LawSynth API call."""

from __future__ import annotations

import io
import json
import re
import sys
import threading
import time
from collections import OrderedDict, deque
from collections.abc import Callable, Iterable, Mapping, MutableMapping
from dataclasses import dataclass
from http import HTTPStatus
from typing import Protocol
from uuid import uuid4

from .settings import GatewaySettings

StartResponse = Callable[[str, list[tuple[str, str]]], object]
WsgiApplication = Callable[[MutableMapping[str, object], StartResponse], Iterable[bytes]]
_REQUEST_ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{7,127}$")
_HEADER_NAME = re.compile(r"^[A-Za-z0-9-]+$")
_HOP_BY_HOP = frozenset({"connection", "keep-alive", "proxy-authenticate", "proxy-authorization", "te", "trailer", "transfer-encoding", "upgrade"})
_FORWARDING = frozenset({"forwarded", "x-forwarded-for", "x-forwarded-host", "x-forwarded-proto", "x-real-ip"})
_METHODS = frozenset({"GET", "POST", "PATCH", "DELETE", "OPTIONS"})


class RemoteUpstreamUnavailable(RuntimeError):
    """The gateway intentionally has no remote proxy, TLS, or retry transport."""


class Backend(Protocol):
    def __call__(self, environ: MutableMapping[str, object], start_response: StartResponse) -> Iterable[bytes]: ...


@dataclass(frozen=True, slots=True)
class Problem(Exception):
    status: int
    code: str
    message: str
    headers: Mapping[str, str] | None = None


class BoundedRateLimiter:
    """A lock-protected sliding window with a bounded client key-space."""

    def __init__(self, *, requests: int, window_seconds: float, max_clients: int, clock: Callable[[], float] = time.monotonic) -> None:
        self._requests, self._window, self._max_clients, self._clock = requests, window_seconds, max_clients, clock
        self._clients: OrderedDict[str, deque[float]] = OrderedDict()
        self._lock = threading.Lock()

    def admit(self, client: str) -> bool:
        now = self._clock()
        with self._lock:
            bucket = self._clients.get(client)
            if bucket is None:
                if len(self._clients) >= self._max_clients:
                    self._clients.popitem(last=False)
                bucket = deque()
                self._clients[client] = bucket
            else:
                self._clients.move_to_end(client)
            cutoff = now - self._window
            while bucket and bucket[0] <= cutoff:
                bucket.popleft()
            if len(bucket) >= self._requests:
                return False
            bucket.append(now)
            return True


class InProcessWsgiBackend:
    """The only production backend supported by this gateway revision.

    Network upstream URLs, proxy retries, and TLS termination are deliberately
    absent: treating an unimplemented remote path as successful is unsafe.
    Deploy a separately audited reverse proxy when those capabilities are
    needed, or call this object with the local API WSGI application.
    """

    def __init__(self, application: WsgiApplication) -> None:
        if not callable(application):
            raise TypeError("application must be a WSGI callable")
        self.application = application

    def __call__(self, environ: MutableMapping[str, object], start_response: StartResponse) -> Iterable[bytes]:
        return self.application(environ, start_response)

    @classmethod
    def remote(cls, *_: object, **__: object) -> "InProcessWsgiBackend":
        raise RemoteUpstreamUnavailable("remote upstream proxying, TLS termination, and retries are unavailable")


class GatewayApplication:
    """Canonicalize, authorize, rate-limit, and relay a request to local WSGI."""

    def __init__(self, backend: WsgiApplication, settings: GatewaySettings | None = None) -> None:
        self.settings = settings or GatewaySettings.from_environment()
        self._backend = InProcessWsgiBackend(backend)
        self._limiter = BoundedRateLimiter(
            requests=self.settings.requests_per_window,
            window_seconds=self.settings.rate_window_seconds,
            max_clients=self.settings.max_clients,
        )
        self._accepting = True
        self._state_lock = threading.Lock()

    def close(self) -> None:
        with self._state_lock:
            self._accepting = False
        close = getattr(self._backend.application, "close", None)
        if callable(close):
            close()

    def __call__(self, environ: MutableMapping[str, object], start_response: StartResponse) -> Iterable[bytes]:
        request_id = self._request_id(environ)
        origin: str | None = None
        try:
            method, path, query, headers, body, client = self._admit(environ)
            origin = headers.get("Origin")
            if path == "/healthz":
                response = self._health_response(request_id)
            elif path == "/readyz":
                response = self._ready_response(request_id)
            else:
                self._check_origin(origin)
                if method == "OPTIONS":
                    response = self._preflight(headers, request_id)
                else:
                    if not self._limiter.admit(client):
                        raise Problem(429, "rate_limited", "request rate limit exceeded", {"Retry-After": str(max(1, int(self.settings.rate_window_seconds)))})
                    response = self._invoke(method, path, query, headers, body, client, request_id)
        except Problem as problem:
            response = self._error(problem, request_id)
        except Exception:
            response = self._error(Problem(502, "backend_failure", "the backend could not serve this request"), request_id)
        return self._respond(response, start_response, request_id, origin)

    def _admit(self, environ: Mapping[str, object]) -> tuple[str, str, str, dict[str, str], bytes, str]:
        method, path = environ.get("REQUEST_METHOD"), environ.get("PATH_INFO")
        query, client = environ.get("QUERY_STRING", ""), environ.get("REMOTE_ADDR", "unknown")
        if not isinstance(method, str) or method not in _METHODS:
            raise Problem(405, "method_not_allowed", "unsupported HTTP method")
        if not isinstance(path, str) or not path.startswith("/") or "\\" in path or any(ord(char) < 32 or ord(char) == 127 for char in path) or any(part in {".", ".."} for part in path.split("/")):
            raise Problem(400, "invalid_path", "path is not an absolute, normalized path")
        if path not in {"/healthz", "/readyz"} and not (path == self.settings.api_prefix or path.startswith(self.settings.api_prefix + "/")):
            raise Problem(404, "route_not_found", "gateway only exposes the LawSynth API prefix")
        if path not in {"/healthz", "/readyz"}:
            with self._state_lock:
                if not self._accepting:
                    raise Problem(503, "gateway_draining", "gateway is not accepting requests")
        if not isinstance(query, str) or any(ord(char) < 32 or ord(char) == 127 for char in query):
            raise Problem(400, "invalid_query", "query string is invalid")
        if not isinstance(client, str) or not client or len(client) > 128 or any(not (char.isdigit() or char.isalpha() or char in ".:-") for char in client):
            client = "unknown"
        headers = self._headers(environ)
        body = self._body(environ, headers)
        return method, path, query, headers, body, client

    def _headers(self, environ: Mapping[str, object]) -> dict[str, str]:
        headers: dict[str, str] = {}
        for key, value in environ.items():
            if not isinstance(key, str) or not key.startswith("HTTP_"):
                continue
            if not isinstance(value, str):
                raise Problem(400, "invalid_header", "header values must be text")
            name = self._canonical_header(key[5:].replace("_", "-"))
            if name.lower() in _HOP_BY_HOP or name.lower() in _FORWARDING:
                continue
            if name in headers:
                raise Problem(400, "duplicate_header", "duplicate headers are not supported")
            headers[name] = self._valid_header_value(value)
        content_type = environ.get("CONTENT_TYPE")
        if content_type not in (None, ""):
            if not isinstance(content_type, str):
                raise Problem(400, "invalid_header", "Content-Type must be text")
            headers["Content-Type"] = self._valid_header_value(content_type)
        if len(headers) > self.settings.max_headers:
            raise Problem(431, "too_many_headers", "request has too many headers")
        if sum(len(name) + len(value) + 4 for name, value in headers.items()) > self.settings.max_header_bytes:
            raise Problem(431, "headers_too_large", "request headers exceed the configured limit")
        return headers

    @staticmethod
    def _canonical_header(name: str) -> str:
        if not _HEADER_NAME.fullmatch(name):
            raise Problem(400, "invalid_header", "header name is invalid")
        return "-".join(part.capitalize() for part in name.split("-"))

    @staticmethod
    def _valid_header_value(value: str) -> str:
        if any((ord(char) < 32 and char != "\t") or ord(char) == 127 or ord(char) > 255 for char in value):
            raise Problem(400, "invalid_header", "header values contain control characters")
        return value

    def _body(self, environ: Mapping[str, object], headers: Mapping[str, str]) -> bytes:
        if environ.get("HTTP_TRANSFER_ENCODING") not in (None, ""):
            raise Problem(400, "unsupported_transfer_encoding", "chunked or transformed request bodies are not supported")
        raw_length = environ.get("CONTENT_LENGTH", "")
        if raw_length in (None, ""):
            return b""
        if not isinstance(raw_length, str) or not raw_length.isascii() or not raw_length.isdecimal():
            raise Problem(400, "invalid_content_length", "Content-Length must be a non-negative integer")
        length = int(raw_length)
        if length > self.settings.max_body_bytes:
            raise Problem(413, "payload_too_large", "request body exceeds the configured limit")
        stream = environ.get("wsgi.input")
        if not hasattr(stream, "read"):
            raise Problem(400, "missing_request_body", "WSGI input stream is missing")
        body = stream.read(length + 1)
        if not isinstance(body, bytes) or len(body) != length:
            raise Problem(400, "invalid_request_body", "request body does not match Content-Length")
        if body and not headers.get("Content-Type"):
            raise Problem(415, "missing_content_type", "request body requires Content-Type")
        return body

    def _check_origin(self, origin: str | None) -> None:
        if origin is not None and origin not in self.settings.allowed_origins:
            raise Problem(403, "origin_forbidden", "origin is not allowed")

    def _preflight(self, headers: Mapping[str, str], request_id: str) -> dict[str, object]:
        origin, requested = headers.get("Origin"), headers.get("Access-Control-Request-Method")
        if origin is None or requested not in _METHODS - {"OPTIONS"}:
            raise Problem(400, "invalid_preflight", "preflight requires an allowed origin and request method")
        requested_headers = headers.get("Access-Control-Request-Headers", "")
        names = [item.strip() for item in requested_headers.split(",") if item.strip()]
        for name in names:
            self._canonical_header(name)
            if name.lower() in _HOP_BY_HOP or name.lower() in _FORWARDING:
                raise Problem(400, "invalid_preflight", "preflight requests a forbidden header")
        return {"status": 204, "headers": {"Access-Control-Allow-Methods": "GET, POST, PATCH, DELETE", "Access-Control-Allow-Headers": ", ".join(names), "Access-Control-Max-Age": "600"}, "body": b""}

    def _invoke(self, method: str, path: str, query: str, headers: Mapping[str, str], body: bytes, client: str, request_id: str) -> dict[str, object]:
        backend_environ: MutableMapping[str, object] = {
            "REQUEST_METHOD": method,
            "SCRIPT_NAME": "",
            "PATH_INFO": path,
            "QUERY_STRING": query,
            "SERVER_PROTOCOL": "HTTP/1.1",
            "REMOTE_ADDR": client,
            "wsgi.version": (1, 0),
            "wsgi.url_scheme": "http",
            "wsgi.input": io.BytesIO(body),
            "wsgi.errors": sys.stderr,
            "wsgi.multithread": True,
            "wsgi.multiprocess": False,
            "wsgi.run_once": False,
            "CONTENT_LENGTH": str(len(body)),
        }
        for name, value in headers.items():
            if name == "Content-Type":
                backend_environ["CONTENT_TYPE"] = value
            else:
                backend_environ["HTTP_" + name.upper().replace("-", "_")] = value
        backend_environ["HTTP_X_REQUEST_ID"] = request_id
        backend_environ["HTTP_X_FORWARDED_FOR"] = client
        backend_environ["HTTP_X_FORWARDED_PROTO"] = "http"
        captured: dict[str, object] = {}

        def start_response(status: str, response_headers: list[tuple[str, str]], exc_info: object = None) -> None:
            if exc_info is not None:
                raise Problem(502, "backend_failure", "backend response failed")
            if "status" in captured:
                raise Problem(502, "backend_failure", "backend invoked start_response twice")
            captured["status"], captured["headers"] = status, response_headers

        iterable = self._backend(backend_environ, start_response)
        try:
            payload = b"".join(iterable)
        finally:
            close = getattr(iterable, "close", None)
            if callable(close):
                close()
        status = captured.get("status")
        response_headers = captured.get("headers")
        if not isinstance(status, str) or not isinstance(response_headers, list):
            raise Problem(502, "backend_failure", "backend did not produce a valid WSGI response")
        try:
            status_code = int(status.split(" ", 1)[0])
            HTTPStatus(status_code)
        except (ValueError, IndexError) as error:
            raise Problem(502, "backend_failure", "backend returned an invalid status") from error
        if not all(isinstance(part, bytes) for part in [payload]):
            raise Problem(502, "backend_failure", "backend response must be bytes")
        clean_headers: dict[str, str] = {}
        for item in response_headers:
            if not isinstance(item, tuple) or len(item) != 2 or not all(isinstance(value, str) for value in item):
                raise Problem(502, "backend_failure", "backend returned invalid headers")
            name, value = item
            canonical = self._canonical_header(name)
            if canonical.lower() in _HOP_BY_HOP or canonical.lower() in {"content-length", "server"}:
                continue
            clean_headers[canonical] = self._valid_header_value(value)
        return {"status": status_code, "headers": clean_headers, "body": payload}

    @staticmethod
    def _request_id(environ: Mapping[str, object]) -> str:
        candidate = environ.get("HTTP_X_REQUEST_ID")
        return candidate if isinstance(candidate, str) and _REQUEST_ID.fullmatch(candidate) else str(uuid4())

    def _health_response(self, request_id: str) -> dict[str, object]:
        with self._state_lock:
            accepting = self._accepting
        return {"status": 200, "headers": {}, "body": {"status": "ok", "accepting": accepting, "request_id": request_id}}

    def _ready_response(self, request_id: str) -> dict[str, object]:
        with self._state_lock:
            accepting = self._accepting
        status = 200 if accepting else 503
        return {"status": status, "headers": {}, "body": {"status": "ready" if accepting else "draining", "request_id": request_id}}

    @staticmethod
    def _error(problem: Problem, request_id: str) -> dict[str, object]:
        return {"status": problem.status, "headers": dict(problem.headers or {}), "body": {"error": {"code": problem.code, "message": problem.message, "request_id": request_id}}}

    def _respond(self, response: Mapping[str, object], start_response: StartResponse, request_id: str, origin: str | None) -> Iterable[bytes]:
        status = response.get("status")
        if not isinstance(status, int) or status not in HTTPStatus._value2member_map_:
            status, response = 500, self._error(Problem(500, "internal_error", "gateway response failed"), request_id)
        body = response.get("body", b"")
        try:
            payload = body if isinstance(body, bytes) else json.dumps(body, allow_nan=False, separators=(",", ":")).encode("utf-8")
        except (TypeError, ValueError):
            status, payload = 500, b'{"error":{"code":"internal_error","message":"gateway response failed"}}'
        headers: dict[str, str] = {"Cache-Control": "no-store", "X-Content-Type-Options": "nosniff", "X-Request-Id": request_id, "Content-Length": str(len(payload))}
        response_headers = response.get("headers")
        if isinstance(response_headers, Mapping):
            for name, value in response_headers.items():
                if isinstance(name, str) and isinstance(value, str):
                    headers[self._canonical_header(name)] = self._valid_header_value(value)
        # The edge owns correlation. A backend must not replace the client-safe
        # identifier that was generated or validated during admission.
        headers["X-Request-Id"] = request_id
        if not isinstance(body, bytes):
            headers.setdefault("Content-Type", "application/json; charset=utf-8")
        if origin is not None and origin in self.settings.allowed_origins:
            headers["Access-Control-Allow-Origin"] = origin
            headers["Vary"] = "Origin"
        start_response(f"{status} {HTTPStatus(status).phrase}", list(headers.items()))
        return [payload]


def create_gateway(backend: WsgiApplication, settings: GatewaySettings | None = None) -> GatewayApplication:
    return GatewayApplication(backend, settings)
