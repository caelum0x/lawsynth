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


def request(app, method: str, path: str, *, body: object | None = None, headers: dict[str, str] | None = None) -> tuple[int, dict[str, str], dict[str, Any] | None]:
    raw = b"" if body is None else json.dumps(body).encode("utf-8")
    environ: dict[str, object] = {
        "REQUEST_METHOD": method,
        "PATH_INFO": path,
        "QUERY_STRING": "",
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
