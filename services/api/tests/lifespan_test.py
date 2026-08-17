"""Unit tests for the HTTP-process lifecycle owner."""

from __future__ import annotations

import pytest

from lawsynth_api.lifespan import ApiLifespan
from lawsynth_server.settings import Settings as ServerSettings


def _settings(tmp_path) -> ServerSettings:
    return ServerSettings(
        database_url=f"sqlite:///{tmp_path / 'metadata.sqlite3'}",
        object_root=tmp_path / "objects",
        tokens={},
        max_upload_bytes=1024,
    )


def test_application_is_available_until_closed(tmp_path):
    lifespan = ApiLifespan(_settings(tmp_path))
    assert lifespan.application is not None
    lifespan.close()
    with pytest.raises(RuntimeError):
        _ = lifespan.application


def test_close_is_idempotent(tmp_path):
    lifespan = ApiLifespan(_settings(tmp_path))
    lifespan.close()
    lifespan.close()  # second close must not raise


def test_context_manager_closes_on_exit(tmp_path):
    with ApiLifespan(_settings(tmp_path)) as lifespan:
        assert lifespan.application is not None
    with pytest.raises(RuntimeError):
        _ = lifespan.application


def test_accepts_injected_application(tmp_path):
    from lawsynth_server.app import Application

    domain = Application(_settings(tmp_path))
    lifespan = ApiLifespan(_settings(tmp_path), domain)
    try:
        assert lifespan.application is domain
    finally:
        lifespan.close()
