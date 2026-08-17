"""Tests for the product feature endpoints (explain/forecast/report/compare).

These exercise the real WSGI transport through the shared harness: happy paths,
authentication (401), scope (403), unknown world (404), bad input (422), and the
native-absent 503 contract for the simulate-backed ``forecast``.
"""

from __future__ import annotations

import io
import json

import pytest
from _harness import TOKEN, auth, make_app, request

from lawsynth_api import products, worlds

# A single-state exponential-decay world with a parameterized law.
_WORLD = {
    "name": "decay",
    "states": ["x"],
    "controls": [],
    "parameters": {"rate": 0.2},
    "equations": {"x": "-rate * x"},
}

# A two-state world used for the compare diff (adds a variable, changes a param).
_WORLD2 = {
    "name": "coupled",
    "states": ["x", "y"],
    "controls": [],
    "parameters": {"rate": 0.5},
    "equations": {"x": "-rate * x", "y": "x - y"},
}

# A read-only and a scope-less token, both in the same tenant as the harness'
# default read+write TOKEN, to probe scope authorization independent of tenancy.
READ_ONLY = "11112222333344445555666677778888"
NO_SCOPE = "aaaabbbbccccddddeeeeffff000011112"
_EXTRA = {
    READ_ONLY: ("acme", frozenset({"read"})),
    NO_SCOPE: ("acme", frozenset()),
}


def _native_present() -> bool:
    try:
        import lawsynth

        _ = lawsynth.World
        return True
    except Exception:
        return False


def _make(tmp_path):
    return make_app(tmp_path, extra_tokens=_EXTRA)


def _create_world(app, world, key):
    status, _, created = request(app, "POST", "/v1/worlds", body=world, headers=auth(key=key))
    assert status == 201, created
    return created["id"]


def _raw_request(app, method: str, path: str, *, headers=None):
    """Drive the WSGI app and return the undecoded response body (for HTML)."""

    environ: dict[str, object] = {
        "REQUEST_METHOD": method,
        "PATH_INFO": path,
        "QUERY_STRING": "",
        "CONTENT_LENGTH": "0",
        "wsgi.input": io.BytesIO(b""),
    }
    for name, value in (headers or {}).items():
        environ[f"HTTP_{name.upper().replace('-', '_')}"] = value
    captured: dict[str, object] = {}

    def start_response(status, response_headers):
        captured["status"], captured["headers"] = status, dict(response_headers)

    payload = b"".join(app(environ, start_response))
    return int(str(captured["status"]).split(" ", 1)[0]), captured["headers"], payload


# --------------------------------------------------------------------------- #
# Route recognition (unit)                                                     #
# --------------------------------------------------------------------------- #


def test_match_recognizes_each_product_route():
    assert products.match("GET", ["worlds", "id", "explain"]) == "explain"
    assert products.match("POST", ["worlds", "id", "forecast"]) == "forecast"
    assert products.match("GET", ["worlds", "id", "report"]) == "report"
    assert products.match("POST", ["worlds", "compare"]) == "compare"


def test_match_rejects_unrelated_or_wrong_method_routes():
    assert products.match("POST", ["worlds", "id", "explain"]) is None
    assert products.match("GET", ["worlds", "id", "forecast"]) is None
    assert products.match("GET", ["worlds", "id"]) is None
    assert products.match("POST", ["runs", "compare"]) is None
    assert products.match("GET", ["worlds", "compare"]) is None


def test_worlds_classify_includes_product_labels():
    assert worlds.classify("GET", ["worlds", "id", "explain"]) == "worlds.explain"
    assert worlds.classify("POST", ["worlds", "id", "forecast"]) == "worlds.forecast"
    assert worlds.classify("GET", ["worlds", "id", "report"]) == "worlds.report"
    assert worlds.classify("POST", ["worlds", "compare"]) == "worlds.compare"


# --------------------------------------------------------------------------- #
# explain                                                                      #
# --------------------------------------------------------------------------- #


def test_explain_happy_path(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-explain")
        status, _, body = request(app, "GET", f"/v1/worlds/{world_id}/explain", headers=auth())
        assert status == 200
        assert body["variables"] == ["x"]
        assert body["parameters"] == {"rate": 0.2}
        assert body["complexity"]["laws"] == 1
        assert body["dependencies"] == {"x": ["x"]}
        assert body["laws"][0]["target"] == "x"
        assert body["laws"][0]["readable"].startswith("dx/dt =")
        assert body["assumptions"]
    finally:
        app.close()


def test_explain_requires_authentication(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-401")
        status, _, body = request(app, "GET", f"/v1/worlds/{world_id}/explain")
        assert status == 401 and body["error"]["code"]
    finally:
        app.close()


def test_explain_forbidden_without_read_scope(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-403")
        status, _, body = request(app, "GET", f"/v1/worlds/{world_id}/explain", headers=auth(token=NO_SCOPE))
        assert status == 403 and body["error"]["code"] == "forbidden"
    finally:
        app.close()


def test_explain_unknown_world_404(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, body = request(app, "GET", "/v1/worlds/does-not-exist/explain", headers=auth())
        assert status == 404 and body["error"]["code"] == "not_found"
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# forecast (simulate-backed)                                                   #
# --------------------------------------------------------------------------- #


def test_forecast_rejects_bad_input_422(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-fc-422")
        # Missing horizon -> validation fails before any native call.
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/forecast",
            body={"step": 0.1, "initial": {"x": 1.0}}, headers=auth(),
        )
        assert status == 422 and body["error"]["code"] == "validation_error"
    finally:
        app.close()


def test_forecast_requires_initial_state_422(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-fc-init")
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/forecast",
            body={"horizon": 1.0, "step": 0.1}, headers=auth(),
        )
        assert status == 422
    finally:
        app.close()


def test_forecast_forbidden_without_write_scope(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-fc-403")
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/forecast",
            body={"horizon": 1.0, "step": 0.1, "initial": {"x": 1.0}},
            headers=auth(token=READ_ONLY),
        )
        assert status == 403 and body["error"]["code"] == "forbidden"
    finally:
        app.close()


def test_forecast_unknown_world_404(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, body = request(
            app, "POST", "/v1/worlds/missing/forecast",
            body={"horizon": 1.0, "step": 0.1, "initial": {"x": 1.0}}, headers=auth(),
        )
        assert status == 404 and body["error"]["code"] == "not_found"
    finally:
        app.close()


@pytest.mark.skipif(_native_present(), reason="native runtime is installed")
def test_forecast_reports_missing_native_503(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-fc-503")
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/forecast",
            body={"horizon": 1.0, "step": 0.1, "initial": {"x": 1.0}}, headers=auth(),
        )
        assert status == 503 and body["error"]["code"] == "native_unavailable"
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_forecast_runs_when_native_present(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-fc-ok")
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/forecast",
            body={"horizon": 1.0, "step": 0.1, "initial": {"x": 1.0}}, headers=auth(),
        )
        assert status == 200
        assert body["trajectory"]["time"]
        assert "x" in body["trajectory"]["values"]
        assert body["interventions"] == []
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_forecast_applies_scheduled_intervention(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-fc-iv")
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/forecast",
            body={
                "horizon": 1.0, "step": 0.1, "initial": {"x": 1.0},
                "interventions": [{"at": 0.5, "parameters": {"rate": 2.0}}],
            },
            headers=auth(),
        )
        assert status == 200
        assert body["interventions"][0]["at"] == 0.5
        assert body["trajectory"]["time"][0] == 0.0
        assert body["trajectory"]["time"][-1] == pytest.approx(1.0, abs=1e-6)
    finally:
        app.close()


def test_forecast_rejects_intervention_outside_window_422(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-fc-iv-bad")
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/forecast",
            body={
                "horizon": 1.0, "step": 0.1, "initial": {"x": 1.0},
                "interventions": [{"at": 5.0, "parameters": {"rate": 2.0}}],
            },
            headers=auth(),
        )
        assert status == 422
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# report (HTML)                                                                #
# --------------------------------------------------------------------------- #


def test_report_returns_self_contained_html(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-report")
        status, headers, payload = _raw_request(app, "GET", f"/v1/worlds/{world_id}/report", headers=auth())
        assert status == 200
        assert headers["Content-Type"].startswith("text/html")
        text = payload.decode("utf-8")
        assert text.startswith("<!doctype html")
        assert "LawSynth World" in text
        assert "svg" in text  # inline chart / structure is embedded, no external assets
        assert "http://" not in text.replace("http://www.w3.org", "")  # no external asset URLs
    finally:
        app.close()


def test_report_requires_authentication_401(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-report-401")
        status, _, body = request(app, "GET", f"/v1/worlds/{world_id}/report")
        assert status == 401
    finally:
        app.close()


def test_report_forbidden_without_read_scope_403(tmp_path):
    app = _make(tmp_path)
    try:
        world_id = _create_world(app, _WORLD, "w-report-403")
        status, _, body = request(app, "GET", f"/v1/worlds/{world_id}/report", headers=auth(token=NO_SCOPE))
        assert status == 403 and body["error"]["code"] == "forbidden"
    finally:
        app.close()


def test_report_unknown_world_404(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, body = request(app, "GET", "/v1/worlds/nope/report", headers=auth())
        assert status == 404 and body["error"]["code"] == "not_found"
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# compare                                                                      #
# --------------------------------------------------------------------------- #


def test_compare_happy_path(tmp_path):
    app = _make(tmp_path)
    try:
        left = _create_world(app, _WORLD, "w-cmp-a")
        right = _create_world(app, _WORLD2, "w-cmp-b")
        status, _, body = request(
            app, "POST", "/v1/worlds/compare",
            body={"left": left, "right": right}, headers=auth(),
        )
        assert status == 200
        assert body["variables"]["added"] == ["y"]
        assert body["variables"]["common"] == ["x"]
        assert body["parameters"]["changed"]["rate"]["left"] == 0.2
        assert body["parameters"]["changed"]["rate"]["right"] == 0.5
        assert "y" in body["laws"]["added"]
        assert body["complexity_delta"]["laws"] == 1
    finally:
        app.close()


def test_compare_requires_authentication_401(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, body = request(app, "POST", "/v1/worlds/compare", body={"left": "a", "right": "b"})
        assert status == 401
    finally:
        app.close()


def test_compare_forbidden_without_read_scope_403(tmp_path):
    app = _make(tmp_path)
    try:
        left = _create_world(app, _WORLD, "w-cmp-403a")
        right = _create_world(app, _WORLD2, "w-cmp-403b")
        status, _, body = request(
            app, "POST", "/v1/worlds/compare",
            body={"left": left, "right": right}, headers=auth(token=NO_SCOPE),
        )
        assert status == 403
    finally:
        app.close()


def test_compare_unknown_world_404(tmp_path):
    app = _make(tmp_path)
    try:
        left = _create_world(app, _WORLD, "w-cmp-404")
        status, _, body = request(
            app, "POST", "/v1/worlds/compare",
            body={"left": left, "right": "missing"}, headers=auth(),
        )
        assert status == 404 and body["error"]["code"] == "not_found"
    finally:
        app.close()


def test_compare_bad_input_422(tmp_path):
    app = _make(tmp_path)
    try:
        left = _create_world(app, _WORLD, "w-cmp-422")
        status, _, body = request(
            app, "POST", "/v1/worlds/compare",
            body={"left": left}, headers=auth(),
        )
        assert status == 422 and body["error"]["code"] == "validation_error"
    finally:
        app.close()
