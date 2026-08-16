"""Bounded adapter for CSV dialects commonly emitted by scientific tools."""

from __future__ import annotations

import csv
import io
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True, slots=True)
class CsvOptions:
    delimiter: str | None = None
    decimal: str = "."
    thousands: str | None = None
    encoding: str = "utf-8-sig"
    max_rows: int = 1_000_000
    max_bytes: int = 64 * 1024 * 1024

    def __post_init__(self) -> None:
        if self.delimiter is not None and len(self.delimiter) != 1:
            raise ValueError("delimiter must be one character")
        if self.decimal not in {".", ","}:
            raise ValueError("decimal must be '.' or ','")
        if self.thousands is not None and len(self.thousands) != 1:
            raise ValueError("thousands separator must be one character")
        if self.max_rows < 1 or self.max_bytes < 1:
            raise ValueError("CSV limits must be positive")


def _number(value: str, options: CsvOptions) -> Any:
    normalized = value.strip()
    if not normalized:
        return None
    if options.thousands:
        normalized = normalized.replace(options.thousands, "")
    if options.decimal == ",":
        normalized = normalized.replace(",", ".")
    try:
        return int(normalized)
    except ValueError:
        try:
            return float(normalized)
        except ValueError:
            return value.strip()


def parse_csv(payload: bytes, options: CsvOptions = CsvOptions()) -> list[dict[str, Any]]:
    if len(payload) > options.max_bytes:
        raise ValueError("CSV payload exceeds max_bytes")
    try:
        text = payload.decode(options.encoding)
    except (LookupError, UnicodeDecodeError) as exc:
        raise ValueError("CSV payload encoding is invalid") from exc
    sample = text[:8192]
    dialect = csv.Sniffer().sniff(sample, delimiters=",;\t|") if options.delimiter is None else None
    delimiter = options.delimiter or dialect.delimiter
    reader = csv.DictReader(io.StringIO(text), delimiter=delimiter)
    if not reader.fieldnames or any(not name.strip() for name in reader.fieldnames):
        raise ValueError("CSV requires a non-empty header")
    names = [name.strip() for name in reader.fieldnames]
    if len(names) != len(set(names)):
        raise ValueError("CSV header names must be unique")
    rows: list[dict[str, Any]] = []
    for index, row in enumerate(reader):
        if index >= options.max_rows:
            raise ValueError("CSV payload exceeds max_rows")
        if None in row:
            raise ValueError(f"CSV row {index + 2} has too many fields")
        rows.append({name: _number(row[original] or "", options) for name, original in zip(names, reader.fieldnames, strict=True)})
    return rows


class CsvVariantAdapter:
    def invoke(self, request: dict[str, Any]) -> dict[str, Any]:
        raw = request.get("payload")
        if isinstance(raw, str):
            payload = raw.encode(str(request.get("encoding", "utf-8")))
        elif isinstance(raw, bytes):
            payload = raw
        else:
            raise TypeError("payload must be bytes or text")
        options = CsvOptions(**dict(request.get("options", {})))
        rows = parse_csv(payload, options)
        return {"records": rows, "row_count": len(rows), "columns": list(rows[0]) if rows else []}
