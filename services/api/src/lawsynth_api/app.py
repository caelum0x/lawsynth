"""A dependency-free WSGI translation layer for the LawSynth domain API."""

from __future__ import annotations

import json
from http import HTTPStatus
from typing import Callable, Iterable, Mapping, MutableMapping
from urllib.parse import parse_qsl
from uuid import uuid4

from lawsynth_server.app import Application

from .lifespan import ApiLifespan
from .settings import ApiSettings

StartResponse = Callable[[str, list[tuple[str, str]]], object]


class RequestProblem(Exception):
    def __init__(self, status: int, code: str, message: str) -> None:
        self.status, self.code, self.message = status, code, message


class WsgiApplication:
    """Translate WSGI requests to ``lawsynth_server.Application.dispatch``.

    It deliberately has no routes of its own beyond an explicit marker for the
    not-yet-deployed worker HTTP transport.  All supported operations retain
    the domain service's authentication, tenant isolation, validation, event,
    and idempotency behavior.
    """

    def __init__(self, settings: ApiSettings, *, domain: Application | None = None) -> None:
        self.settings = settings
        self._lifespan = ApiLifespan(settings.server, domain)

    def close(self) -> None:
        self._lifespan.close()

    def __call__(self, environ: MutableMapping[str, object], start_response: StartResponse) -> Iterable[bytes]:
        request_id = str(uuid4())
        try:
            request = self._request(environ)
            if request["path"].startswith("/v1/worker/") or request["path"] == "/v1/worker":
                response = self._error(501, "worker_transport_unavailable", "worker HTTP transport is not deployed by this API process", request_id)
            else:
                response = self._lifespan.application.dispatch(request)
                response.setdefault("headers", {}).setdefault("X-Request-ID", request_id)
        except RequestProblem as error:
            response = self._error(error.status, error.code, error.message, request_id)
        except RuntimeError:
            response = self._error(503, "service_unavailable", "the API process is shutting down", request_id)
        return self._respond(response, start_response)

    def _request(self, environ: Mapping[str, object]) -> dict[str, object]:
        method = environ.get("REQUEST_METHOD")
        path = environ.get("PATH_INFO")
        query = environ.get("QUERY_STRING", "")
        if not isinstance(method, str) or method not in {"GET", "POST", "PATCH", "DELETE"}:
            raise RequestProblem(405, "method_not_allowed", "only GET, POST, PATCH, and DELETE are supported")
        if not isinstance(path, str) or not path.startswith("/"):
            raise RequestProblem(400, "invalid_path", "PATH_INFO must be an absolute path")
        if "\x00" in path or "\\" in path or any(part in {".", ".."} for part in path.split("/")):
            raise RequestProblem(400, "invalid_path", "path contains a forbidden segment")
        if not isinstance(query, str):
            raise RequestProblem(400, "invalid_query", "QUERY_STRING must be text")
        try:
            pairs = parse_qsl(query, keep_blank_values=True, strict_parsing=True, max_num_fields=32)
        except ValueError as error:
            raise RequestProblem(400, "invalid_query", "query string is malformed") from error
        if len({key for key, _ in pairs}) != len(pairs):
            raise RequestProblem(400, "invalid_query", "duplicate query parameters are not supported")
        headers = self._headers(environ)
        body = self._body(environ, headers)
        return {"method": method, "path": path, "query": dict(pairs), "headers": headers, "body": body}

    @staticmethod
    def _headers(environ: Mapping[str, object]) -> dict[str, str]:
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

    def _body(self, environ: Mapping[str, object], headers: Mapping[str, str]) -> dict[str, object] | None:
        content_length = environ.get("CONTENT_LENGTH", "")
        if content_length in (None, ""):
            return None
        if not isinstance(content_length, str) or not content_length.isascii() or not content_length.isdecimal():
            raise RequestProblem(400, "invalid_content_length", "Content-Length must be a non-negative integer")
        length = int(content_length)
        if length > self.settings.max_request_bytes:
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

    @staticmethod
    def _error(status: int, code: str, message: str, request_id: str) -> dict[str, object]:
        return {
            "status": status,
            "headers": {"X-Request-ID": request_id},
            "body": {"error": {"code": code, "message": message, "request_id": request_id}},
        }

    @staticmethod
    def _respond(response: Mapping[str, object], start_response: StartResponse) -> Iterable[bytes]:
        status = response.get("status")
        if not isinstance(status, int) or status not in HTTPStatus._value2member_map_:
            status, response = 500, WsgiApplication._error(500, "internal_error", "internal server error", str(uuid4()))
        body = response.get("body", {})
        try:
            payload = b"" if status == 204 else json.dumps(body, allow_nan=False, separators=(",", ":")).encode("utf-8")
        except (TypeError, ValueError):
            status, payload = 500, json.dumps(WsgiApplication._error(500, "internal_error", "internal server error", str(uuid4()))["body"], separators=(",", ":")).encode("utf-8")
        response_headers = response.get("headers", {})
        safe_headers = {"Content-Type": "application/json; charset=utf-8", "Cache-Control": "no-store", "X-Content-Type-Options": "nosniff"}
        if isinstance(response_headers, Mapping):
            for name, value in response_headers.items():
                if isinstance(name, str) and isinstance(value, str) and "\r" not in name + value and "\n" not in name + value:
                    safe_headers[name] = value
        safe_headers["Content-Length"] = str(len(payload))
        phrase = HTTPStatus(status).phrase
        start_response(f"{status} {phrase}", list(safe_headers.items()))
        return [payload]


def create_wsgi_app(settings: ApiSettings | None = None, *, domain: Application | None = None) -> WsgiApplication:
    return WsgiApplication(settings or ApiSettings.from_environment(), domain=domain)
