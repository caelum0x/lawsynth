from __future__ import annotations

import io
import json
from pathlib import Path
from typing import Any

from lawsynth_api import ApiSettings, create_wsgi_app
from lawsynth_server.settings import Settings as ServerSettings


TOKEN = "0123456789abcdef0123456789abcdef"


def make_app(tmp_path):
    server = ServerSettings(
        database_url=f"sqlite:///{tmp_path / 'metadata.sqlite3'}",
        object_root=tmp_path / "objects",
        tokens={TOKEN: ("acme", frozenset({"read", "write"}))},
        max_upload_bytes=1024,
    )
    return create_wsgi_app(ApiSettings(server=server, environment="test", max_request_bytes=1024))


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


def headers(*, key: str | None = None) -> dict[str, str]:
    result = {"Authorization": f"Bearer {TOKEN}"}
    if key:
        result["Idempotency-Key"] = key
    return result


def test_wsgi_health_and_real_project_lifecycle(tmp_path):
    app = make_app(tmp_path)
    fixture = json.loads((Path(__file__).parents[1] / "fixtures" / "project-create.json").read_text())
    try:
        status, response_headers, health = request(app, "GET", "/v1/health")
        assert status == 200
        assert health["status"] == "ok"
        assert response_headers["Cache-Control"] == "no-store"
        assert response_headers["X-Request-ID"]

        status, _, created = request(app, "POST", "/v1/projects", body=fixture, headers=headers(key="create-coastal"))
        assert status == 201
        status, _, listed = request(app, "GET", "/v1/projects", headers=headers())
        assert status == 200
        assert listed["items"] == [created]
    finally:
        app.close()


def test_wsgi_preserves_idempotent_artifact_write(tmp_path):
    app = make_app(tmp_path)
    artifact = {"data_base64": "dmVyaWZpZWQ=", "media_type": "text/plain"}
    try:
        first = request(app, "POST", "/v1/artifacts", body=artifact, headers=headers(key="artifact-1"))
        repeated = request(app, "POST", "/v1/artifacts", body=artifact, headers=headers(key="artifact-1"))
        assert first[0] == repeated[0] == 201
        assert first[2] == repeated[2]
        assert first[1]["Idempotency-Replayed"] == "false"
        assert repeated[1]["Idempotency-Replayed"] == "true"
    finally:
        app.close()


def test_wsgi_rejects_malformed_transport_before_domain_dispatch(tmp_path):
    app = make_app(tmp_path)
    try:
        environ = {
            "REQUEST_METHOD": "POST",
            "PATH_INFO": "/v1/projects",
            "QUERY_STRING": "",
            "CONTENT_LENGTH": "1",
            "CONTENT_TYPE": "application/json",
            "HTTP_AUTHORIZATION": f"Bearer {TOKEN}",
            "wsgi.input": io.BytesIO(b"["),
        }
        captured: dict[str, object] = {}
        body = b"".join(app(environ, lambda status, response_headers: captured.update(status=status, headers=dict(response_headers))))
        assert str(captured["status"]).startswith("400")
        assert json.loads(body)["error"]["code"] == "invalid_json"
    finally:
        app.close()


def test_worker_transport_is_an_explicit_boundary(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, body = request(app, "POST", "/v1/worker/jobs", body={}, headers=headers(key="job-1"))
        assert status == 501
        assert body["error"]["code"] == "worker_transport_unavailable"
    finally:
        app.close()


def test_wsgi_reports_missing_native_runtime_without_faking_simulation(tmp_path):
    app = make_app(tmp_path)
    world = {
        "name": "decay",
        "states": ["x"],
        "controls": [],
        "parameters": {"rate": 0.2},
        "equations": {"x": "-rate * x"},
    }
    try:
        status, _, created = request(app, "POST", "/v1/worlds", body=world, headers=headers(key="world-1"))
        assert status == 201
        status, _, body = request(
            app,
            "POST",
            f"/v1/worlds/{created['id']}/simulate",
            body={"initial": {"x": 1.0}, "horizon": 1.0, "step": 0.1},
            headers=headers(key="simulate-1"),
        )
        assert status == 503
        assert body["error"]["code"] == "native_unavailable"
    finally:
        app.close()


def test_production_settings_reject_volatile_storage():
    try:
        ApiSettings.from_environment({"LAWSYNTH_API_ENV": "production"})
    except Exception as error:
        assert getattr(error, "code", None) == "validation_error"
    else:
        raise AssertionError("production must not accept in-memory metadata")


def make_readonly_app(tmp_path):
    server = ServerSettings(
        database_url=f"sqlite:///{tmp_path / 'metadata.sqlite3'}",
        object_root=tmp_path / "objects",
        tokens={TOKEN: ("acme", frozenset({"read"}))},
        max_upload_bytes=1024,
    )
    return create_wsgi_app(ApiSettings(server=server, environment="test", max_request_bytes=1024))


def test_version_endpoint_publishes_explicit_protocol(tmp_path):
    app = make_app(tmp_path)
    try:
        # No authentication required: version is public capability discovery.
        status, response_headers, body = request(app, "GET", "/v1/version")
        assert status == 200
        assert body["protocol"] == "1"
        assert isinstance(body["version"], str) and body["version"]
        # The protocol version is published on every response, including health.
        assert response_headers["X-Api-Version"] == "1"
        _, health_headers, _ = request(app, "GET", "/v1/health")
        assert health_headers["X-Api-Version"] == "1"
    finally:
        app.close()


def test_version_negotiation_rejects_unsupported_client(tmp_path):
    app = make_app(tmp_path)
    try:
        for accepted in ("1", "v1"):
            status, _, _ = request(app, "GET", "/v1/version", headers={"X-Api-Version": accepted})
            assert status == 200
        status, _, body = request(app, "GET", "/v1/version", headers={"X-Api-Version": "2"})
        assert status == 406
        assert body["error"]["code"] == "unsupported_api_version"
    finally:
        app.close()


def test_artifact_get_roundtrip_and_failure_modes(tmp_path):
    app = make_app(tmp_path)
    artifact = {"data_base64": "dmVyaWZpZWQ=", "media_type": "text/plain"}
    try:
        status, _, created = request(app, "POST", "/v1/artifacts", body=artifact, headers=headers(key="artifact-get"))
        assert status == 201
        sha = created["sha256"]

        # Happy path: content-addressed download returns the stored bytes.
        status, _, fetched = request(app, "GET", f"/v1/artifacts/{sha}", headers=headers())
        assert status == 200
        assert fetched["data_base64"] == artifact["data_base64"]
        assert fetched["size"] == 8 and fetched["sha256"] == sha

        # Auth failure.
        status, _, body = request(app, "GET", f"/v1/artifacts/{sha}")
        assert status == 401

        # Validation error: malformed identifier.
        status, _, body = request(app, "GET", "/v1/artifacts/not-a-sha", headers=headers())
        assert status == 422
        assert body["error"]["code"] == "validation_error"

        # Not found: well-formed but absent object.
        status, _, body = request(app, "GET", f"/v1/artifacts/{'0' * 64}", headers=headers())
        assert status == 404
        assert body["error"]["code"] == "not_found"
    finally:
        app.close()


def test_run_cancel_lifecycle_and_authorization(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, run = request(app, "POST", "/v1/runs", body={"name": "cancel-me"}, headers=headers(key="run-1"))
        assert status == 201 and run["status"] == "queued"
        run_id = run["id"]

        # Auth failure.
        status, _, _ = request(app, "POST", f"/v1/runs/{run_id}/cancel", headers={"Idempotency-Key": "c0"})
        assert status == 401

        # Happy path.
        status, _, cancelled = request(app, "POST", f"/v1/runs/{run_id}/cancel", headers=headers(key="cancel-1"))
        assert status == 200 and cancelled["status"] == "cancelled"

        # Cancelling a terminal run conflicts (fresh key so it is not a replay).
        status, _, body = request(app, "POST", f"/v1/runs/{run_id}/cancel", headers=headers(key="cancel-2"))
        assert status == 409 and body["error"]["code"] == "conflict"

        # Not found.
        status, _, body = request(app, "POST", "/v1/runs/does-not-exist/cancel", headers=headers(key="cancel-3"))
        assert status == 404 and body["error"]["code"] == "not_found"
    finally:
        app.close()


def test_run_cancel_requires_write_scope(tmp_path):
    app = make_readonly_app(tmp_path)
    try:
        status, _, body = request(app, "POST", "/v1/runs/anything/cancel", headers=headers(key="ro-cancel"))
        assert status == 403
        assert body["error"]["code"] == "forbidden"
    finally:
        app.close()


def test_run_events_are_scoped_and_authorized(tmp_path):
    app = make_app(tmp_path)
    try:
        _, _, run = request(app, "POST", "/v1/runs", body={"name": "trace"}, headers=headers(key="run-e"))
        run_id = run["id"]
        request(app, "POST", f"/v1/runs/{run_id}/cancel", headers=headers(key="cancel-e"))

        status, _, listed = request(app, "GET", f"/v1/runs/{run_id}/events", headers=headers())
        assert status == 200
        topics = {event["topic"] for event in listed["items"]}
        assert {"runs.created", "runs.cancelled"} <= topics
        assert all(event["payload"]["id"] == run_id for event in listed["items"])

        # Unknown run is a 404, not an empty list, so ownership is verified.
        status, _, body = request(app, "GET", "/v1/runs/missing/events", headers=headers())
        assert status == 404

        # Candidates remain intentionally unexposed (not backed by the server).
        status, _, body = request(app, "GET", f"/v1/runs/{run_id}/candidates", headers=headers())
        assert status == 404
    finally:
        app.close()


def test_list_pagination_envelope_and_auth(tmp_path):
    app = make_app(tmp_path)
    try:
        for index in range(3):
            status, _, _ = request(app, "POST", "/v1/projects", body={"name": f"p{index}"}, headers=headers(key=f"p{index}"))
            assert status == 201

        status, _, first = request(app, "GET", "/v1/projects", headers=headers(), query="limit=2")
        assert status == 200
        assert len(first["items"]) == 2
        assert first["total"] == 3 and first["limit"] == 2
        assert first["next_cursor"] is not None

        status, _, second = request(app, "GET", "/v1/projects", headers=headers(), query=f"cursor={first['next_cursor']}&limit=2")
        assert status == 200
        assert len(second["items"]) == 1
        assert second["total"] == 3 and second["next_cursor"] is None

        # Auth failure on a list read.
        status, _, _ = request(app, "GET", "/v1/projects")
        assert status == 401
    finally:
        app.close()
