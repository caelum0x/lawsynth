from pathlib import Path

import pytest

from lawsynth_server.app import create_app
from lawsynth_server.settings import Settings


@pytest.fixture()
def settings(tmp_path: Path) -> Settings:
    return Settings(object_root=tmp_path / "objects", tokens={"writer": ("org-a", frozenset({"read", "write"})), "reader": ("org-b", frozenset({"read"}))})


@pytest.fixture()
def app(settings):
    return create_app(settings)


def request(method, path, *, token="writer", body=None, key="k-1", query=None):
    headers = {"Authorization": f"Bearer {token}"}
    if method != "GET":
        headers["Idempotency-Key"] = key
    return {"method": method, "path": path, "headers": headers, "body": body, "query": query or {}}
