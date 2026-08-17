"""Tests for the WSGI composition root: telemetry, readiness, decoration."""

from __future__ import annotations

from _harness import auth, make_app, request


def test_health_publishes_protocol_version_header(tmp_path):
    app = make_app(tmp_path)
    try:
        status, headers, health = request(app, "GET", "/v1/health")
        assert status == 200 and health["status"] == "ok"
        assert headers["X-Api-Version"] == "1"
        assert headers["Cache-Control"] == "no-store"
    finally:
        app.close()


def test_unsupported_api_version_is_rejected(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, body = request(app, "GET", "/v1/version", headers={"X-Api-Version": "2"})
        assert status == 406 and body["error"]["code"] == "unsupported_api_version"
    finally:
        app.close()


def test_worker_transport_is_an_explicit_boundary(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, body = request(app, "POST", "/v1/worker/jobs", body={}, headers=auth(key="job-1"))
        assert status == 501 and body["error"]["code"] == "worker_transport_unavailable"
    finally:
        app.close()


def test_oversized_body_is_rejected_before_dispatch(tmp_path):
    app = make_app(tmp_path)  # max_request_bytes == 1024
    try:
        status, _, body = request(app, "POST", "/v1/projects", body={"name": "x" * 2048}, headers=auth(key="big"))
        assert status == 413 and body["error"]["code"] == "payload_too_large"
    finally:
        app.close()


def test_telemetry_counts_completed_requests_by_route(tmp_path):
    app = make_app(tmp_path)
    try:
        request(app, "GET", "/v1/health")
        request(app, "POST", "/v1/projects", body={"name": "p"}, headers=auth(key="p-1"))
        request(app, "GET", "/v1/projects", headers=auth())
        snapshot = app.telemetry.snapshot()
        assert snapshot["total"] == 3
        assert snapshot["by_route"]["health:200"] == 1
        assert snapshot["by_route"]["projects.create:201"] == 1
        assert snapshot["by_route"]["projects.list:200"] == 1
    finally:
        app.close()


def test_telemetry_labels_unknown_routes(tmp_path):
    app = make_app(tmp_path)
    try:
        request(app, "GET", "/v1/nonexistent", headers=auth())
        snapshot = app.telemetry.snapshot()
        assert any(key.startswith("unknown:") for key in snapshot["by_route"])
    finally:
        app.close()


def test_readiness_probes_domain_accessors(tmp_path):
    app = make_app(tmp_path)
    try:
        request(app, "GET", "/v1/health")
        readiness = app.readiness()
        assert readiness["database"] is True
        assert readiness["storage"] is True
        assert set(readiness["resources"]) == {"projects", "datasets", "worlds", "runs"}
        assert readiness["telemetry"]["total"] == 1
    finally:
        app.close()


def test_malformed_json_is_a_transport_error(tmp_path):
    import io

    app = make_app(tmp_path)
    try:
        environ = {
            "REQUEST_METHOD": "POST",
            "PATH_INFO": "/v1/projects",
            "QUERY_STRING": "",
            "CONTENT_LENGTH": "1",
            "CONTENT_TYPE": "application/json",
            "HTTP_AUTHORIZATION": f"Bearer {auth()['Authorization'].split()[1]}",
            "wsgi.input": io.BytesIO(b"["),
        }
        captured = {}
        body = b"".join(app(environ, lambda status, hs: captured.update(status=status, headers=dict(hs))))
        assert str(captured["status"]).startswith("400")
        import json as _json

        assert _json.loads(body)["error"]["code"] == "invalid_json"
    finally:
        app.close()
