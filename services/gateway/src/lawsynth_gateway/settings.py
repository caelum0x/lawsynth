"""Immutable settings for the gateway's request-admission boundary."""

from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Mapping
from urllib.parse import urlsplit


class GatewayConfigurationError(ValueError):
    """Raised when an unsafe gateway configuration is requested."""


def _positive(values: Mapping[str, str], name: str, default: int) -> int:
    raw = values.get(name, str(default))
    try:
        value = int(raw)
    except ValueError as error:
        raise GatewayConfigurationError(f"{name} must be an integer") from error
    if value < 1:
        raise GatewayConfigurationError(f"{name} must be positive")
    return value


def _origins(raw: str) -> frozenset[str]:
    result: set[str] = set()
    for origin in (item.strip() for item in raw.split(",")):
        if not origin:
            continue
        parsed = urlsplit(origin)
        if parsed.scheme not in {"http", "https"} or not parsed.netloc or parsed.path or parsed.query or parsed.fragment:
            raise GatewayConfigurationError("allowed origins must be bare http(s) origins")
        result.add(origin)
    return frozenset(result)


@dataclass(frozen=True, slots=True)
class GatewaySettings:
    max_body_bytes: int = 64 * 1024 * 1024
    max_header_bytes: int = 32 * 1024
    max_headers: int = 64
    max_clients: int = 10_000
    requests_per_window: int = 120
    rate_window_seconds: float = 60.0
    allowed_origins: frozenset[str] = frozenset()
    api_prefix: str = "/v1"

    def __post_init__(self) -> None:
        if self.max_body_bytes < 1 or self.max_header_bytes < 1 or self.max_headers < 1:
            raise GatewayConfigurationError("body and header limits must be positive")
        if self.max_clients < 1 or self.requests_per_window < 1 or self.rate_window_seconds <= 0:
            raise GatewayConfigurationError("rate limit settings must be positive")
        if not self.api_prefix.startswith("/") or self.api_prefix.endswith("/"):
            raise GatewayConfigurationError("api_prefix must start with one slash and not end with one")

    @classmethod
    def from_environment(cls, values: Mapping[str, str] | None = None) -> "GatewaySettings":
        values = os.environ if values is None else values
        return cls(
            max_body_bytes=_positive(values, "LAWSYNTH_GATEWAY_MAX_BODY_BYTES", 64 * 1024 * 1024),
            max_header_bytes=_positive(values, "LAWSYNTH_GATEWAY_MAX_HEADER_BYTES", 32 * 1024),
            max_headers=_positive(values, "LAWSYNTH_GATEWAY_MAX_HEADERS", 64),
            max_clients=_positive(values, "LAWSYNTH_GATEWAY_MAX_CLIENTS", 10_000),
            requests_per_window=_positive(values, "LAWSYNTH_GATEWAY_REQUESTS_PER_WINDOW", 120),
            rate_window_seconds=float(_positive(values, "LAWSYNTH_GATEWAY_RATE_WINDOW_SECONDS", 60)),
            allowed_origins=_origins(values.get("LAWSYNTH_GATEWAY_ALLOWED_ORIGINS", "")),
        )
