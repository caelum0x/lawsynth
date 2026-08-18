"""Coverage for the stability-analysis resource (``POST /v1/worlds/{id}/analysis/stability``).

Two tiers mirror :mod:`tests.test_native_integration`:

* **Engine-free** — request validation (missing box, unknown option, malformed
  interval), the box-dimension-mismatch ``422`` (checked against the stored
  world's state count before any engine work), auth enforcement, and — when the
  optional ``lawsynth`` runtime is absent from this env — the honest ``503``
  boundary. These run everywhere.
* **Live** — a stored deterministic world is analysed through the real
  CLI-backed engine, asserting the classified fixed point and the honest
  empty-box result. These **skip cleanly** when the native extension or the
  compiled CLI binary is unavailable (never a silent pass).
"""

from __future__ import annotations

import pytest

from conftest import request

from lawsynth_server.analysis import validate_stability_request
from lawsynth_server.errors import ValidationError


# --------------------------------------------------------------------------- #
# Engine discovery — mirrors tests/test_native_integration                     #
# --------------------------------------------------------------------------- #


def _native_extension_available() -> bool:
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name in {"lawsynth", "lawsynth._native"}:
            return False
        raise
    return True


def _cli_binary_available() -> bool:
    try:
        from lawsynth import analysis

        analysis._locate_binary()
    except Exception:
        return False
    return True


def _engine_available() -> bool:
    """The live stability path needs the native extension (build + save) and the CLI."""
    return _native_extension_available() and _cli_binary_available()


_STABLE_NODE = {
    "name": "stable-node",
    "states": ["x", "y"],
    "equations": {"x": "-x", "y": "-2*y"},
}


def _store_world(app, body=None, key="world"):
    response = app.dispatch(request("POST", "/v1/worlds", body=body or dict(_STABLE_NODE), key=key))
    assert response["status"] == 201, response
    return response["body"]["id"]


# --------------------------------------------------------------------------- #
# Request validation (no engine required)                                      #
# --------------------------------------------------------------------------- #


def test_validate_stability_request_normalizes_box_and_knobs():
    spec = validate_stability_request({"box": [[-1.0, 1.0], [-2.0, 2.0]], "grid": 7, "tolerance": 1e-9})
    assert spec["box"] == [(-1.0, 1.0), (-2.0, 2.0)]
    assert spec["grid"] == 7
    assert spec["tolerance"] == pytest.approx(1e-9)


def test_validate_stability_request_accepts_low_high_string():
    assert validate_stability_request({"box": "-1:1, -2:2"})["box"] == [(-1.0, 1.0), (-2.0, 2.0)]


def test_validate_stability_request_rejects_missing_box():
    with pytest.raises(ValidationError):
        validate_stability_request({"grid": 5})


def test_validate_stability_request_rejects_unknown_option():
    with pytest.raises(ValidationError):
        validate_stability_request({"box": [[-1.0, 1.0]], "wobble": 3})


def test_validate_stability_request_rejects_malformed_interval():
    with pytest.raises(ValidationError):
        validate_stability_request({"box": [[1.0, -1.0]]})  # lower above upper
    with pytest.raises(ValidationError):
        validate_stability_request({"box": [["a", "b"]]})  # non-numeric
    with pytest.raises(ValidationError):
        validate_stability_request({"box": []})  # empty


def test_validate_stability_request_rejects_bad_knob():
    with pytest.raises(ValidationError):
        validate_stability_request({"box": [[-1.0, 1.0]], "grid": 0})
    with pytest.raises(ValidationError):
        validate_stability_request({"box": [[-1.0, 1.0]], "tolerance": -1.0})


# --------------------------------------------------------------------------- #
# HTTP surface: bad requests and auth (no engine required)                     #
# --------------------------------------------------------------------------- #


def test_stability_missing_box_returns_422(app):
    world_id = _store_world(app)
    response = app.dispatch(request("POST", f"/v1/worlds/{world_id}/analysis/stability", body={}, key="s"))
    assert response["status"] == 422
    assert response["body"]["error"]["code"] == "validation_error"


def test_stability_wrong_dimension_box_returns_422(app):
    # The stored world has two states; a one-interval box cannot match it. The
    # dimension check runs against the stored declarative world, so it needs no
    # engine and returns a clear 4xx (not a 5xx traceback).
    world_id = _store_world(app)
    response = app.dispatch(
        request("POST", f"/v1/worlds/{world_id}/analysis/stability", body={"box": [[-1.0, 1.0]]}, key="s")
    )
    assert response["status"] == 422
    assert response["body"]["error"]["code"] == "validation_error"
    assert "state count" in response["body"]["error"]["message"]


def test_stability_requires_write_scope(app):
    world_id = _store_world(app)
    response = app.dispatch(
        request(
            "POST",
            f"/v1/worlds/{world_id}/analysis/stability",
            token="reader",
            body={"box": [[-1.0, 1.0], [-1.0, 1.0]]},
            key="s",
        )
    )
    assert response["status"] == 403
    assert response["body"]["error"]["code"] == "forbidden"


def test_stability_requires_authentication(app):
    world_id = _store_world(app)
    response = app.dispatch(
        {
            "method": "POST",
            "path": f"/v1/worlds/{world_id}/analysis/stability",
            "headers": {"Idempotency-Key": "s"},
            "body": {"box": [[-1.0, 1.0], [-1.0, 1.0]]},
        }
    )
    assert response["status"] == 401
    assert response["body"]["error"]["code"] == "authentication_required"


# --------------------------------------------------------------------------- #
# Engine-absence boundary — runs only when the runtime is unavailable           #
# --------------------------------------------------------------------------- #


def test_stability_reports_native_unavailable_when_engine_absent(app):
    if _engine_available():
        pytest.skip("this assertion covers source-only server deployments")
    world_id = _store_world(app)
    response = app.dispatch(
        request(
            "POST",
            f"/v1/worlds/{world_id}/analysis/stability",
            body={"box": [[-1.0, 1.0], [-1.0, 1.0]]},
            key="s",
        )
    )
    assert response["status"] == 503
    assert response["body"]["error"]["code"] == "native_unavailable"


# --------------------------------------------------------------------------- #
# Live engine — real CLI-backed stability (skip cleanly when absent)            #
# --------------------------------------------------------------------------- #


def test_stability_classifies_a_stored_stable_node(app):
    if not _engine_available():
        pytest.skip("requires the built lawsynth native extension and CLI binary")
    world_id = _store_world(app)
    response = app.dispatch(
        request(
            "POST",
            f"/v1/worlds/{world_id}/analysis/stability",
            body={"box": [[-1.0, 1.0], [-1.0, 1.0]]},
            key="s",
        )
    )
    assert response["status"] == 200
    body = response["body"]
    assert body["states"] == ["x", "y"]
    assert body["seeds_converged"] > 0
    assert len(body["fixed_points"]) == 1
    point = body["fixed_points"][0]
    assert point["classification"] == "stable node"
    assert point["inconclusive"] is False
    assert all(abs(value) < 1e-6 for value in point["coordinates"])
    assert point["state_values"] == {"x": pytest.approx(0.0), "y": pytest.approx(0.0)}
    # A stable node linearizes to all-real, all-negative eigenvalues.
    assert all(eig["im"] == 0.0 and eig["re"] < 0.0 for eig in point["eigenvalues"])
    assert response["headers"]["Idempotency-Replayed"] == "false"


def test_stability_classifies_a_saddle(app):
    if not _engine_available():
        pytest.skip("requires the built lawsynth native extension and CLI binary")
    world_id = _store_world(
        app,
        body={"name": "saddle", "states": ["x", "y"], "equations": {"x": "x", "y": "-y"}},
    )
    response = app.dispatch(
        request(
            "POST",
            f"/v1/worlds/{world_id}/analysis/stability",
            body={"box": [[-1.0, 1.0], [-1.0, 1.0]]},
            key="s",
        )
    )
    assert response["status"] == 200
    points = response["body"]["fixed_points"]
    assert len(points) == 1
    assert points[0]["classification"] == "saddle"
    # A saddle has one positive and one negative real eigenvalue.
    reals = sorted(eig["re"] for eig in points[0]["eigenvalues"])
    assert reals[0] < 0.0 < reals[-1]


def test_stability_empty_box_is_a_valid_empty_result(app):
    if not _engine_available():
        pytest.skip("requires the built lawsynth native extension and CLI binary")
    world_id = _store_world(app)
    # The node's only fixed point is the origin; a box away from it yields no
    # equilibrium inside the box. That is a valid 200 with an empty list plus the
    # honest seed accounting — not an error.
    response = app.dispatch(
        request(
            "POST",
            f"/v1/worlds/{world_id}/analysis/stability",
            body={"box": [[2.0, 3.0], [2.0, 3.0]]},
            key="s",
        )
    )
    assert response["status"] == 200
    assert response["body"]["fixed_points"] == []
    assert response["body"]["seeds_total"] > 0


def test_stability_is_idempotent(app):
    if not _engine_available():
        pytest.skip("requires the built lawsynth native extension and CLI binary")
    world_id = _store_world(app)
    payload = {"box": [[-1.0, 1.0], [-1.0, 1.0]]}
    first = app.dispatch(request("POST", f"/v1/worlds/{world_id}/analysis/stability", body=payload, key="rep"))
    second = app.dispatch(request("POST", f"/v1/worlds/{world_id}/analysis/stability", body=payload, key="rep"))
    assert first["status"] == second["status"] == 200
    assert second["headers"]["Idempotency-Replayed"] == "true"
    assert second["body"] == first["body"]
