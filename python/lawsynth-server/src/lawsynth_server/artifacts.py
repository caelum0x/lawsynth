"""Content-addressed artifact metadata and safe object references."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class Artifact:
    sha256: str
    size: int
    media_type: str


def artifact_from_bytes(data: bytes, media_type: str = "application/octet-stream") -> Artifact:
    if not data:
        raise ValidationError("empty artifacts are not accepted")
    if not media_type or "/" not in media_type:
        raise ValidationError("invalid media type")
    return Artifact(hashlib.sha256(data).hexdigest(), len(data), media_type)
