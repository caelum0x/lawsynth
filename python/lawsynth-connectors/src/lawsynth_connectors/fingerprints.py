"""Stable source and record fingerprints for reproducible ingestion."""

from __future__ import annotations

import hashlib
import json
import math
from collections.abc import Iterable, Mapping
from dataclasses import asdict, dataclass, is_dataclass
from datetime import date, datetime, timezone
from decimal import Decimal
from pathlib import Path
from typing import Any


@dataclass(frozen=True, slots=True)
class DatasetFingerprint:
    algorithm: str
    digest: str
    row_count: int | None = None
    byte_count: int | None = None

    def __post_init__(self) -> None:
        if self.algorithm != "sha256":
            raise ValueError("only sha256 dataset fingerprints are supported")
        if len(self.digest) != 64 or any(c not in "0123456789abcdef" for c in self.digest):
            raise ValueError("fingerprint digest is not a lowercase SHA-256 value")


def canonical_value(value: Any) -> Any:
    """Convert common scientific values into deterministic JSON data."""
    if value is None or isinstance(value, (str, bool, int)):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            return {"$float": str(value)}
        return value
    if isinstance(value, Decimal):
        return {"$decimal": str(value)}
    if isinstance(value, datetime):
        normalized = value.astimezone(timezone.utc) if value.tzinfo else value
        return {"$datetime": normalized.isoformat()}
    if isinstance(value, date):
        return {"$date": value.isoformat()}
    if isinstance(value, bytes):
        return {"$bytes": value.hex()}
    if isinstance(value, Path):
        return {"$path": value.as_posix()}
    if is_dataclass(value) and not isinstance(value, type):
        return canonical_value(asdict(value))
    if isinstance(value, Mapping):
        return {
            str(key): canonical_value(item)
            for key, item in sorted(value.items(), key=lambda pair: str(pair[0]))
        }
    if isinstance(value, (list, tuple)):
        return [canonical_value(item) for item in value]
    if isinstance(value, (set, frozenset)):
        canonical = [canonical_value(item) for item in value]
        return sorted(canonical, key=lambda item: json.dumps(item, sort_keys=True))
    if hasattr(value, "item") and callable(value.item):
        return canonical_value(value.item())
    return {"$repr": repr(value), "$type": type(value).__qualname__}


def fingerprint_records(records: Iterable[Mapping[str, Any]]) -> DatasetFingerprint:
    digest = hashlib.sha256()
    row_count = 0
    byte_count = 0

    for record in records:
        encoded = json.dumps(
            canonical_value(record),
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        ).encode("utf-8")
        digest.update(len(encoded).to_bytes(8, "big"))
        digest.update(encoded)
        row_count += 1
        byte_count += len(encoded)

    return DatasetFingerprint(
        algorithm="sha256",
        digest=digest.hexdigest(),
        row_count=row_count,
        byte_count=byte_count,
    )


def fingerprint_file(path: str | Path, *, chunk_size: int = 1024 * 1024) -> DatasetFingerprint:
    if chunk_size < 1:
        raise ValueError("fingerprint chunk size must be positive")

    source = Path(path)
    digest = hashlib.sha256()
    byte_count = 0
    with source.open("rb") as stream:
        while chunk := stream.read(chunk_size):
            digest.update(chunk)
            byte_count += len(chunk)
    return DatasetFingerprint("sha256", digest.hexdigest(), byte_count=byte_count)
