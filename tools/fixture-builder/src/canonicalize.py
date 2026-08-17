"""Canonical JSON encoding for reproducible fixtures.

The repository writes JSON artifacts with sorted keys, two-space indentation, a
single trailing newline, and LF line endings (see ``benchmarks/_common.py``).
This module centralises that convention and adds stable float formatting so that
regenerating a fixture on any platform yields byte-identical output.
"""

from __future__ import annotations

import json
import math
from typing import Any


def _normalize(value: Any) -> Any:
    """Recursively normalise floats to a stable, round-trippable form."""
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ValueError(f"non-finite float is not allowed in a fixture: {value!r}")
        # Round-trip through repr to avoid platform-specific trailing digits,
        # collapsing integral floats (2.0) to keep output tidy but valid JSON.
        if value == int(value) and abs(value) < 1e16:
            return float(int(value))
        return float(repr(value))
    if isinstance(value, dict):
        return {str(key): _normalize(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [_normalize(item) for item in value]
    return value


def canonical_json(value: Any) -> str:
    """Return the canonical JSON text for ``value`` (with trailing newline)."""
    normalized = _normalize(value)
    return json.dumps(normalized, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def canonical_bytes(value: Any) -> bytes:
    """Return canonical JSON encoded as UTF-8 bytes for hashing and writing."""
    return canonical_json(value).encode("utf-8")
