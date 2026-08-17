"""Tests for the datasets resource module and its live WSGI surface."""

from __future__ import annotations

from _harness import auth, make_app, request

from lawsynth_api import datasets


def test_classify_covers_dataset_routes():
    assert datasets.classify("GET", ["datasets"]) == "datasets.list"
    assert datasets.classify("POST", ["datasets"]) == "datasets.create"
    assert datasets.classify("DELETE", ["datasets", "id"]) == "datasets.delete"


def test_datasets_have_no_streaming_projection():
    assert datasets.lifecycle_events("POST", {"id": "d1"}) == []


def test_dataset_create_validates_schema_via_domain(tmp_path):
    app = make_app(tmp_path)
    try:
        # Valid: non-empty unique schema.
        status, _, created = request(
            app, "POST", "/v1/datasets", body={"name": "series", "schema": ["t", "x"]}, headers=auth(key="d-1")
        )
        assert status == 201 and created["schema"] == ["t", "x"]

        # Invalid: schema must be a non-empty list of names (domain 422).
        status, _, body = request(
            app, "POST", "/v1/datasets", body={"name": "bad", "schema": []}, headers=auth(key="d-2")
        )
        assert status == 422 and body["error"]["code"] == "validation_error"
    finally:
        app.close()
