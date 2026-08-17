"""Deterministic canonicalization and dataset/file fingerprints."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import date, datetime, timezone
from decimal import Decimal
from pathlib import Path

import pytest

from lawsynth_connectors.fingerprints import (
    DatasetFingerprint,
    canonical_value,
    fingerprint_file,
    fingerprint_records,
)


def test_dataset_fingerprint_validates_digest() -> None:
    with pytest.raises(ValueError):
        DatasetFingerprint("md5", "0" * 64)
    with pytest.raises(ValueError):
        DatasetFingerprint("sha256", "not-hex")


def test_fingerprint_records_is_stable_and_order_sensitive() -> None:
    rows = [{"a": 1, "b": 2.0}, {"a": 3, "b": 4.0}]
    first = fingerprint_records(rows)
    assert first == fingerprint_records([dict(r) for r in rows])
    assert first.row_count == 2
    assert first.byte_count is not None and first.byte_count > 0
    reordered = fingerprint_records(list(reversed(rows)))
    assert reordered.digest != first.digest


def test_fingerprint_is_key_order_independent() -> None:
    a = fingerprint_records([{"a": 1, "b": 2}])
    b = fingerprint_records([{"b": 2, "a": 1}])
    assert a.digest == b.digest


def test_canonical_value_handles_scientific_types() -> None:
    assert canonical_value(float("nan")) == {"$float": "nan"}
    assert canonical_value(Decimal("1.5")) == {"$decimal": "1.5"}
    assert canonical_value(b"\x00\x01") == {"$bytes": "0001"}
    assert canonical_value(date(2020, 1, 1)) == {"$date": "2020-01-01"}
    assert canonical_value(Path("/tmp/x")) == {"$path": "/tmp/x"}
    dt = datetime(2020, 1, 1, 12, tzinfo=timezone.utc)
    assert canonical_value(dt)["$datetime"].startswith("2020-01-01T12:00:00")


def test_canonical_value_sorts_mappings_and_sets() -> None:
    assert canonical_value({"b": 1, "a": 2}) == {"a": 2, "b": 1}
    assert canonical_value({3, 1, 2}) == [1, 2, 3]


def test_canonical_value_dataclass() -> None:
    @dataclass
    class Point:
        x: int
        y: int

    assert canonical_value(Point(1, 2)) == {"x": 1, "y": 2}


def test_canonical_value_unknown_object_repr_fallback() -> None:
    class Opaque:
        pass

    result = canonical_value(Opaque())
    assert "$repr" in result and result["$type"].endswith("Opaque")


def test_fingerprint_file_matches_bytes(tmp_path: Path) -> None:
    import hashlib

    data = b"time,x\n0,1\n1,2\n"
    path = tmp_path / "d.csv"
    path.write_bytes(data)
    fingerprint = fingerprint_file(path)
    assert fingerprint.digest == hashlib.sha256(data).hexdigest()
    assert fingerprint.byte_count == len(data)


def test_fingerprint_file_rejects_bad_chunk_size(tmp_path: Path) -> None:
    path = tmp_path / "d.csv"
    path.write_bytes(b"x")
    with pytest.raises(ValueError):
        fingerprint_file(path, chunk_size=0)
