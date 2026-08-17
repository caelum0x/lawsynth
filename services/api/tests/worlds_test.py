"""Tests for the worlds resource module and its live WSGI surface."""

from __future__ import annotations

from _harness import auth, make_app, request

from lawsynth_api import worlds

_WORLD = {
    "name": "decay",
    "states": ["x"],
    "controls": [],
    "parameters": {"rate": 0.2},
    "equations": {"x": "-rate * x"},
}


def test_classify_covers_world_routes_and_simulate_action():
    assert worlds.classify("GET", ["worlds"]) == "worlds.list"
    assert worlds.classify("POST", ["worlds"]) == "worlds.create"
    assert worlds.classify("GET", ["worlds", "id"]) == "worlds.get"
    # The simulate action is delegated to the simulations module.
    assert worlds.classify("POST", ["worlds", "id", "simulate"]) == "worlds.simulate"


def test_worlds_have_no_streaming_projection():
    assert worlds.lifecycle_events("POST", {"id": "w1"}) == []


def test_world_create_roundtrip(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, created = request(app, "POST", "/v1/worlds", body=_WORLD, headers=auth(key="w-1"))
        assert status == 201 and created["states"] == ["x"]
        status, _, fetched = request(app, "GET", f"/v1/worlds/{created['id']}", headers=auth())
        assert status == 200 and fetched["id"] == created["id"]
    finally:
        app.close()


def test_world_simulate_reports_missing_native_runtime(tmp_path):
    # Native runtime is absent in this environment; the API must surface a 503
    # rather than fabricate a trajectory.
    app = make_app(tmp_path)
    try:
        _, _, created = request(app, "POST", "/v1/worlds", body=_WORLD, headers=auth(key="w-2"))
        status, _, body = request(
            app,
            "POST",
            f"/v1/worlds/{created['id']}/simulate",
            body={"initial": {"x": 1.0}, "horizon": 1.0, "step": 0.1},
            headers=auth(key="sim-1"),
        )
        assert status == 503 and body["error"]["code"] == "native_unavailable"
    finally:
        app.close()
