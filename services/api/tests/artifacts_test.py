"""Tests for the artifacts resource module and its live WSGI surface."""

from __future__ import annotations

import json

from _harness import auth, make_app, request

from lawsynth_api import artifacts, downloads
from lawsynth_api.events import EventKind

_ARTIFACT = {"data_base64": "dmVyaWZpZWQ=", "media_type": "text/plain"}


def test_classify_covers_create_and_download():
    assert artifacts.classify("POST", ["artifacts"]) == "artifacts.create"
    assert artifacts.classify("GET", ["artifacts", "sha"]) == "artifacts.download"


def test_lifecycle_events_projects_artifact_created_with_optional_run():
    events = artifacts.lifecycle_events("POST", {"id": "a1", "sha256": "abc", "run_id": "run-1"})
    assert len(events) == 1
    kind, payload, run_id = events[0]
    assert kind is EventKind.ARTIFACT_CREATED and run_id == "run-1"
    assert json.loads(payload) == {"id": "a1", "sha256": "abc"}


def test_lifecycle_events_ignores_non_post():
    assert artifacts.lifecycle_events("GET", {"id": "a1"}) == []


def test_artifact_download_headers_derives_etag_from_hash():
    sha = "0" * 64
    assert downloads.artifact_download_headers({"sha256": sha}) == {"ETag": f'"{sha}"'}
    assert downloads.artifact_download_headers({"sha256": "short"}) == {}
    assert downloads.artifact_download_headers("not-a-mapping") == {}


def test_artifact_put_get_roundtrip_and_etag_header(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, created = request(app, "POST", "/v1/artifacts", body=_ARTIFACT, headers=auth(key="a-1"))
        assert status == 201
        sha = created["sha256"]

        status, response_headers, fetched = request(app, "GET", f"/v1/artifacts/{sha}", headers=auth())
        assert status == 200
        assert fetched["data_base64"] == _ARTIFACT["data_base64"] and fetched["size"] == 8
        # The download is decorated with a strong ETag derived from the content.
        assert response_headers["ETag"] == f'"{sha}"'
    finally:
        app.close()
