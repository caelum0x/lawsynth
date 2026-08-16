"""Format compatibility checks for already decoded metadata."""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any

from .errors import ArtifactValidationError


SUPPORTED_FORMAT_MAJOR = 1


def check_format(metadata: Mapping[str, Any]) -> int:
    value = metadata.get("format_version", 1)
    if isinstance(value, str):
        major_text = value.split(".", 1)[0]
        if not major_text.isdigit():
            raise ArtifactValidationError("format_version must begin with an integer")
        major = int(major_text)
    elif isinstance(value, int) and not isinstance(value, bool):
        major = value
    else:
        raise ArtifactValidationError("format_version must be an integer or version string")
    if major != SUPPORTED_FORMAT_MAJOR:
        raise ArtifactValidationError(f"format major {major} is not supported (expected {SUPPORTED_FORMAT_MAJOR})")
    return major
