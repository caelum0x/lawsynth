"""Opaque cursors, HMAC tamper protection, and bounded chunking."""

from __future__ import annotations

import pytest

from lawsynth_connectors.errors import DataValidationError
from lawsynth_connectors.pagination import (
    CursorCodec,
    Page,
    PageRequest,
    chunked,
    paginate_sequence,
)


def test_page_request_bounds() -> None:
    assert PageRequest().size == 100
    with pytest.raises(ValueError):
        PageRequest(size=0)
    with pytest.raises(ValueError):
        PageRequest(size=10_001)


def test_page_has_more_property() -> None:
    assert Page(items=(), next_cursor="c").has_more is True
    assert Page(items=(1,), next_cursor=None).has_more is False


def test_cursor_codec_roundtrip_unsigned() -> None:
    codec = CursorCodec()
    cursor = codec.encode({"offset": 42})
    assert codec.decode(cursor) == {"offset": 42}


def test_cursor_codec_hmac_detects_tampering() -> None:
    codec = CursorCodec(secret=b"0123456789abcdef")
    cursor = codec.encode({"offset": 1})
    assert codec.decode(cursor) == {"offset": 1}
    tampered = ("A" if cursor[0] != "A" else "B") + cursor[1:]
    with pytest.raises(DataValidationError):
        codec.decode(tampered)


def test_cursor_codec_rejects_short_secret() -> None:
    with pytest.raises(ValueError):
        CursorCodec(secret=b"tooshort")


def test_cursor_codec_rejects_malformed_cursor() -> None:
    with pytest.raises(DataValidationError):
        CursorCodec().decode("!!!not-base64!!!")


def test_paginate_sequence_walks_all_pages() -> None:
    values = list(range(10))
    request = PageRequest(size=4)
    seen: list[int] = []
    codec = CursorCodec()
    while True:
        page = paginate_sequence(values, request, codec=codec)
        seen.extend(page.items)
        assert page.total == 10
        if not page.has_more:
            break
        request = PageRequest(size=4, cursor=page.next_cursor)
    assert seen == values


def test_paginate_sequence_rejects_negative_offset_cursor() -> None:
    codec = CursorCodec()
    bad = codec.encode({"offset": -1})
    with pytest.raises(DataValidationError):
        paginate_sequence([1, 2, 3], PageRequest(size=2, cursor=bad), codec=codec)


def test_chunked_yields_bounded_immutable_tuples() -> None:
    chunks = list(chunked(iter(range(5)), 2))
    assert chunks == [(0, 1), (2, 3), (4,)]
    assert all(isinstance(chunk, tuple) for chunk in chunks)


def test_chunked_rejects_bad_size() -> None:
    with pytest.raises(ValueError):
        list(chunked([1], 0))
