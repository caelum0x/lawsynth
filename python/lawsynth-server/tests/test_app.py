import base64

from conftest import request


def test_health_and_project_lifecycle(app):
    assert app.dispatch({"method": "GET", "path": "/health"})["body"]["status"] == "ok"
    created = app.dispatch(request("POST", "/projects", body={"name": "climate"}))
    assert created["status"] == 201
    listed = app.dispatch(request("GET", "/projects"))
    assert listed["body"]["items"][0]["id"] == created["body"]["id"]


def test_crud_idempotency_and_artifacts(app):
    created = app.dispatch(request("POST", "/projects", body={"name": "v1"}, key="create"))
    identifier = created["body"]["id"]
    assert app.dispatch(request("PATCH", f"/projects/{identifier}", body={"metadata": {"stage": "review"}}, key="patch"))["body"]["metadata"]["stage"] == "review"
    artifact = app.dispatch(request("POST", "/artifacts", body={"data_base64": base64.b64encode(b"verified").decode(), "media_type": "text/plain"}, key="artifact"))
    assert artifact["body"]["size"] == 8
    assert app.dispatch(request("DELETE", f"/projects/{identifier}", body={}, key="delete"))["status"] == 204


def test_app_rejects_invalid_artifact_encoding(app):
    response = app.dispatch(request("POST", "/artifacts", body={"data_base64": "***"}, key="bad-artifact"))
    assert response["status"] == 422
