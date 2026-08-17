"""Tests for the discovery-as-a-service run workflow.

These exercise the real WSGI transport through the shared harness: unit checks of
config normalisation and route recognition (native-agnostic), the request-shape
error contracts that surface before the native probe (401/403/404/422), and --
guarded with ``skipif`` -- the native-absent 503 path and the full native-present
loop (submit -> poll -> fetch world -> explain/report).
"""

from __future__ import annotations

import io
import math
import time

import pytest
from _harness import TOKEN, auth, make_app, request

from lawsynth_api import discovery

# Read-only and scope-less tokens in the same tenant as the harness' default
# read+write TOKEN, to probe scope authorization independent of tenancy.
READ_ONLY = "11112222333344445555666677778888"
NO_SCOPE = "aaaabbbbccccddddeeeeffff000011112"
_EXTRA = {
    READ_ONLY: ("acme", frozenset({"read"})),
    NO_SCOPE: ("acme", frozenset()),
}


def _native_present() -> bool:
    return discovery.native_available()


def _make(tmp_path):
    # Discovery payloads carry inline observation series, so raise the request
    # ceiling above the harness' tiny default.
    return make_app(tmp_path, extra_tokens=_EXTRA, max_bytes=1_000_000)


def _decay(samples: int = 60, dt: float = 0.1, rate: float = 0.5):
    """A clean single-state exponential-decay series discovery recovers well."""

    times = [round(i * dt, 6) for i in range(samples)]
    return times, {"x": [math.exp(-rate * t) for t in times]}


def _discovery_body(**overrides):
    times, columns = _decay()
    body = {
        "name": overrides.pop("name", "decay-run"),
        "dataset": {"time": times, "columns": columns},
        "states": ["x"],
        "discovery": {"polynomial_degree": 2, "threshold": 0.05},
    }
    body.update(overrides)
    return body


def _poll(app, run_id, *, token=TOKEN, attempts=400, delay=0.01):
    """Poll GET /v1/runs/{id} until the run reaches a terminal status."""

    for _ in range(attempts):
        status, _, run = request(app, "GET", f"/v1/runs/{run_id}", headers=auth(token=token))
        assert status == 200, run
        if run["status"] in {"succeeded", "failed", "cancelled"}:
            return run
        time.sleep(delay)
    raise AssertionError(f"run {run_id} did not terminate; last status {run['status']}")


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
    captured: dict = {}
    payload = b"".join(app(environ, lambda status, hs: captured.update(status=status)))
    return payload.decode("utf-8")


# --------------------------------------------------------------------------- #
# Unit: config normalisation + native probe (native-agnostic)                 #
# --------------------------------------------------------------------------- #


def test_normalize_config_expands_alias_and_recipe():
    merged = discovery._normalize_config({"recipe": "ecology", "degree": 3})
    # recipe seeds a base; explicit degree alias overrides polynomial_degree.
    assert merged["polynomial_degree"] == 3
    assert merged["solver"] == "stlsq"


def test_normalize_config_rejects_unknown_option():
    with pytest.raises(Exception):
        discovery._normalize_config({"nonsense": 1})


def test_normalize_config_rejects_conflicting_degree_aliases():
    with pytest.raises(Exception):
        discovery._normalize_config({"degree": 2, "polynomial_degree": 3})


def test_native_available_returns_bool():
    assert isinstance(discovery.native_available(), bool)


# --------------------------------------------------------------------------- #
# Request-shape contracts that surface before the native probe                #
# (run identically whether or not native is installed)                        #
# --------------------------------------------------------------------------- #


def test_submit_requires_authentication_401(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, body = request(app, "POST", "/v1/runs", body=_discovery_body())
        assert status == 401 and body["error"]["code"]
    finally:
        app.close()


def test_submit_forbidden_without_write_scope_403(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, body = request(
            app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(token=READ_ONLY, key="d-403")
        )
        assert status == 403 and body["error"]["code"] == "forbidden"
    finally:
        app.close()


def test_submit_requires_idempotency_key_422(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, body = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth())
        assert status == 422 and body["error"]["code"] == "validation_error"
    finally:
        app.close()


def test_submit_missing_states_422(tmp_path):
    app = _make(tmp_path)
    try:
        body = _discovery_body()
        body.pop("states")
        status, _, out = request(app, "POST", "/v1/runs", body=body, headers=auth(key="d-nostate"))
        assert status == 422 and out["error"]["code"] == "validation_error"
    finally:
        app.close()


def test_submit_conflicting_dataset_refs_422(tmp_path):
    app = _make(tmp_path)
    try:
        body = _discovery_body()
        body["dataset_id"] = "some-id"
        status, _, out = request(app, "POST", "/v1/runs", body=body, headers=auth(key="d-both"))
        assert status == 422 and out["error"]["code"] == "validation_error"
    finally:
        app.close()


def test_submit_unknown_dataset_id_404(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, out = request(
            app,
            "POST",
            "/v1/runs",
            body={"name": "r", "dataset_id": "does-not-exist", "states": ["x"]},
            headers=auth(key="d-404"),
        )
        assert status == 404 and out["error"]["code"] == "not_found"
    finally:
        app.close()


def test_run_world_unknown_run_404(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, out = request(app, "GET", "/v1/runs/does-not-exist/world", headers=auth())
        assert status == 404 and out["error"]["code"] == "not_found"
    finally:
        app.close()


def test_run_world_requires_authentication_401(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, out = request(app, "GET", "/v1/runs/whatever/world")
        assert status == 401
    finally:
        app.close()


def test_plain_run_create_still_flows_through_domain(tmp_path):
    # A run with no dataset reference is not a discovery submit: it must keep the
    # existing domain behaviour (queued record, no native involvement).
    app = _make(tmp_path)
    try:
        status, _, run = request(app, "POST", "/v1/runs", body={"name": "plain"}, headers=auth(key="plain-1"))
        assert status == 201 and run["status"] == "queued"
        assert "world_id" not in run or not run["world_id"]
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# Native-absent: the honest 503 boundary                                      #
# --------------------------------------------------------------------------- #


@pytest.mark.skipif(_native_present(), reason="native runtime is installed")
def test_submit_reports_missing_native_503(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, out = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(key="d-503"))
        assert status == 503 and out["error"]["code"] == "native_unavailable"
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# Native-present: the full discovery loop                                     #
# --------------------------------------------------------------------------- #


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_inline_discovery_full_loop(tmp_path):
    app = _make(tmp_path)
    try:
        status, _, run = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(key="loop-1"))
        assert status == 201 and run["status"] == "queued"
        run_id = run["id"]

        done = _poll(app, run_id)
        assert done["status"] == "succeeded"
        assert isinstance(done["world_id"], str) and done["world_id"]
        summary = done["metadata"]["summary"]
        assert summary["laws"] == 1
        assert summary["complexity"]["laws"] == 1
        assert summary["mse"] is not None and summary["mse"] >= 0.0
        assert summary["world_id"] == done["world_id"]

        # GET /v1/runs/{id}/world returns the discovered world + product links.
        status, _, world = request(app, "GET", f"/v1/runs/{run_id}/world", headers=auth())
        assert status == 200
        world_id = world["world_id"]
        assert world_id == done["world_id"]
        assert "x" in world["world"]["equations"]
        assert world["links"]["explain"] == f"/v1/worlds/{world_id}/explain"

        # A completed run flows straight into explain and report.
        status, _, explained = request(app, "GET", f"/v1/worlds/{world_id}/explain", headers=auth())
        assert status == 200 and explained["variables"] == ["x"]

        status, headers, payload = _raw_get(app, f"/v1/worlds/{world_id}/report")
        assert status == 200 and headers["Content-Type"].startswith("text/html")
        assert b"LawSynth World" in payload

        # The lifecycle projected onto the SSE stream.
        kinds = [line[len("event: "):] for line in _sse(app).splitlines() if line.startswith("event: ")]
        assert kinds == ["run_queued", "run_started", "run_succeeded"]
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_uploaded_dataset_discovery(tmp_path):
    app = _make(tmp_path)
    try:
        times, columns = _decay()
        status, _, dataset = request(
            app,
            "POST",
            "/v1/datasets",
            body={"name": "uploaded-decay", "schema": ["x"], "time": times, "columns": columns},
            headers=auth(key="ds-1"),
        )
        assert status == 201
        status, _, run = request(
            app,
            "POST",
            "/v1/runs",
            body={"name": "from-upload", "dataset_id": dataset["id"], "states": ["x"]},
            headers=auth(key="up-run-1"),
        )
        assert status == 201
        done = _poll(app, run["id"])
        assert done["status"] == "succeeded"
        assert done["dataset_id"] == dataset["id"]
        assert done["metadata"]["summary"]["laws"] >= 1
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_inline_csv_discovery(tmp_path):
    app = _make(tmp_path)
    try:
        times, columns = _decay(samples=40)
        rows = ["t,x"] + [f"{times[i]},{columns['x'][i]}" for i in range(len(times))]
        status, _, run = request(
            app,
            "POST",
            "/v1/runs",
            body={"name": "csv-run", "dataset": {"csv": "\n".join(rows)}, "states": ["x"]},
            headers=auth(key="csv-1"),
        )
        assert status == 201
        done = _poll(app, run["id"])
        assert done["status"] == "succeeded" and done["world_id"]
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_submit_is_idempotent_on_repeated_key(tmp_path):
    app = _make(tmp_path)
    try:
        body = _discovery_body(name="idem")
        status, _, first = request(app, "POST", "/v1/runs", body=body, headers=auth(key="same-key"))
        assert status == 201
        status, headers, second = request(app, "POST", "/v1/runs", body=body, headers=auth(key="same-key"))
        assert status == 201
        assert second["id"] == first["id"]
        assert headers.get("Idempotency-Replayed") == "true"
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_run_fails_honestly_on_bad_solver(tmp_path):
    app = _make(tmp_path)
    try:
        body = _discovery_body(name="doomed")
        body["discovery"] = {"solver": "bogus"}
        status, _, run = request(app, "POST", "/v1/runs", body=body, headers=auth(key="fail-1"))
        assert status == 201 and run["status"] == "queued"

        done = _poll(app, run["id"])
        assert done["status"] == "failed"
        assert isinstance(done["metadata"]["error"], str) and done["metadata"]["error"]

        # No world was fabricated: fetching the run's world is a 409, not a 200.
        status, _, out = request(app, "GET", f"/v1/runs/{run['id']}/world", headers=auth())
        assert status == 409 and out["error"]["code"] == "conflict"

        # The failure projected onto the SSE stream.
        kinds = [line[len("event: "):] for line in _sse(app).splitlines() if line.startswith("event: ")]
        assert kinds == ["run_queued", "run_started", "run_failed"]
    finally:
        app.close()


def _raw_get(app, path, *, token=TOKEN):
    """Drive a GET and return the undecoded body (for HTML report)."""

    environ = {
        "REQUEST_METHOD": "GET",
        "PATH_INFO": path,
        "QUERY_STRING": "",
        "CONTENT_LENGTH": "0",
        "wsgi.input": io.BytesIO(b""),
        "HTTP_AUTHORIZATION": f"Bearer {token}",
    }
    captured: dict = {}

    def start_response(status, response_headers):
        captured["status"], captured["headers"] = status, dict(response_headers)

    payload = b"".join(app(environ, start_response))
    return int(str(captured["status"]).split(" ", 1)[0]), captured["headers"], payload
