"""Tests for the projects resource module and its live WSGI surface."""

from __future__ import annotations

from _harness import auth, make_app, request

from lawsynth_api import projects


def test_classify_covers_collection_and_item_routes():
    assert projects.classify("GET", ["projects"]) == "projects.list"
    assert projects.classify("POST", ["projects"]) == "projects.create"
    assert projects.classify("GET", ["projects", "id"]) == "projects.get"
    assert projects.classify("PATCH", ["projects", "id"]) == "projects.update"
    assert projects.classify("DELETE", ["projects", "id"]) == "projects.delete"


def test_projects_have_no_streaming_projection():
    assert projects.lifecycle_events("POST", {"id": "p1"}) == []


def test_project_create_and_list_roundtrip(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, created = request(app, "POST", "/v1/projects", body={"name": "coastal"}, headers=auth(key="p-1"))
        assert status == 201 and created["name"] == "coastal"
        status, _, listed = request(app, "GET", "/v1/projects", headers=auth())
        assert status == 200 and listed["items"] == [created]
    finally:
        app.close()


def test_project_list_requires_authentication(tmp_path):
    app = make_app(tmp_path)
    try:
        status, _, _ = request(app, "GET", "/v1/projects")
        assert status == 401
    finally:
        app.close()
