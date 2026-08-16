"""Opaque, signed-free cursors for a single in-process repository ordering."""

from __future__ import annotations

import base64
import binascii
import json
from dataclasses import dataclass

from .errors import ValidationError


@dataclass(frozen=True, slots=True)
class Page:
    items: tuple[dict[str, object], ...]
    next_cursor: str | None


def decode_cursor(cursor: str | None) -> int:
    if cursor is None:
        return 0
    try:
        value = json.loads(base64.urlsafe_b64decode(cursor.encode() + b"=="))
        index = value["offset"]
        if not isinstance(index, int) or index < 0:
            raise ValueError
        return index
    except (ValueError, KeyError, TypeError, binascii.Error, json.JSONDecodeError) as exc:
        raise ValidationError("invalid pagination cursor") from exc


def encode_cursor(offset: int) -> str:
    return base64.urlsafe_b64encode(json.dumps({"offset": offset}, separators=(",", ":")).encode()).decode().rstrip("=")


def page(items: list[dict[str, object]], *, cursor: str | None, limit: int, maximum: int) -> Page:
    if not 1 <= limit <= maximum:
        raise ValidationError("page limit is outside allowed range", details={"maximum": maximum})
    start = decode_cursor(cursor)
    selection = tuple(items[start : start + limit])
    next_cursor = encode_cursor(start + limit) if start + limit < len(items) else None
    return Page(selection, next_cursor)
