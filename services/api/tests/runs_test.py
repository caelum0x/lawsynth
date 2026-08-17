"""Tests for the runs resource module and its live WSGI/SSE surface."""

from __future__ import annotations

import io
import json

from _harness import TOKEN, auth, make_app, request

from lawsynth_api import runs
from lawsynth_api.events import EventKind


def test_status_kinds_cover_every_terminal_and_transient_state():
    assert runs.STATUS_KINDS["queued"] is EventKind.RUN_QUEUED
    assert runs.STATUS_KINDS["running"] is EventKind.RUN_STARTED
    assert runs.STATUS_KINDS["cancelled"] is EventKind.RUN_CANCELLED


def test_lifecycle_events_projects_known_status():
    events = runs.lifecycle_events("POST", {"id": "run-1", "status": "queued"})
    assert len(events) == 1
    kind, payload, run_id = events[0]
    assert kind is EventKind.RUN_QUEUED and run_id == "run-1"
    assert json.loads(payload) == {"id": "run-1", "status": "queued"}


def test_lifecycle_events_ignores_unknown_status_and_reads():
    assert runs.lifecycle_events("POST", {"id": "run-1", "status": "mystery"}) == []
    assert runs.lifecycle_events("GET", {"id": "run-1", "status": "queued"}) == []


def test_classify_covers_cancel_and_events_actions():
    assert runs.classify("POST", ["runs", "id", "cancel"]) == "runs.cancel"
    assert runs.classify("GET", ["runs", "id", "events"]) == "runs.events"
    assert runs.classify("POST", ["runs"]) == "runs.create"


def test_run_create_cancel_and_event_journal(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, run = request(app, "POST", "/v1/runs", body={"name": "cancel-me"}, headers=auth(key="run-1"))
        assert status == 201 and run["status"] == "queued"
        status, _, cancelled = request(app, "POST", f"/v1/runs/{run['id']}/cancel", headers=auth(key="cancel-1"))
        assert status == 200 and cancelled["status"] == "cancelled"

        status, _, listed = request(app, "GET", f"/v1/runs/{run['id']}/events", headers=auth())
        assert status == 200
        topics = {event["topic"] for event in listed["items"]}
        assert {"runs.created", "runs.cancelled"} <= topics
    finally:
        app.close()


def _sse(app, token=TOKEN):
    environ = {
        "REQUEST_METHOD": "GET",
        "PATH_INFO": "/v1/events",
        "QUERY_STRING": "",
        "CONTENT_LENGTH": "0",
        "wsgi.input": io.BytesIO(b""),
        "HTTP_ACCEPT": "text/event-stream",
        "HTTP_AUTHORIZATION": f"Bearer {token}",
    }
    captured = {}
    payload = b"".join(app(environ, lambda status, hs: captured.update(status=status, headers=dict(hs))))
    return payload.decode("utf-8")


def test_run_lifecycle_projects_onto_sse_stream(tmp_path):
    app = make_app(tmp_path)
    try:
        _, _, run = request(app, "POST", "/v1/runs", body={"name": "trace"}, headers=auth(key="run-1"))
        request(app, "POST", f"/v1/runs/{run['id']}/cancel", headers=auth(key="cancel-1"))
        text = _sse(app)
        kinds = [line[len("event: "):] for line in text.splitlines() if line.startswith("event: ")]
        assert kinds == ["run_queued", "run_cancelled"]
    finally:
        app.close()
