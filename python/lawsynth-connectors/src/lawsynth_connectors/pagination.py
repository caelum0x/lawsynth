"""Opaque cursors and bounded page helpers shared by remote connectors."""

from __future__ import annotations

import base64
import hashlib
import hmac
import json
from collections.abc import Iterable, Iterator, Mapping, Sequence
from dataclasses import dataclass
from typing import Any, Generic, TypeVar

from .errors import DataValidationError

T = TypeVar("T")


@dataclass(frozen=True, slots=True)
class PageRequest:
    size: int = 100
    cursor: str | None = None

    def __post_init__(self) -> None:
        if not 1 <= self.size <= 10_000:
            raise ValueError("page size must be in 1..10,000")


@dataclass(frozen=True, slots=True)
class Page(Generic[T]):
    items: Sequence[T]
    next_cursor: str | None
    total: int | None = None

    @property
    def has_more(self) -> bool:
        return self.next_cursor is not None


class CursorCodec:
    """Encode cursor state with optional HMAC tamper protection."""

    def __init__(self, secret: bytes | None = None) -> None:
        if secret is not None and len(secret) < 16:
            raise ValueError("cursor signing secret must contain at least 16 bytes")
        self._secret = secret

    def encode(self, state: Mapping[str, Any]) -> str:
        payload = json.dumps(
            dict(state),
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=True,
        ).encode("utf-8")
        signature = self._signature(payload)
        envelope = len(signature).to_bytes(1, "big") + signature + payload
        return base64.urlsafe_b64encode(envelope).rstrip(b"=").decode("ascii")

    def decode(self, cursor: str) -> Mapping[str, Any]:
        try:
            padding = "=" * (-len(cursor) % 4)
            envelope = base64.urlsafe_b64decode(cursor + padding)
            signature_size = envelope[0]
            signature = envelope[1 : signature_size + 1]
            payload = envelope[signature_size + 1 :]
        except (ValueError, IndexError) as exc:
            raise DataValidationError("page cursor is malformed") from exc

        expected = self._signature(payload)
        if not hmac.compare_digest(signature, expected):
            raise DataValidationError("page cursor signature is invalid")

        try:
            state = json.loads(payload)
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise DataValidationError("page cursor payload is invalid") from exc
        if not isinstance(state, dict):
            raise DataValidationError("page cursor state must be an object")
        return state

    def _signature(self, payload: bytes) -> bytes:
        if self._secret is None:
            return b""
        return hmac.new(self._secret, payload, hashlib.sha256).digest()


def paginate_sequence(
    values: Sequence[T],
    request: PageRequest,
    *,
    codec: CursorCodec | None = None,
) -> Page[T]:
    """Return a stable offset page for an already materialized sequence."""
    codec = codec or CursorCodec()
    offset = 0
    if request.cursor:
        state = codec.decode(request.cursor)
        raw_offset = state.get("offset")
        if not isinstance(raw_offset, int) or raw_offset < 0:
            raise DataValidationError("page cursor offset is invalid")
        offset = raw_offset

    items = tuple(values[offset : offset + request.size])
    next_offset = offset + len(items)
    next_cursor = (
        codec.encode({"offset": next_offset}) if next_offset < len(values) else None
    )
    return Page(items=items, next_cursor=next_cursor, total=len(values))


def chunked(values: Iterable[T], size: int) -> Iterator[tuple[T, ...]]:
    """Yield bounded immutable chunks without materializing the full source."""
    if size < 1:
        raise ValueError("chunk size must be positive")

    batch: list[T] = []
    for value in values:
        batch.append(value)
        if len(batch) == size:
            yield tuple(batch)
            batch.clear()
    if batch:
        yield tuple(batch)
