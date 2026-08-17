"""A dependency-free client for a running LawSynth API service.

``Client`` drives the *discovery-as-a-service* product loop from Python: submit
a discovery run against an uploaded (or inline) dataset, poll it to completion,
fetch and explain the discovered world, and render a self-contained report ---
all over the service's ``/v1`` HTTP contract (bearer auth, ``X-Api-Version: 1``,
and the ``{"error": {code, message, request_id}}`` envelope).

Two transports back the exact same client so the whole loop is testable offline:

* :class:`_HttpTransport` speaks real HTTP over :mod:`urllib` --- ``Client("http://host:8080", token=...)``.
* :class:`_WsgiTransport` calls a WSGI application object in-process (building the
  ``environ`` and capturing ``start_response``) --- ``Client(wsgi_app=create_wsgi_app(...))``.
  No socket is opened, so the example and tests run fully offline and deterministically.

Only the standard library is used; the native engine is never touched by the
client (discovery runs on the *server* side).
"""

from __future__ import annotations

import csv as _csv
import io
import json
from dataclasses import dataclass
from os import PathLike
from pathlib import Path
from typing import Callable, Iterable, Mapping, Sequence
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode, urlsplit
from urllib.request import Request as _UrlRequest
from urllib.request import urlopen
from uuid import uuid4

from .errors import ApiError, RunTimeout, ValidationError

__all__ = ["Client", "Run"]

_TERMINAL_STATUSES = frozenset({"succeeded", "failed", "cancelled"})
_WRITE_METHODS = frozenset({"POST", "PATCH", "DELETE"})


# --------------------------------------------------------------------------- #
# Run — the client-side view of a submitted discovery run                      #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class Run:
    """An immutable snapshot of a discovery run as reported by the service.

    ``summary`` folds in the result fields the service surfaces once a run is
    done (``mse``, ``complexity``, ``laws``, ``world_id``); fields the running
    API has not yet populated are simply absent. ``raw`` is the untouched run
    record for forward compatibility.
    """

    id: str
    status: str
    world_id: str | None
    name: str | None
    dataset_id: str | None
    summary: Mapping[str, object]
    raw: Mapping[str, object]

    @property
    def terminal(self) -> bool:
        return self.status in _TERMINAL_STATUSES

    @property
    def succeeded(self) -> bool:
        return self.status == "succeeded"

    @classmethod
    def from_payload(cls, payload: Mapping[str, object]) -> "Run":
        if not isinstance(payload, Mapping) or not isinstance(payload.get("id"), str):
            raise ApiError(
                "run response did not contain an id", status=502, code="invalid_response"
            )
        summary: dict[str, object] = {}
        for key in ("mse", "complexity", "laws", "law_count", "world_id"):
            if key in payload and payload[key] is not None:
                summary[key] = payload[key]
        # The service may nest the result under "result" or "summary".
        for container in ("result", "summary"):
            nested = payload.get(container)
            if isinstance(nested, Mapping):
                summary.update({k: v for k, v in nested.items() if v is not None})
        return cls(
            id=str(payload["id"]),
            status=str(payload.get("status", "unknown")),
            world_id=payload.get("world_id") if isinstance(payload.get("world_id"), str) else None,
            name=payload.get("name") if isinstance(payload.get("name"), str) else None,
            dataset_id=payload.get("dataset_id") if isinstance(payload.get("dataset_id"), str) else None,
            summary=summary,
            raw=dict(payload),
        )


# --------------------------------------------------------------------------- #
# Transports — one JSON contract, two ways to reach the app                    #
# --------------------------------------------------------------------------- #


class _Transport:
    """Send a request and return ``(status, headers, body_bytes)``."""

    def request(
        self, method: str, path: str, headers: Mapping[str, str], body: bytes
    ) -> tuple[int, dict[str, str], bytes]:  # pragma: no cover - interface
        raise NotImplementedError


class _WsgiTransport(_Transport):
    """Drive a WSGI application object in-process --- no socket required."""

    def __init__(self, app: Callable[[dict, Callable], Iterable[bytes]]) -> None:
        self._app = app

    def request(
        self, method: str, path: str, headers: Mapping[str, str], body: bytes
    ) -> tuple[int, dict[str, str], bytes]:
        split = urlsplit(path)
        environ: dict[str, object] = {
            "REQUEST_METHOD": method,
            "PATH_INFO": split.path,
            "QUERY_STRING": split.query,
            "SERVER_NAME": "in-process",
            "SERVER_PORT": "80",
            "SERVER_PROTOCOL": "HTTP/1.1",
            "wsgi.version": (1, 0),
            "wsgi.url_scheme": "http",
            "wsgi.input": io.BytesIO(body),
            "wsgi.errors": io.StringIO(),
            "wsgi.multithread": False,
            "wsgi.multiprocess": False,
            "wsgi.run_once": False,
            "CONTENT_LENGTH": str(len(body)),
        }
        for name, value in headers.items():
            if name.lower() == "content-type":
                environ["CONTENT_TYPE"] = value
            else:
                environ[f"HTTP_{name.upper().replace('-', '_')}"] = value
        captured: dict[str, object] = {}

        def start_response(status: str, response_headers: list[tuple[str, str]]) -> None:
            captured["status"] = status
            captured["headers"] = dict(response_headers)

        chunks = self._app(environ, start_response)
        try:
            payload = b"".join(chunks)
        finally:
            close = getattr(chunks, "close", None)
            if callable(close):  # honor the WSGI iterable close protocol
                close()
        status_line = str(captured.get("status", "500 Internal Server Error"))
        status_code = int(status_line.split(" ", 1)[0])
        return status_code, dict(captured.get("headers", {})), payload  # type: ignore[arg-type]


class _HttpTransport(_Transport):
    """Speak real HTTP to a remote LawSynth service over ``urllib``."""

    def __init__(self, base_url: str, *, timeout: float) -> None:
        self._base = base_url.rstrip("/")
        self._timeout = timeout

    def request(
        self, method: str, path: str, headers: Mapping[str, str], body: bytes
    ) -> tuple[int, dict[str, str], bytes]:
        url = f"{self._base}{path}"
        data = body if body else None
        req = _UrlRequest(url, data=data, method=method)
        for name, value in headers.items():
            req.add_header(name, value)
        try:
            with urlopen(req, timeout=self._timeout) as response:  # noqa: S310 - user-provided base URL
                return response.status, dict(response.headers.items()), response.read()
        except HTTPError as error:  # 4xx/5xx still carry a JSON envelope
            payload = error.read()
            return error.code, dict(error.headers.items() if error.headers else {}), payload
        except URLError as error:
            raise ApiError(
                f"could not reach LawSynth service at {url}: {error.reason}",
                status=0,
                code="connection_error",
            ) from error


# --------------------------------------------------------------------------- #
# Client                                                                        #
# --------------------------------------------------------------------------- #


class Client:
    """A stdlib client for a running LawSynth API (discovery-as-a-service).

    Construct it against a live service::

        client = lawsynth.Client("http://localhost:8080", token="…")

    or, for fully offline use, against the API's in-process WSGI app::

        client = lawsynth.Client(wsgi_app=create_wsgi_app(settings))

    Then drive the remote product loop::

        run = client.submit_discovery(csv="obs.csv", time="t", state=["x", "y"], preset="ecology")
        run = client.wait(run)                 # poll until terminal
        world = client.world(run)              # the discovered world
        print(client.explain(run.world_id))    # plain-language explanation
        client.report(run.world_id, "out.html")
    """

    def __init__(
        self,
        base_url: str | None = None,
        *,
        token: str | None = None,
        wsgi_app: Callable[[dict, Callable], Iterable[bytes]] | None = None,
        api_version: str = "1",
        max_poll_attempts: int = 20,
        timeout: float = 30.0,
    ) -> None:
        if (base_url is None) == (wsgi_app is None):
            raise ValidationError("provide exactly one of base_url or wsgi_app")
        if max_poll_attempts < 1:
            raise ValidationError("max_poll_attempts must be positive")
        self._transport: _Transport = (
            _WsgiTransport(wsgi_app)
            if wsgi_app is not None
            else _HttpTransport(str(base_url), timeout=timeout)
        )
        self._token = token
        self._api_version = api_version
        self._max_poll_attempts = max_poll_attempts

    # -- low-level request helpers ----------------------------------------- #

    def _headers(self, *, json_body: bool, idempotency_key: str | None) -> dict[str, str]:
        headers = {"X-Api-Version": self._api_version, "Accept": "application/json"}
        if self._token:
            headers["Authorization"] = f"Bearer {self._token}"
        if json_body:
            headers["Content-Type"] = "application/json"
        if idempotency_key:
            headers["Idempotency-Key"] = idempotency_key
        return headers

    def _call(
        self,
        method: str,
        path: str,
        *,
        body: object | None = None,
        query: Mapping[str, str] | None = None,
        idempotency_key: str | None = None,
    ) -> tuple[int, dict[str, str], bytes]:
        if query:
            path = f"{path}?{urlencode(query)}"
        if method in _WRITE_METHODS and idempotency_key is None and body is not None:
            idempotency_key = uuid4().hex
        raw = b"" if body is None else json.dumps(body).encode("utf-8")
        headers = self._headers(json_body=body is not None, idempotency_key=idempotency_key)
        return self._transport.request(method, path, headers, raw)

    def _request_json(
        self,
        method: str,
        path: str,
        *,
        body: object | None = None,
        query: Mapping[str, str] | None = None,
        idempotency_key: str | None = None,
    ) -> dict[str, object]:
        status, _, payload = self._call(
            method, path, body=body, query=query, idempotency_key=idempotency_key
        )
        parsed = self._decode_json(payload) if payload else {}
        if status >= 400:
            raise self._as_error(status, parsed)
        if not isinstance(parsed, dict):
            raise ApiError(
                f"expected a JSON object from {path}", status=502, code="invalid_response"
            )
        return parsed

    @staticmethod
    def _decode_json(payload: bytes) -> object:
        try:
            return json.loads(payload.decode("utf-8"))
        except (ValueError, UnicodeDecodeError) as error:
            raise ApiError(
                "service returned a malformed JSON response",
                status=502,
                code="invalid_response",
            ) from error

    @staticmethod
    def _as_error(status: int, parsed: object) -> ApiError:
        code, message, request_id = "error", "request failed", None
        if isinstance(parsed, Mapping):
            envelope = parsed.get("error")
            if isinstance(envelope, Mapping):
                code = str(envelope.get("code", code))
                message = str(envelope.get("message", message))
                rid = envelope.get("request_id")
                request_id = str(rid) if isinstance(rid, str) else None
        return ApiError(message, status=status, code=code, request_id=request_id)

    # -- service metadata --------------------------------------------------- #

    def version(self) -> dict[str, object]:
        """The service protocol/version banner (``GET /v1/version``)."""
        return self._request_json("GET", "/v1/version")

    def health(self) -> dict[str, object]:
        """The service health probe (``GET /v1/health``)."""
        return self._request_json("GET", "/v1/health")

    # -- datasets ----------------------------------------------------------- #

    def upload_dataset(
        self,
        *,
        time: Sequence[float],
        columns: Mapping[str, Sequence[float]],
        name: str | None = None,
    ) -> str:
        """Upload numeric observations and return the new dataset id.

        ``columns`` maps every state/observable name to a series aligned with
        ``time``; the dataset ``schema`` is exactly those column names.
        """
        schema = list(columns)
        if not schema:
            raise ValidationError("a dataset needs at least one column")
        body: dict[str, object] = {
            "name": name or "dataset",
            "schema": schema,
            "time": [float(value) for value in time],
            "columns": {key: [float(v) for v in series] for key, series in columns.items()},
        }
        created = self._request_json("POST", "/v1/datasets", body=body)
        dataset_id = created.get("id")
        if not isinstance(dataset_id, str):
            raise ApiError(
                "dataset upload did not return an id", status=502, code="invalid_response"
            )
        return dataset_id

    # -- discovery runs ----------------------------------------------------- #

    def submit_discovery(
        self,
        *,
        state: Sequence[str],
        csv: str | PathLike[str] | None = None,
        columns: Mapping[str, Sequence[float]] | None = None,
        time: str | Sequence[float] = "t",
        dataset_id: str | None = None,
        preset: str | None = None,
        degree: int | None = None,
        threshold: float | None = None,
        solver: str | None = None,
        include_trigonometric: bool | None = None,
        include_rational: bool | None = None,
        name: str = "discovery",
        world_name: str | None = None,
        idempotency_key: str | None = None,
    ) -> Run:
        """Submit a discovery run and return its initial :class:`Run` status.

        The dataset is referenced one of three ways (exactly one required):
        an existing ``dataset_id``, an inline ``columns``/``time`` pair, or a
        ``csv`` source (path or literal CSV text) whose ``time`` column and
        ``state`` columns are read out. ``preset`` selects a curated discovery
        recipe (``ecology``/``mechanics``/``epidemiology``/…); ``degree``,
        ``threshold``, ``solver`` and the feature toggles layer on top of it and
        always win.
        """
        states = list(state)
        if not states:
            raise ValidationError("at least one state column is required")
        resolved_dataset = self._resolve_dataset(
            dataset_id=dataset_id, csv=csv, columns=columns, time=time, state=states, name=name
        )
        discovery = self._build_discovery(
            preset=preset,
            degree=degree,
            threshold=threshold,
            solver=solver,
            include_trigonometric=include_trigonometric,
            include_rational=include_rational,
        )
        body: dict[str, object] = {
            "name": name,
            "dataset_id": resolved_dataset,
            "states": states,
            "discovery": discovery,
        }
        if world_name is not None:
            body["world_name"] = world_name
        created = self._request_json(
            "POST", "/v1/runs", body=body, idempotency_key=idempotency_key
        )
        return Run.from_payload(created)

    def _resolve_dataset(
        self,
        *,
        dataset_id: str | None,
        csv: str | PathLike[str] | None,
        columns: Mapping[str, Sequence[float]] | None,
        time: str | Sequence[float],
        state: Sequence[str],
        name: str,
    ) -> str:
        sources = [src for src in (dataset_id, csv, columns) if src is not None]
        if len(sources) != 1:
            raise ValidationError("provide exactly one of dataset_id, csv, or columns")
        if dataset_id is not None:
            return dataset_id
        if columns is not None:
            if isinstance(time, str):
                raise ValidationError("inline columns require a numeric 'time' sequence")
            return self.upload_dataset(time=time, columns=columns, name=name)
        # CSV source: 'time' names the time column, 'state' the observable columns.
        if not isinstance(time, str):
            raise ValidationError("csv source requires the 'time' column name as a string")
        parsed_time, parsed_columns = _read_csv(csv, time_column=time, state_columns=state)
        return self.upload_dataset(time=parsed_time, columns=parsed_columns, name=name)

    @staticmethod
    def _build_discovery(
        *,
        preset: str | None,
        degree: int | None,
        threshold: float | None,
        solver: str | None,
        include_trigonometric: bool | None,
        include_rational: bool | None,
    ) -> dict[str, object]:
        """Resolve a preset (client-side) and layer explicit knobs on top.

        Presets are resolved through :mod:`lawsynth.recipes` into the concrete
        discovery options the service accepts, so ``preset=`` works against a
        service that only understands raw discovery knobs.
        """
        options: dict[str, object] = {}
        if preset is not None:
            from . import recipes

            config = recipes.get(preset).config()
            options = {
                "polynomial_degree": config.polynomial_degree,
                "threshold": config.threshold,
                "solver": config.solver,
                "derivative_method": config.derivative_method,
            }
            if config.include_trigonometric:
                options["include_trigonometric"] = True
            if config.include_rational:
                options["include_rational"] = True
        if degree is not None:
            options["polynomial_degree"] = degree
        if threshold is not None:
            options["threshold"] = threshold
        if solver is not None:
            options["solver"] = solver
        if include_trigonometric is not None:
            options["include_trigonometric"] = include_trigonometric
        if include_rational is not None:
            options["include_rational"] = include_rational
        return options

    def get_run(self, run_id: str) -> Run:
        """Fetch the current status/summary of a run (``GET /v1/runs/{id}``)."""
        return Run.from_payload(self._request_json("GET", f"/v1/runs/{run_id}"))

    def wait(self, run: Run | str) -> Run:
        """Poll a run until it reaches a terminal status; return the final run.

        Deterministic and sleep-free: the service runs discovery synchronously
        (or quickly), so a bounded number of polls suffices. Raises
        :class:`~lawsynth.errors.RunTimeout` if the bound is exhausted first.
        """
        run_id = run.id if isinstance(run, Run) else str(run)
        current = run if isinstance(run, Run) else self.get_run(run_id)
        attempts = 0
        while not current.terminal and attempts < self._max_poll_attempts:
            current = self.get_run(run_id)
            attempts += 1
        if not current.terminal:
            raise RunTimeout(run_id, status=current.status, attempts=attempts)
        return current

    # -- worlds ------------------------------------------------------------- #

    def world(self, run: Run | str) -> dict[str, object]:
        """The world discovered by a run.

        Prefers ``GET /v1/runs/{id}/world`` and gracefully falls back to
        ``GET /v1/worlds/{world_id}`` when the run-scoped endpoint is not yet
        deployed. Both are normalized to the flat world record: the run-scoped
        endpoint may wrap it in a ``{"run_id", "world_id", "world", "links"}``
        envelope, which is unwrapped here.
        """
        run_id = run.id if isinstance(run, Run) else str(run)
        try:
            return self._unwrap_world(self._request_json("GET", f"/v1/runs/{run_id}/world"))
        except ApiError as error:
            if error.status != 404:
                raise
        record = run if isinstance(run, Run) else self.get_run(run_id)
        if record.world_id is None:
            raise ApiError(
                f"run {run_id!r} has no discovered world yet",
                status=409,
                code="world_unavailable",
            )
        return self.get_world(record.world_id)

    @staticmethod
    def _unwrap_world(payload: dict[str, object]) -> dict[str, object]:
        """Return the flat world record from either a flat or ``{world: …}`` envelope."""
        inner = payload.get("world")
        if isinstance(inner, dict) and "equations" in inner:
            return inner
        return payload

    def get_world(self, world_id: str) -> dict[str, object]:
        """Fetch a discovered world record (``GET /v1/worlds/{id}``)."""
        return self._request_json("GET", f"/v1/worlds/{world_id}")

    def explain(self, world_id: str) -> dict[str, object]:
        """Plain-language explanation of a world (``GET /v1/worlds/{id}/explain``)."""
        return self._request_json("GET", f"/v1/worlds/{world_id}/explain")

    def forecast(
        self,
        world_id: str,
        *,
        initial: Mapping[str, float],
        horizon: float,
        step: float,
        start: float = 0.0,
        parameters: Mapping[str, float] | None = None,
        inputs: Mapping[str, float] | None = None,
        interventions: Sequence[Mapping[str, object]] | None = None,
    ) -> dict[str, object]:
        """Run a native forecast (``POST /v1/worlds/{id}/forecast``)."""
        body: dict[str, object] = {
            "initial": {k: float(v) for k, v in initial.items()},
            "horizon": float(horizon),
            "step": float(step),
            "start": float(start),
        }
        if parameters:
            body["parameters"] = {k: float(v) for k, v in parameters.items()}
        if inputs:
            body["inputs"] = {k: float(v) for k, v in inputs.items()}
        if interventions:
            body["interventions"] = list(interventions)
        return self._request_json("POST", f"/v1/worlds/{world_id}/forecast", body=body)

    def compare(self, left_id: str, right_id: str) -> dict[str, object]:
        """Structured diff of two worlds (``POST /v1/worlds/compare``)."""
        return self._request_json(
            "POST", "/v1/worlds/compare", body={"left": left_id, "right": right_id}
        )

    def report(self, world_id: str, path: str | PathLike[str]) -> Path:
        """Write a world's self-contained HTML report to ``path``.

        The report endpoint returns ``text/html`` on success; a failure still
        carries the shared JSON error envelope, which is decoded and raised.
        """
        status, headers, payload = self._call("GET", f"/v1/worlds/{world_id}/report")
        if status >= 400:
            raise self._as_error(status, self._decode_json(payload) if payload else {})
        target = Path(path)
        if target.suffix.lower() not in {".html", ".htm"}:
            raise ValidationError("report path must end in .html or .htm")
        target.write_bytes(payload)
        return target


# --------------------------------------------------------------------------- #
# CSV ingest                                                                    #
# --------------------------------------------------------------------------- #


def _read_csv(
    source: str | PathLike[str],
    *,
    time_column: str,
    state_columns: Sequence[str],
) -> tuple[list[float], dict[str, list[float]]]:
    """Read a CSV path (or literal CSV text) into ``(time, {state: series})``."""
    text = _read_csv_text(source)
    reader = _csv.DictReader(text.splitlines())
    if reader.fieldnames is None:
        raise ValidationError("CSV is empty or has no header row")
    required = [time_column, *state_columns]
    missing = [column for column in required if column not in reader.fieldnames]
    if missing:
        raise ValidationError(
            f"CSV is missing required columns {missing}; found {reader.fieldnames}"
        )
    time: list[float] = []
    columns: dict[str, list[float]] = {column: [] for column in state_columns}
    for line_number, row in enumerate(reader, start=2):
        try:
            time.append(float(row[time_column]))
            for column in state_columns:
                columns[column].append(float(row[column]))
        except (TypeError, ValueError) as error:
            raise ValidationError(
                f"non-numeric value on CSV line {line_number}: {error}"
            ) from error
    if not time:
        raise ValidationError("CSV contains a header but no data rows")
    return time, columns


def _read_csv_text(source: str | PathLike[str]) -> str:
    """Return CSV text: read a file when ``source`` points at one, else use it verbatim."""
    if isinstance(source, PathLike):
        return Path(source).read_text(encoding="utf-8")
    if "\n" not in source and "\r" not in source:
        candidate = Path(source)
        try:
            if candidate.is_file():
                return candidate.read_text(encoding="utf-8")
        except OSError:
            pass
    return source
