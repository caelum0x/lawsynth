"""Configuration for the HTTP process, separate from domain configuration.

The domain service never reads environment variables.  Keeping this adapter at
the process boundary prevents request handlers from silently changing tenant or
storage configuration after startup.
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping

from lawsynth_server.errors import ValidationError
from lawsynth_server.settings import Settings as ServerSettings


def _positive_integer(values: Mapping[str, str], name: str, default: int) -> int:
    raw = values.get(name, str(default))
    try:
        value = int(raw)
    except ValueError as error:
        raise ValidationError(f"{name} must be an integer") from error
    if value < 1:
        raise ValidationError(f"{name} must be positive")
    return value


def _tokens(raw: str | None) -> dict[str, tuple[str, frozenset[str]]]:
    if raw is None or not raw.strip():
        return {}
    try:
        document = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValidationError("LAWSYNTH_API_TOKENS_JSON must be valid JSON") from error
    if not isinstance(document, dict):
        raise ValidationError("LAWSYNTH_API_TOKENS_JSON must map tokens to tenant grants")
    parsed: dict[str, tuple[str, frozenset[str]]] = {}
    for token, grant in document.items():
        if not isinstance(token, str) or len(token) < 16 or not isinstance(grant, dict):
            raise ValidationError("each API token must have a tenant grant")
        organization_id, scopes = grant.get("organization_id"), grant.get("scopes")
        if (
            not isinstance(organization_id, str)
            or not organization_id
            or not isinstance(scopes, list)
            or not scopes
            or any(scope not in {"read", "write", "admin"} for scope in scopes)
        ):
            raise ValidationError("token grants require organization_id and read/write/admin scopes")
        parsed[token] = (organization_id, frozenset(scopes))
    return parsed


@dataclass(frozen=True, slots=True)
class ApiSettings:
    """Immutable process settings used when constructing a WSGI application."""

    server: ServerSettings
    environment: str = "development"
    max_request_bytes: int = 64 * 1024 * 1024
    event_stream_retention: int = 1024

    def __post_init__(self) -> None:
        if self.environment not in {"development", "test", "staging", "production"}:
            raise ValidationError("environment must be development, test, staging, or production")
        if self.max_request_bytes < 1:
            raise ValidationError("max_request_bytes must be positive")
        if self.max_request_bytes < self.server.max_upload_bytes:
            raise ValidationError("max_request_bytes cannot be lower than max_upload_bytes")
        if self.event_stream_retention < 1:
            raise ValidationError("event_stream_retention must be positive")

    @classmethod
    def from_environment(cls, values: Mapping[str, str] | None = None) -> "ApiSettings":
        values = os.environ if values is None else values
        environment = values.get("LAWSYNTH_API_ENV", "development").lower()
        database_url = values.get("LAWSYNTH_DATABASE_URL", ":memory:")
        object_root = Path(values.get("LAWSYNTH_OBJECT_ROOT", ".lawsynth-objects"))
        upload_limit = _positive_integer(values, "LAWSYNTH_MAX_UPLOAD_BYTES", 64 * 1024 * 1024)
        request_limit = _positive_integer(values, "LAWSYNTH_API_MAX_REQUEST_BYTES", upload_limit)
        retention = _positive_integer(values, "LAWSYNTH_API_EVENT_RETENTION", 1024)
        if environment == "production":
            if database_url == ":memory:":
                raise ValidationError("LAWSYNTH_DATABASE_URL must use durable storage in production")
            if not object_root.is_absolute():
                raise ValidationError("LAWSYNTH_OBJECT_ROOT must be an absolute path in production")
        server = ServerSettings(
            database_url=database_url,
            object_root=object_root,
            tokens=_tokens(values.get("LAWSYNTH_API_TOKENS_JSON")),
            max_page_size=_positive_integer(values, "LAWSYNTH_MAX_PAGE_SIZE", 100),
            max_upload_bytes=upload_limit,
            telemetry_enabled=values.get("LAWSYNTH_TELEMETRY", "false").lower() == "true",
        )
        return cls(server=server, environment=environment, max_request_bytes=request_limit, event_stream_retention=retention)
