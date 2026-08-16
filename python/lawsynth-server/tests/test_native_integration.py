"""In-process service coverage for the real optional native execution boundary."""

from __future__ import annotations

from math import exp

import pytest

from conftest import request


def _native_extension_available() -> bool:
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name in {"lawsynth", "lawsynth._native"}:
            return False
        raise
    return True


def _observations() -> dict[str, object]:
    time = [index * 0.01 for index in range(101)]
    return {
        "name": "growth-observations",
        "schema": ["x"],
        "time": time,
        "columns": {"x": [exp(2.0 * point) for point in time]},
    }


def test_discovery_run_reports_a_specific_native_availability_boundary(app):
    if _native_extension_available():
        pytest.skip("this assertion covers source-only server deployments")
    dataset = app.dispatch(request("POST", "/datasets", body=_observations(), key="dataset"))
    assert dataset["status"] == 201

    response = app.dispatch(
        request(
            "POST",
            "/runs",
            body={"name": "discover-growth", "dataset_id": dataset["body"]["id"], "states": ["x"]},
            key="run",
        )
    )

    assert response["status"] == 503
    assert response["body"]["error"]["code"] == "native_unavailable"


def test_dataset_discovery_and_world_simulation_execute_the_installed_native_engine(app):
    if not _native_extension_available():
        pytest.skip("requires the actual built lawsynth native extension")
    dataset = app.dispatch(request("POST", "/v1/datasets", body=_observations(), key="dataset"))
    assert dataset["status"] == 201

    run = app.dispatch(
        request(
            "POST",
            "/v1/runs",
            body={
                "name": "discover-growth",
                "dataset_id": dataset["body"]["id"],
                "states": ["x"],
                "discovery": {"polynomial_degree": 1, "threshold": 0.01, "solver": "sr3"},
            },
            key="run",
        )
    )
    assert run["status"] == 201
    assert run["body"]["status"] == "succeeded"

    simulation = app.dispatch(
        request(
            "POST",
            f"/v1/worlds/{run['body']['world_id']}/simulate",
            body={"horizon": 0.1, "step": 0.01, "initial": {"x": 1.0}},
            key="simulate",
        )
    )
    assert simulation["status"] == 200
    assert simulation["body"]["time"][-1] == pytest.approx(0.1)
    assert simulation["body"]["values"]["x"][-1] > 1.1
