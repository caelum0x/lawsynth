"""A dependency-free WSGI translation layer for the LawSynth domain API.

This module is the *composition root* of the transport.  It owns no request or
response mechanics of its own: header/version/body ingest lives in
:mod:`middleware` and :mod:`uploads`, authentication and scope in :mod:`auth` and
:mod:`authorization`, per-resource route classification and SSE lifecycle
projection in the resource modules (:mod:`projects`, :mod:`datasets`,
:mod:`worlds`, :mod:`runs`, :mod:`simulations`, :mod:`artifacts`), stream and
download response construction in :mod:`downloads`, and typed access to the
domain in :mod:`storage`, :mod:`database`, and :mod:`repositories`.  ``app.py``
only decides *which* collaborator handles a request and delegates.
"""

from __future__ import annotations

import time
from typing import Callable, Iterable, Mapping, MutableMapping
from urllib.parse import parse_qsl
from uuid import uuid4

from lawsynth_server.app import Application
from lawsynth_server.errors import ServerError

from . import artifacts, datasets, downloads, products, projects, runs, uploads, worlds
from .auth import ApiAuthenticator
from .discovery import DiscoveryService
from .authorization import READ, WRITE, require_scope_or_problem
from .database import ApiDatabase
from .events import EventBus
from .lifespan import ApiLifespan
from .metering import MeteringLog
from .quota import QuotaGuard
from .tenancy import ArtifactOwnership
from .middleware import (
    RequestProblem,
    error_envelope,
    extract_headers,
    finalize_response,
    negotiate_api_version,
)
from .repositories import ApiRepositories
from .settings import ApiSettings
from .storage import ApiStorage
from .telemetry import RequestTelemetry

StartResponse = Callable[[str, list[tuple[str, str]]], object]

# Public resource segments that carry a route classifier and an SSE lifecycle
# projection.  The dispatch loop consults this registry for telemetry labels and
# for translating a successful mutation into streamed events.
_RESOURCES = {module.SEGMENT: module for module in (projects, datasets, worlds, runs, artifacts)}


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
        self._events = EventBus(retention=settings.event_stream_retention)
        self._telemetry = RequestTelemetry()
        self._metering = MeteringLog()
        self._artifacts = ArtifactOwnership()
        self._quota = QuotaGuard(settings.quota)
        services = self._lifespan.application.services
        self._auth = ApiAuthenticator(services.auth)
        self._storage = ApiStorage(services.storage)
        self._database = ApiDatabase(services.database)
        self._repositories = ApiRepositories(services)
        self._discovery = DiscoveryService(services, self._events, self._quota, self._metering)

    def close(self) -> None:
        self._discovery.close()
        self._lifespan.close()

    @property
    def telemetry(self) -> RequestTelemetry:
        return self._telemetry

    def readiness(self) -> dict[str, object]:
        """Report process readiness from the typed domain accessors.

        This is internal introspection (not an HTTP route): it probes the
        metadata connection and the object root through the same facades the
        transport uses, and folds in the request telemetry snapshot.
        """

        return {
            "database": self._database.ping(),
            "storage": self._storage.ensure_root(),
            "resources": list(self._repositories.segments()),
            "telemetry": self._telemetry.snapshot(),
        }

    def __call__(self, environ: MutableMapping[str, object], start_response: StartResponse) -> Iterable[bytes]:
        request_id = str(uuid4())
        request: dict[str, object] | None = None
        try:
            request = self._request(environ)
            if request["method"] == "GET" and request["path"] == "/v1/events" and downloads.wants_sse(request["headers"]):
                return self._stream_events(request, request_id, start_response)
            parts = self._parts(str(request["path"]))
            product = products.match(str(request["method"]), parts)
            if product == "report":
                return self._serve_report(request, parts, request_id, start_response)
            if product is not None:
                response = self._handle_product(request, product, parts, request_id)
            elif self._is_usage(request, parts):
                response = self._handle_usage(request, request_id)
            elif self._is_discovery_submit(request, parts):
                response = self._handle_discovery_submit(request, request_id)
            elif self._is_run_world(request, parts):
                response = self._handle_run_world(request, parts, request_id)
            elif self._is_artifact_download(request, parts):
                response = self._handle_artifact_download(request, parts, request_id)
            elif request["path"].startswith("/v1/worker/") or request["path"] == "/v1/worker":
                response = error_envelope(501, "worker_transport_unavailable", "worker HTTP transport is not deployed by this API process", request_id)
            else:
                response = self._lifespan.application.dispatch(request)
                response.setdefault("headers", {}).setdefault("X-Request-ID", request_id)
                self._emit_lifecycle(request, response)
                self._decorate_download(request, response)
                self._record_artifact_ownership(request, response)
        except RequestProblem as error:
            response = error_envelope(error.status, error.code, error.message, request_id)
        except RuntimeError:
            response = error_envelope(503, "service_unavailable", "the API process is shutting down", request_id)
        self._record(request, response)
        return finalize_response(response, start_response)

    # -- Server-Sent Events -------------------------------------------------
    #
    # Delivery semantics (WSGI is synchronous, so there is no server push):
    # each ``GET /v1/events`` call authenticates, resolves the caller's scope
    # (tenant/organization from the bearer token), then drains and returns, as
    # framed SSE, every currently-retained event for that scope whose sequence
    # is greater than the ``Last-Event-ID`` request header (0 when absent).
    # The stream is then closed -- the connection is NOT held open for future
    # events.  Clients resume with cursor-based polling: reconnect and send the
    # id of the last event they received via ``Last-Event-ID``.  Retention is
    # bounded and in-process (see ``events.EventBus``); events evicted from the
    # ring buffer are not replayable.

    def _stream_events(self, request: Mapping[str, object], request_id: str, start_response: StartResponse) -> Iterable[bytes]:
        headers = request["headers"]
        principal = self._auth.authenticate_or_problem(headers)
        require_scope_or_problem(principal, READ)
        after = downloads.last_event_id(headers)
        events = self._events.events_after(principal.organization_id, after)
        return downloads.open_stream(events, request_id, start_response)

    def _emit_lifecycle(self, request: Mapping[str, object], response: Mapping[str, object]) -> None:
        """Translate a successful run/artifact mutation into a streamed ApiEvent.

        Emission happens at the API boundary from the domain's own outcome, so
        no run/artifact state is duplicated.  Idempotent replays are skipped to
        avoid double-emitting an event for a single logical change.  The
        per-resource projection lives in each resource module.
        """

        status = response.get("status")
        if not isinstance(status, int) or status >= 300:
            return
        response_headers = response.get("headers", {})
        if isinstance(response_headers, Mapping) and response_headers.get("Idempotency-Replayed") == "true":
            return
        body = response.get("body")
        if not isinstance(body, Mapping):
            return
        segment = self._segment(str(request["path"]))
        module = _RESOURCES.get(segment)
        if module is None:
            return
        principal = self._auth.silent(request["headers"])
        if principal is None:
            return
        scope = principal.organization_id
        now = int(time.time() * 1000)
        for kind, payload, run_id in module.lifecycle_events(str(request["method"]), body):
            self._events.append(scope, now, kind, payload, run_id=run_id)

    # -- Product features ---------------------------------------------------
    #
    # The product surface (explain/forecast/report/compare) is composed at this
    # boundary rather than in the domain dispatcher: it reuses the domain's
    # world repository and native engine, but its request/response shapes are a
    # transport concern.  Every handler authenticates and scopes exactly like
    # the rest of the API, translates a domain :class:`ServerError` into the
    # shared error envelope, and returns honest capability boundaries (404 for
    # an unknown world, 422 for bad input, 503 when the native engine backs an
    # operation and is absent).

    # explain/report/compare read only declarative structure; forecast runs the
    # native simulator, so it requires the same write scope as ``/simulate``.
    _PRODUCT_SCOPES = {"explain": READ, "forecast": WRITE, "report": READ, "compare": READ}

    def _handle_product(self, request: Mapping[str, object], action: str, parts: list[str], request_id: str) -> dict[str, object]:
        """Authenticate, scope, and run a JSON product action (not the report)."""

        principal = self._auth.authenticate_or_problem(request["headers"])
        require_scope_or_problem(principal, self._PRODUCT_SCOPES[action])
        worlds_repo = self._repositories.get("worlds")
        try:
            if action == "compare":
                left_id, right_id = products.compare_refs(request.get("body"))
                left = worlds_repo.get(principal.organization_id, left_id)
                right = worlds_repo.get(principal.organization_id, right_id)
                body = products.compare(left, right)
            else:
                world = worlds_repo.get(principal.organization_id, parts[1])
                body = products.explain(world) if action == "explain" else products.forecast(world, request.get("body"))
        except ServerError as error:
            return error_envelope(error.status_code, error.code, error.message, request_id)
        return {"status": 200, "headers": {"X-Request-ID": request_id}, "body": body}

    # -- Discovery-as-a-service ---------------------------------------------
    #
    # The discovery run workflow is composed at this boundary rather than in the
    # domain dispatcher: submitting a run creates a domain run record, runs the
    # native engine in-process (queued -> running -> succeeded/failed on a worker
    # thread), stores the resulting world through the domain, and emits the
    # run-lifecycle ``ApiEvent``s through the same bus that backs ``/v1/events``.
    # A discovery submit is a ``POST /v1/runs`` that names a dataset (an uploaded
    # ``dataset_id`` or an inline ``dataset``); a plain run create -- no dataset
    # reference -- still flows through the domain dispatcher unchanged.

    @staticmethod
    def _is_discovery_submit(request: Mapping[str, object], parts: list[str]) -> bool:
        if request["method"] != "POST" or parts != ["runs"]:
            return False
        body = request.get("body")
        return isinstance(body, Mapping) and ("dataset_id" in body or "dataset" in body)

    @staticmethod
    def _is_run_world(request: Mapping[str, object], parts: list[str]) -> bool:
        return request["method"] == "GET" and len(parts) == 3 and parts[0] == "runs" and parts[2] == "world"

    def _handle_discovery_submit(self, request: Mapping[str, object], request_id: str) -> dict[str, object]:
        """Authenticate, scope, and submit a discovery run (honest 503 if native absent)."""

        headers = request["headers"]
        principal = self._auth.authenticate_or_problem(headers)
        require_scope_or_problem(principal, WRITE)
        key = headers.get("Idempotency-Key")
        if not isinstance(key, str) or not key:
            return error_envelope(422, "validation_error", "Idempotency-Key is required for writes", request_id)
        try:
            return self._discovery.submit(principal.organization_id, request.get("body"), key, request_id)
        except ServerError as error:
            return error_envelope(error.status_code, error.code, error.message, request_id)

    def _handle_run_world(self, request: Mapping[str, object], parts: list[str], request_id: str) -> dict[str, object]:
        """Authenticate, scope, and return the world a completed run discovered."""

        principal = self._auth.authenticate_or_problem(request["headers"])
        require_scope_or_problem(principal, READ)
        try:
            return self._discovery.run_world(principal.organization_id, parts[1], request_id)
        except ServerError as error:
            return error_envelope(error.status_code, error.code, error.message, request_id)

    # -- Metering & usage reporting ----------------------------------------
    #
    # The metering log is an append-only, tenant-partitioned record of billable
    # actions.  ``GET /v1/usage`` is its read surface: it authenticates, scopes
    # to the caller's tenant, and returns only that tenant's usage -- never
    # another's -- so a hosted deployment can bill from the same isolation
    # boundary a token grants.

    @staticmethod
    def _is_usage(request: Mapping[str, object], parts: list[str]) -> bool:
        return request["method"] == "GET" and parts == ["usage"]

    def _handle_usage(self, request: Mapping[str, object], request_id: str) -> dict[str, object]:
        principal = self._auth.authenticate_or_problem(request["headers"])
        require_scope_or_problem(principal, READ)
        return {
            "status": 200,
            "headers": {"X-Request-ID": request_id},
            "body": self._metering.usage(principal.organization_id),
        }

    # -- Artifact download (tenant-scoped over content-addressed storage) ---
    #
    # Artifacts are stored content-addressed (dedup by SHA-256), but a hash is
    # not a grant: a download MUST be authorized against the caller's tenant.
    # The API records ownership on a successful upload and checks it here before
    # delegating the actual byte-serving to the domain, so a tenant can never
    # download a hash it did not store -- even one it somehow learned.

    @staticmethod
    def _is_artifact_download(request: Mapping[str, object], parts: list[str]) -> bool:
        return request["method"] == "GET" and len(parts) == 2 and parts[0] == "artifacts"

    def _handle_artifact_download(self, request: Mapping[str, object], parts: list[str], request_id: str) -> dict[str, object]:
        principal = self._auth.authenticate_or_problem(request["headers"])
        require_scope_or_problem(principal, READ)
        # A malformed identifier is a request-shape error the domain reports as a
        # 422; only well-formed hashes are subject to the ownership gate, so a
        # tenant can never download a valid hash it did not store.
        if self._is_sha256(parts[1]) and not self._artifacts.owns(principal.organization_id, parts[1]):
            return error_envelope(404, "not_found", "artifact not found", request_id)
        response = self._lifespan.application.dispatch(request)
        response.setdefault("headers", {}).setdefault("X-Request-ID", request_id)
        self._decorate_download(request, response)
        return response

    @staticmethod
    def _is_sha256(value: str) -> bool:
        return len(value) == 64 and all(character in "0123456789abcdef" for character in value)

    def _record_artifact_ownership(self, request: Mapping[str, object], response: Mapping[str, object]) -> None:
        """Grant the caller's tenant ownership of a just-stored artifact hash.

        Runs after a successful ``POST /v1/artifacts`` (including idempotent
        replays, which re-grant the same hash harmlessly).  Never raises: an
        ownership-recording failure must not change the upload's outcome.
        """

        try:
            if request["method"] != "POST" or self._parts(str(request["path"])) != ["artifacts"]:
                return
            if response.get("status") != 201:
                return
            body = response.get("body")
            if not isinstance(body, Mapping):
                return
            sha256 = body.get("sha256")
            principal = self._auth.silent(request["headers"])
            if principal is None or not isinstance(sha256, str):
                return
            self._artifacts.grant(principal.organization_id, sha256)
        except Exception:
            pass

    def _serve_report(self, request: Mapping[str, object], parts: list[str], request_id: str, start_response: StartResponse) -> Iterable[bytes]:
        """Serve ``GET /v1/worlds/{id}/report`` as a self-contained HTML document.

        HTML cannot flow through the JSON finalizer, so this path frames its own
        response.  Auth/scope/domain failures still render the shared JSON error
        envelope so error semantics stay uniform across the surface.
        """

        try:
            principal = self._auth.authenticate_or_problem(request["headers"])
            require_scope_or_problem(principal, READ)
            world = self._repositories.get("worlds").get(principal.organization_id, parts[1])
            document = products.report_html(world)
        except RequestProblem as error:
            return finalize_response(error_envelope(error.status, error.code, error.message, request_id), start_response)
        except ServerError as error:
            return finalize_response(error_envelope(error.status_code, error.code, error.message, request_id), start_response)
        return downloads.open_html(document, request_id, start_response)

    def _decorate_download(self, request: Mapping[str, object], response: MutableMapping[str, object]) -> None:
        """Add content-revalidation headers to a served artifact download.

        Purely additive: never mutates the JSON body or an existing header.
        """

        if response.get("status") != 200:
            return
        if self._route_label(str(request["method"]), str(request["path"])) != "artifacts.download":
            return
        extra = downloads.artifact_download_headers(response.get("body"))
        if not extra:
            return
        headers = response.setdefault("headers", {})
        if isinstance(headers, dict):
            for name, value in extra.items():
                headers.setdefault(name, value)

    # -- Telemetry ----------------------------------------------------------

    def _record(self, request: Mapping[str, object] | None, response: Mapping[str, object]) -> None:
        """Record one completed request; never let telemetry break a response."""

        try:
            label = "malformed" if request is None else self._route_label(str(request["method"]), str(request["path"]))
            status = response.get("status")
            self._telemetry.record(label, status if isinstance(status, int) else 0)
        except Exception:
            pass

    def _route_label(self, method: str, path: str) -> str:
        """Derive a stable telemetry label from a method and path."""

        parts = self._parts(path)
        if not parts:
            return "root"
        segment = parts[0]
        if segment in {"health", "version", "events", "worker", "usage"}:
            return segment
        module = _RESOURCES.get(segment)
        if module is None:
            return "unknown"
        try:
            return module.classify(method, parts)
        except Exception:
            return "unknown"

    @staticmethod
    def _parts(path: str) -> list[str]:
        parts = [part for part in path.split("/") if part]
        if parts[:1] == ["v1"]:
            parts = parts[1:]
        return parts

    @classmethod
    def _segment(cls, path: str) -> str | None:
        parts = cls._parts(path)
        return parts[0] if parts else None

    # -- Request ingest -----------------------------------------------------

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
        headers = extract_headers(environ)
        negotiate_api_version(headers)
        body = uploads.parse_json_body(environ, headers, self.settings.max_request_bytes)
        return {"method": method, "path": path, "query": dict(pairs), "headers": headers, "body": body}


def create_wsgi_app(settings: ApiSettings | None = None, *, domain: Application | None = None) -> WsgiApplication:
    return WsgiApplication(settings or ApiSettings.from_environment(), domain=domain)
