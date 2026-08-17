"""Offline, deterministic tests for :class:`lawsynth.Client`.

The transport/logic tests drive the client against a tiny in-process fake WSGI
service, so they need neither a socket nor the native engine. A final,
importorskip-guarded test exercises the real ``lawsynth_api`` app end to end.
"""

from __future__ import annotations

import io
import json

import pytest

import lawsynth
from lawsynth import ApiError, Client, Run, RunTimeout

# Pre-existing SDK collision: the package exposes a public ``lawsynth.discover``
# callable (which the API server invokes), but the SDK also ships a
# ``lawsynth/discover.py`` submodule. Importing that submodule (as
# ``test_discover.py`` does) rebinds ``lawsynth.discover`` to the module and
# shadows the callable. This module is imported before ``test_discover`` (pytest
# collects alphabetically), so capture the genuine callable now and restore it in
# the end-to-end test so it is order-independent. See the task report.
_PUBLIC_DISCOVER = lawsynth.discover


# --------------------------------------------------------------------------- #
# A minimal fake WSGI service: records requests, returns scripted responses.   #
# --------------------------------------------------------------------------- #


class FakeService:
    """Route (method, path) to a scripted ``(status, headers, body)`` response."""

    def __init__(self):
        self.requests: list[dict[str, object]] = []
        self.routes: dict[tuple[str, str], object] = {}

    def route(self, method: str, path: str, status: int, body, *, content_type="application/json"):
        self.routes[(method, path)] = (status, content_type, body)
        return self

    def __call__(self, environ, start_response):
        body_bytes = environ["wsgi.input"].read()
        headers = {
            key[5:].replace("_", "-").title(): value
            for key, value in environ.items()
            if isinstance(key, str) and key.startswith("HTTP_")
        }
        record = {
            "method": environ["REQUEST_METHOD"],
            "path": environ["PATH_INFO"],
            "query": environ.get("QUERY_STRING", ""),
            "headers": headers,
            "body": json.loads(body_bytes) if body_bytes else None,
        }
        self.requests.append(record)
        key = (record["method"], record["path"])
        status, content_type, body = self.routes.get(
            key, (404, "application/json", {"error": {"code": "not_found", "message": "no route", "request_id": "rid-404"}})
        )
        payload = body if isinstance(body, (bytes, bytearray)) else json.dumps(body).encode("utf-8")
        start_response(f"{status} STATUS", [("Content-Type", content_type)])
        return [payload]


def make_client(service: FakeService, **kwargs) -> Client:
    return Client(wsgi_app=service, token="secret-token", **kwargs)


# --------------------------------------------------------------------------- #
# Construction                                                                  #
# --------------------------------------------------------------------------- #


def test_requires_exactly_one_transport():
    with pytest.raises(lawsynth.ValidationError):
        Client()
    with pytest.raises(lawsynth.ValidationError):
        Client("http://host", wsgi_app=FakeService())


def test_rejects_nonpositive_poll_bound():
    with pytest.raises(lawsynth.ValidationError):
        Client(wsgi_app=FakeService(), max_poll_attempts=0)


# --------------------------------------------------------------------------- #
# Auth / version / idempotency headers                                          #
# --------------------------------------------------------------------------- #


def test_sends_auth_version_and_content_type_headers():
    service = FakeService().route("GET", "/v1/version", 200, {"version": "9.9", "protocol": "1"})
    banner = make_client(service).version()
    assert banner == {"version": "9.9", "protocol": "1"}
    sent = service.requests[-1]["headers"]
    assert sent["Authorization"] == "Bearer secret-token"
    assert sent["X-Api-Version"] == "1"


def test_write_requests_carry_an_idempotency_key():
    service = FakeService().route("POST", "/v1/datasets", 201, {"id": "ds-1"})
    make_client(service).upload_dataset(time=[0.0, 1.0], columns={"x": [1.0, 2.0]}, name="d")
    sent = service.requests[-1]
    assert "Idempotency-Key" in sent["headers"]
    assert sent["body"]["schema"] == ["x"]


# --------------------------------------------------------------------------- #
# Error envelope -> ApiError                                                    #
# --------------------------------------------------------------------------- #


def test_error_envelope_becomes_typed_apierror():
    service = FakeService().route(
        "GET", "/v1/worlds/nope",
        404, {"error": {"code": "not_found", "message": "unknown world", "request_id": "rid-7"}},
    )
    with pytest.raises(ApiError) as excinfo:
        make_client(service).get_world("nope")
    error = excinfo.value
    assert error.status == 404 and error.code == "not_found"
    assert error.request_id == "rid-7"
    assert "unknown world" in str(error)


# --------------------------------------------------------------------------- #
# Run submission, parsing, and bounded waiting                                  #
# --------------------------------------------------------------------------- #


def test_submit_discovery_uploads_then_references_dataset():
    service = (
        FakeService()
        .route("POST", "/v1/datasets", 201, {"id": "ds-9"})
        .route("POST", "/v1/runs", 201, {"id": "run-9", "status": "succeeded", "world_id": "w-9"})
    )
    run = make_client(service).submit_discovery(
        columns={"x": [1.0, 2.0, 3.0], "y": [4.0, 5.0, 6.0]},
        time=[0.0, 1.0, 2.0],
        state=["x", "y"],
        preset="ecology",
    )
    assert isinstance(run, Run) and run.succeeded and run.world_id == "w-9"
    run_body = service.requests[-1]["body"]
    assert run_body["dataset_id"] == "ds-9"
    assert run_body["states"] == ["x", "y"]
    # 'ecology' preset resolved client-side into concrete discovery knobs.
    assert run_body["discovery"]["polynomial_degree"] == 2
    assert run_body["discovery"]["solver"] == "stlsq"


def test_explicit_knobs_override_preset():
    service = FakeService().route("POST", "/v1/runs", 201, {"id": "r", "status": "queued"})
    make_client(service).submit_discovery(
        dataset_id="ds", state=["x"], preset="ecology", degree=4, threshold=0.5, solver="sr3",
    )
    discovery = service.requests[-1]["body"]["discovery"]
    assert discovery["polynomial_degree"] == 4
    assert discovery["threshold"] == 0.5
    assert discovery["solver"] == "sr3"


def test_submit_requires_exactly_one_dataset_source():
    client = make_client(FakeService())
    with pytest.raises(lawsynth.ValidationError):
        client.submit_discovery(state=["x"])  # no source
    with pytest.raises(lawsynth.ValidationError):
        client.submit_discovery(state=["x"], dataset_id="ds", columns={"x": [1.0]})


def test_wait_polls_until_terminal_without_sleeping():
    class Polling(FakeService):
        def __init__(self):
            super().__init__()
            self._statuses = iter(["running", "running", "succeeded"])

        def __call__(self, environ, start_response):
            if environ["PATH_INFO"] == "/v1/runs/run-1":
                status = next(self._statuses)
                start_response("200 OK", [("Content-Type", "application/json")])
                return [json.dumps({"id": "run-1", "status": status, "world_id": "w-1"}).encode()]
            return super().__call__(environ, start_response)

    service = Polling()
    run = make_client(service).wait(Run.from_payload({"id": "run-1", "status": "queued"}))
    assert run.succeeded and run.world_id == "w-1"


def test_wait_raises_run_timeout_when_bound_exhausted():
    service = FakeService().route("GET", "/v1/runs/stuck", 200, {"id": "stuck", "status": "running"})
    with pytest.raises(RunTimeout) as excinfo:
        make_client(service, max_poll_attempts=3).wait("stuck")
    assert excinfo.value.attempts == 3 and excinfo.value.status == "running"


# --------------------------------------------------------------------------- #
# World access: run-scoped endpoint with graceful 404 fallback                  #
# --------------------------------------------------------------------------- #


def test_world_prefers_run_scoped_endpoint():
    service = FakeService().route(
        "GET", "/v1/runs/run-1/world", 200, {"id": "w-1", "name": "W", "equations": {"x": "x"}}
    )
    world = make_client(service).world(Run.from_payload({"id": "run-1", "status": "succeeded", "world_id": "w-1"}))
    assert world["id"] == "w-1"


def test_world_unwraps_run_scoped_envelope():
    # The run-scoped endpoint wraps the record: {run_id, world_id, world, links}.
    service = FakeService().route(
        "GET", "/v1/runs/run-1/world", 200,
        {"run_id": "run-1", "world_id": "w-1", "links": {}, "world": {"id": "w-1", "name": "W", "equations": {"x": "x"}}},
    )
    world = make_client(service).world(Run.from_payload({"id": "run-1", "status": "succeeded", "world_id": "w-1"}))
    assert world["name"] == "W" and world["equations"] == {"x": "x"}


def test_world_falls_back_to_worlds_endpoint_on_404():
    service = (
        FakeService()
        .route("GET", "/v1/runs/run-1/world", 404, {"error": {"code": "not_found", "message": "x", "request_id": "r"}})
        .route("GET", "/v1/worlds/w-1", 200, {"id": "w-1", "name": "fallback"})
    )
    world = make_client(service).world(Run.from_payload({"id": "run-1", "status": "succeeded", "world_id": "w-1"}))
    assert world["name"] == "fallback"
    assert [r["path"] for r in service.requests] == ["/v1/runs/run-1/world", "/v1/worlds/w-1"]


# --------------------------------------------------------------------------- #
# Report: HTML bytes on success, envelope on failure                            #
# --------------------------------------------------------------------------- #


def test_report_writes_html_bytes(tmp_path):
    html = b"<!doctype html><title>report</title>"
    service = FakeService().route("GET", "/v1/worlds/w-1/report", 200, html, content_type="text/html")
    path = make_client(service).report("w-1", tmp_path / "r.html")
    assert path.read_bytes() == html


def test_report_rejects_non_html_extension(tmp_path):
    service = FakeService().route("GET", "/v1/worlds/w-1/report", 200, b"<html>", content_type="text/html")
    with pytest.raises(lawsynth.ValidationError):
        make_client(service).report("w-1", tmp_path / "r.txt")


def test_report_raises_on_error_envelope(tmp_path):
    service = FakeService().route(
        "GET", "/v1/worlds/w-1/report", 403,
        {"error": {"code": "forbidden", "message": "nope", "request_id": "r"}},
    )
    with pytest.raises(ApiError) as excinfo:
        make_client(service).report("w-1", tmp_path / "r.html")
    assert excinfo.value.status == 403


# --------------------------------------------------------------------------- #
# CSV ingest                                                                    #
# --------------------------------------------------------------------------- #


def test_submit_from_csv_text_parses_columns():
    service = (
        FakeService()
        .route("POST", "/v1/datasets", 201, {"id": "ds-csv"})
        .route("POST", "/v1/runs", 201, {"id": "run-csv", "status": "succeeded", "world_id": "w"})
    )
    csv_text = "t,x,y\n0,1,4\n1,2,5\n2,3,6\n"
    make_client(service).submit_discovery(csv=csv_text, time="t", state=["x", "y"])
    dataset_body = service.requests[0]["body"]
    assert dataset_body["time"] == [0.0, 1.0, 2.0]
    assert dataset_body["columns"] == {"x": [1.0, 2.0, 3.0], "y": [4.0, 5.0, 6.0]}


def test_csv_missing_column_is_rejected():
    with pytest.raises(lawsynth.ValidationError):
        make_client(FakeService()).submit_discovery(csv="t,x\n0,1\n", time="t", state=["x", "y"])


# --------------------------------------------------------------------------- #
# End-to-end against the real API app (skipped if server/native unavailable).   #
# --------------------------------------------------------------------------- #


def test_end_to_end_against_real_wsgi_app(tmp_path):
    lawsynth_api = pytest.importorskip("lawsynth_api")
    pytest.importorskip("lawsynth_server")
    try:
        _ = lawsynth.World  # native engine required for discovery
    except lawsynth.NativeError:
        pytest.skip("native engine unavailable")
    # Restore the public discovery callable in case a prior test imported the
    # colliding ``lawsynth.discover`` submodule and shadowed it (see module note).
    lawsynth.discover = _PUBLIC_DISCOVER

    from lawsynth_server.settings import Settings as ServerSettings

    token = "0123456789abcdef0123456789abcdef"
    server = ServerSettings(
        database_url=f"sqlite:///{tmp_path / 'm.sqlite3'}",
        object_root=tmp_path / "obj",
        tokens={token: ("acme", frozenset({"read", "write"}))},
        max_upload_bytes=8 * 1024 * 1024,
    )
    app = lawsynth_api.create_wsgi_app(
        lawsynth_api.ApiSettings(server=server, environment="test", max_request_bytes=8 * 1024 * 1024)
    )
    try:
        client = Client(wsgi_app=app, token=token)
        time = [round(i * 0.1, 6) for i in range(60)]
        x, y = [], []
        xv, yv = 10.0, 5.0
        for _ in time:
            x.append(xv)
            y.append(yv)
            xv, yv = xv + (1.1 * xv - 0.4 * xv * yv) * 0.1, yv + (0.1 * xv * yv - 0.4 * yv) * 0.1
        run = client.submit_discovery(
            columns={"x": x, "y": y}, time=time, state=["x", "y"], preset="ecology"
        )
        run = client.wait(run)
        assert run.succeeded and run.world_id
        world = client.world(run)
        assert set(world["equations"]) == {"x", "y"}
        explanation = client.explain(run.world_id)
        assert explanation["complexity"]["laws"] == 2
        report = client.report(run.world_id, tmp_path / "report.html")
        assert report.read_text().startswith("<!")
    finally:
        app.close()
