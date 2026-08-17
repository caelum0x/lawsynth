from __future__ import annotations

import io
import json

import pytest

from lawsynth_api import (
    ApiEvent,
    ApiSettings,
    EventBus,
    EventKind,
    create_wsgi_app,
    render_frame,
    validate_event_stream,
)
from lawsynth_api.events import PAYLOAD_LIMIT_BYTES
from lawsynth_server.errors import ValidationError
from lawsynth_server.settings import Settings as ServerSettings


TOKEN_ACME = "0123456789abcdef0123456789abcdef"
TOKEN_GLOBEX = "fedcba9876543210fedcba9876543210"


# --------------------------------------------------------------------------- #
# ApiEvent value contract                                                      #
# --------------------------------------------------------------------------- #


def test_api_event_accepts_valid_run_event():
    event = ApiEvent(
        sequence=1,
        occurred_at_ms=1_700_000_000_000,
        project_id="acme",
        run_id="run-1",
        kind=EventKind.RUN_QUEUED,
        payload=json.dumps({"id": "run-1"}),
    )
    assert event.kind is EventKind.RUN_QUEUED
    assert event.to_wire()["kind"] == "run_queued"


def test_api_event_requires_run_id_for_run_kinds():
    with pytest.raises(ValidationError):
        ApiEvent(1, 1, "acme", None, EventKind.RUN_STARTED, "{}")


def test_api_event_allows_missing_run_id_for_artifact_kind():
    event = ApiEvent(1, 1, "acme", None, EventKind.ARTIFACT_CREATED, "{}")
    assert event.run_id is None


def test_api_event_rejects_nul_payload():
    with pytest.raises(ValidationError):
        ApiEvent(1, 1, "acme", None, EventKind.ARTIFACT_CREATED, "bad\x00payload")


def test_api_event_rejects_oversized_payload():
    with pytest.raises(ValidationError):
        ApiEvent(1, 1, "acme", None, EventKind.ARTIFACT_CREATED, "x" * (PAYLOAD_LIMIT_BYTES + 1))


def test_api_event_payload_bound_is_measured_in_utf8_bytes():
    # A 2-byte character right at the boundary must be rejected.
    multibyte = "é" * ((PAYLOAD_LIMIT_BYTES // 2) + 1)
    with pytest.raises(ValidationError):
        ApiEvent(1, 1, "acme", None, EventKind.ARTIFACT_CREATED, multibyte)


def test_api_event_rejects_negative_sequence():
    with pytest.raises(ValidationError):
        ApiEvent(-1, 1, "acme", None, EventKind.ARTIFACT_CREATED, "{}")


def test_validate_event_stream_requires_strictly_increasing_sequence():
    good = [
        ApiEvent(1, 10, "acme", None, EventKind.ARTIFACT_CREATED, "{}"),
        ApiEvent(2, 10, "acme", None, EventKind.ARTIFACT_CREATED, "{}"),
    ]
    validate_event_stream(good)  # does not raise

    with pytest.raises(ValidationError):
        validate_event_stream(
            [
                ApiEvent(2, 10, "acme", None, EventKind.ARTIFACT_CREATED, "{}"),
                ApiEvent(2, 11, "acme", None, EventKind.ARTIFACT_CREATED, "{}"),
            ]
        )


def test_validate_event_stream_rejects_backwards_time():
    with pytest.raises(ValidationError):
        validate_event_stream(
            [
                ApiEvent(1, 20, "acme", None, EventKind.ARTIFACT_CREATED, "{}"),
                ApiEvent(2, 19, "acme", None, EventKind.ARTIFACT_CREATED, "{}"),
            ]
        )


# --------------------------------------------------------------------------- #
# EventBus                                                                     #
# --------------------------------------------------------------------------- #


def test_bus_assigns_monotonic_per_scope_sequence():
    bus = EventBus()
    a = bus.append("acme", 1, EventKind.ARTIFACT_CREATED, "{}")
    b = bus.append("acme", 2, EventKind.ARTIFACT_CREATED, "{}")
    assert (a.sequence, b.sequence) == (1, 2)
    # Sequence is per scope, so a fresh scope also starts at 1.
    c = bus.append("globex", 5, EventKind.ARTIFACT_CREATED, "{}")
    assert c.sequence == 1
    assert [e.sequence for e in bus.events_after("acme", 0)] == [1, 2]


def test_bus_query_after_sequence_returns_only_newer_events():
    bus = EventBus()
    for i in range(1, 4):
        bus.append("acme", i, EventKind.ARTIFACT_CREATED, "{}")
    assert [e.sequence for e in bus.events_after("acme", 1)] == [2, 3]
    assert bus.events_after("acme", 3) == []


def test_bus_enforces_project_isolation():
    bus = EventBus()
    bus.append("acme", 1, EventKind.ARTIFACT_CREATED, "{}")
    bus.append("globex", 1, EventKind.ARTIFACT_CREATED, "{}")
    acme = bus.events_after("acme", 0)
    assert len(acme) == 1 and acme[0].project_id == "acme"
    # An unknown scope never leaks another scope's events.
    assert bus.events_after("unknown", 0) == []


def test_bus_ring_buffer_drops_oldest_past_retention():
    bus = EventBus(retention=3)
    for i in range(1, 6):  # append sequences 1..5 into a size-3 ring
        bus.append("acme", i, EventKind.ARTIFACT_CREATED, "{}")
    retained = bus.events_after("acme", 0)
    # Only the newest three survive; the two oldest were dropped.
    assert [e.sequence for e in retained] == [3, 4, 5]
    # A resume cursor older than the retained window skips the dropped events;
    # the client can detect the gap because the first id (3) > cursor + 1.
    resumed = bus.events_after("acme", 1)
    assert [e.sequence for e in resumed] == [3, 4, 5]


def test_bus_rejects_invalid_retention():
    with pytest.raises(ValidationError):
        EventBus(retention=0)


def test_render_frame_is_well_formed_sse():
    event = ApiEvent(7, 123, "acme", "run-1", EventKind.RUN_STARTED, json.dumps({"id": "run-1"}))
    frame = render_frame(event).decode("utf-8")
    assert frame.startswith("id: 7\n")
    assert "event: run_started\n" in frame
    assert frame.endswith("\n\n")
    data_line = next(line for line in frame.splitlines() if line.startswith("data: "))
    parsed = json.loads(data_line[len("data: ") :])
    assert parsed["sequence"] == 7 and parsed["run_id"] == "run-1"


# --------------------------------------------------------------------------- #
# WSGI SSE endpoint                                                            #
# --------------------------------------------------------------------------- #


def make_app(tmp_path):
    server = ServerSettings(
        database_url=f"sqlite:///{tmp_path / 'metadata.sqlite3'}",
        object_root=tmp_path / "objects",
        tokens={
            TOKEN_ACME: ("acme", frozenset({"read", "write"})),
            TOKEN_GLOBEX: ("globex", frozenset({"read", "write"})),
        },
        max_upload_bytes=1024,
    )
    return create_wsgi_app(ApiSettings(server=server, environment="test", max_request_bytes=1024))


def _json_request(app, method, path, *, body=None, headers=None):
    raw = b"" if body is None else json.dumps(body).encode("utf-8")
    environ = {
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
    captured = {}

    def start_response(status, response_headers):
        captured["status"], captured["headers"] = status, dict(response_headers)

    payload = b"".join(app(environ, start_response))
    decoded = None if not payload else json.loads(payload)
    return int(str(captured["status"]).split(" ", 1)[0]), captured["headers"], decoded


def _sse_request(app, *, token=None, last_event_id=None):
    environ = {
        "REQUEST_METHOD": "GET",
        "PATH_INFO": "/v1/events",
        "QUERY_STRING": "",
        "CONTENT_LENGTH": "0",
        "wsgi.input": io.BytesIO(b""),
        "HTTP_ACCEPT": "text/event-stream",
    }
    if token is not None:
        environ["HTTP_AUTHORIZATION"] = f"Bearer {token}"
    if last_event_id is not None:
        environ["HTTP_LAST_EVENT_ID"] = str(last_event_id)
    captured = {}

    def start_response(status, response_headers):
        captured["status"], captured["headers"] = status, dict(response_headers)

    payload = b"".join(app(environ, start_response))
    return int(str(captured["status"]).split(" ", 1)[0]), captured["headers"], payload.decode("utf-8")


def _parse_sse(text):
    frames = []
    for block in text.split("\n\n"):
        block = block.strip()
        if not block or block.startswith(":"):
            continue
        frame = {}
        for line in block.splitlines():
            field, _, value = line.partition(": ")
            frame[field] = value
        if "data" in frame:
            frame["data"] = json.loads(frame["data"])
        frames.append(frame)
    return frames


def _auth(token=TOKEN_ACME, key=None):
    result = {"Authorization": f"Bearer {token}"}
    if key:
        result["Idempotency-Key"] = key
    return result


def test_sse_requires_authentication(tmp_path):
    app = make_app(tmp_path)
    try:
        status, resp_headers, text = _sse_request(app, token=None)
        assert status == 401
        # Unauthenticated requests get a JSON error envelope, not a stream.
        assert "application/json" in resp_headers["Content-Type"]
        assert json.loads(text)["error"]["code"] == "authentication_required"
    finally:
        app.close()


def test_sse_returns_event_stream_with_framed_run_events(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, created = _json_request(
            app, "POST", "/v1/runs", body={"name": "coastal-run"}, headers=_auth(key="run-1")
        )
        assert status == 201 and created["status"] == "queued"

        status, resp_headers, text = _sse_request(app, token=TOKEN_ACME)
        assert status == 200
        assert resp_headers["Content-Type"] == "text/event-stream; charset=utf-8"
        assert resp_headers["Cache-Control"] == "no-cache"
        assert resp_headers["Connection"] == "keep-alive"
        assert resp_headers["X-Accel-Buffering"] == "no"

        frames = _parse_sse(text)
        assert len(frames) == 1
        assert frames[0]["event"] == "run_queued"
        assert frames[0]["id"] == "1"
        assert frames[0]["data"]["run_id"] == created["id"]
        assert frames[0]["data"]["project_id"] == "acme"
    finally:
        app.close()


def test_sse_streams_patch_and_artifact_transitions(tmp_path):
    app = make_app(tmp_path)
    try:
        _, _, created = _json_request(
            app, "POST", "/v1/runs", body={"name": "run-lifecycle"}, headers=_auth(key="run-1")
        )
        _json_request(
            app,
            "PATCH",
            f"/v1/runs/{created['id']}",
            body={"status": "running"},
            headers=_auth(key="run-patch-1"),
        )
        _json_request(
            app,
            "POST",
            "/v1/artifacts",
            body={"data_base64": "dmVyaWZpZWQ=", "media_type": "text/plain"},
            headers=_auth(key="artifact-1"),
        )

        _, _, text = _sse_request(app, token=TOKEN_ACME)
        kinds = [frame["event"] for frame in _parse_sse(text)]
        assert kinds == ["run_queued", "run_started", "artifact_created"]
    finally:
        app.close()


def test_sse_idempotent_replay_does_not_double_emit(tmp_path):
    app = make_app(tmp_path)
    try:
        artifact = {"data_base64": "dmVyaWZpZWQ=", "media_type": "text/plain"}
        _json_request(app, "POST", "/v1/artifacts", body=artifact, headers=_auth(key="artifact-1"))
        # Same idempotency key replays the stored response and must NOT re-emit.
        _json_request(app, "POST", "/v1/artifacts", body=artifact, headers=_auth(key="artifact-1"))

        _, _, text = _sse_request(app, token=TOKEN_ACME)
        assert len(_parse_sse(text)) == 1
    finally:
        app.close()


def test_sse_resume_via_last_event_id_returns_only_newer_events(tmp_path):
    app = make_app(tmp_path)
    try:
        _, _, created = _json_request(
            app, "POST", "/v1/runs", body={"name": "resume-run"}, headers=_auth(key="run-1")
        )
        _json_request(
            app,
            "PATCH",
            f"/v1/runs/{created['id']}",
            body={"status": "running"},
            headers=_auth(key="run-patch-1"),
        )

        # Resume after sequence 1 -> only the second event (run_started, id 2).
        _, _, text = _sse_request(app, token=TOKEN_ACME, last_event_id=1)
        frames = _parse_sse(text)
        assert len(frames) == 1
        assert frames[0]["id"] == "2"
        assert frames[0]["event"] == "run_started"

        # Resuming at the tail yields no frames (just the open comment).
        _, _, tail = _sse_request(app, token=TOKEN_ACME, last_event_id=2)
        assert _parse_sse(tail) == []
    finally:
        app.close()


def test_sse_rejects_malformed_last_event_id(tmp_path):
    app = make_app(tmp_path)
    try:
        environ = {
            "REQUEST_METHOD": "GET",
            "PATH_INFO": "/v1/events",
            "QUERY_STRING": "",
            "CONTENT_LENGTH": "0",
            "wsgi.input": io.BytesIO(b""),
            "HTTP_ACCEPT": "text/event-stream",
            "HTTP_AUTHORIZATION": f"Bearer {TOKEN_ACME}",
            "HTTP_LAST_EVENT_ID": "not-a-number",
        }
        captured = {}
        body = b"".join(
            app(environ, lambda status, hs: captured.update(status=status, headers=dict(hs)))
        )
        assert str(captured["status"]).startswith("400")
        assert json.loads(body)["error"]["code"] == "invalid_last_event_id"
    finally:
        app.close()


def test_sse_enforces_tenant_isolation(tmp_path):
    app = make_app(tmp_path)
    try:
        # acme creates a run; globex must never observe it.
        _json_request(app, "POST", "/v1/runs", body={"name": "acme-run"}, headers=_auth(TOKEN_ACME, key="a-1"))

        _, _, acme_text = _sse_request(app, token=TOKEN_ACME)
        assert len(_parse_sse(acme_text)) == 1

        _, _, globex_text = _sse_request(app, token=TOKEN_GLOBEX)
        assert _parse_sse(globex_text) == []
    finally:
        app.close()
