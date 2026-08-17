"""Unit tests for API process settings and token parsing."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from lawsynth_api.settings import ApiSettings, _tokens
from lawsynth_server.errors import ValidationError

VALID_TOKEN = "0123456789abcdef0123456789abcdef"


def test_from_environment_defaults_are_development_and_memory():
    settings = ApiSettings.from_environment({})
    assert settings.environment == "development"
    assert settings.server.database_url == ":memory:"
    assert settings.event_stream_retention == 1024


def test_from_environment_parses_tokens_and_limits():
    values = {
        "LAWSYNTH_API_TOKENS_JSON": json.dumps({VALID_TOKEN: {"organization_id": "acme", "scopes": ["read", "write"]}}),
        "LAWSYNTH_MAX_UPLOAD_BYTES": "2048",
        "LAWSYNTH_API_MAX_REQUEST_BYTES": "4096",
        "LAWSYNTH_API_EVENT_RETENTION": "16",
    }
    settings = ApiSettings.from_environment(values)
    assert settings.server.tokens[VALID_TOKEN] == ("acme", frozenset({"read", "write"}))
    assert settings.max_request_bytes == 4096
    assert settings.event_stream_retention == 16


def test_production_rejects_volatile_storage():
    with pytest.raises(ValidationError) as excinfo:
        ApiSettings.from_environment({"LAWSYNTH_API_ENV": "production"})
    assert excinfo.value.code == "validation_error"


def test_production_requires_absolute_object_root():
    with pytest.raises(ValidationError):
        ApiSettings.from_environment(
            {
                "LAWSYNTH_API_ENV": "production",
                "LAWSYNTH_DATABASE_URL": "sqlite:///tmp/metadata.sqlite3",
                "LAWSYNTH_OBJECT_ROOT": "relative/objects",
            }
        )


def test_request_limit_cannot_be_below_upload_limit():
    from lawsynth_server.settings import Settings as ServerSettings

    server = ServerSettings(object_root=Path("/tmp/objects"), max_upload_bytes=4096)
    with pytest.raises(ValidationError):
        ApiSettings(server=server, environment="test", max_request_bytes=1024)


def test_tokens_rejects_short_token():
    with pytest.raises(ValidationError):
        _tokens(json.dumps({"short": {"organization_id": "acme", "scopes": ["read"]}}))


def test_tokens_rejects_unknown_scope():
    with pytest.raises(ValidationError):
        _tokens(json.dumps({VALID_TOKEN: {"organization_id": "acme", "scopes": ["superuser"]}}))


def test_tokens_empty_input_is_empty_map():
    assert _tokens(None) == {}
    assert _tokens("  ") == {}
