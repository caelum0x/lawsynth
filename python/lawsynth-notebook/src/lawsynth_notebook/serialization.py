"""Strict JSON handling for decoded, inspectable artifacts."""

from __future__ import annotations

import json
import math
from collections.abc import Mapping
from pathlib import Path
from typing import Any

from .errors import ArtifactValidationError


def require_mapping(value: Any, label: str = "artifact") -> Mapping[str, Any]:
    if not isinstance(value, Mapping) or not all(isinstance(key, str) for key in value):
        raise ArtifactValidationError(f"{label} must be an object with string keys")
    return value


def canonical_json(value: Any) -> str:
    """Return stable JSON and reject NaN/infinite values that cannot round-trip."""
    try:
        return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False)
    except (TypeError, ValueError) as error:
        raise ArtifactValidationError(f"artifact is not canonical JSON: {error}") from error


def load_json(source: str | Path | Mapping[str, Any]) -> Mapping[str, Any]:
    if isinstance(source, Mapping):
        return require_mapping(source)
    path = Path(source)
    try:
        parsed = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ArtifactValidationError(f"cannot read JSON artifact {path}: {error}") from error
    return require_mapping(parsed, str(path))


def finite_number(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise ArtifactValidationError(f"{label} must be a finite number")
    return float(value)
