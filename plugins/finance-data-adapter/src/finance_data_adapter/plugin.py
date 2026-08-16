"""Normalize provider-specific market bars into a reproducible OHLCV schema."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from datetime import datetime, timezone
from typing import Any

FIELDS = ("timestamp", "open", "high", "low", "close", "volume")


def _timestamp(value: Any) -> str:
    if isinstance(value, (int, float)):
        return datetime.fromtimestamp(value, timezone.utc).isoformat()
    parsed = datetime.fromisoformat(str(value).replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        raise ValueError("market timestamp must include a timezone")
    return parsed.astimezone(timezone.utc).isoformat()


class FinanceDataAdapter:
    def __init__(self, *, max_rows: int = 1_000_000) -> None:
        self.max_rows = max_rows

    def invoke(self, request: Mapping[str, Any]) -> dict[str, Any]:
        rows = request.get("records", ())
        mapping = {field: field for field in FIELDS} | dict(request.get("mapping", {}))
        if not isinstance(rows, Sequence) or len(rows) > self.max_rows:
            raise ValueError("finance records are invalid or exceed max_rows")
        symbol = str(request.get("symbol", "")).upper().strip()
        if not symbol or len(symbol) > 32:
            raise ValueError("symbol must contain 1..32 characters")
        normalized: list[dict[str, Any]] = []
        for index, source in enumerate(rows):
            if not isinstance(source, Mapping):
                raise TypeError(f"market row {index} is not an object")
            bar = {field: source.get(mapping[field]) for field in FIELDS}
            bar["timestamp"] = _timestamp(bar["timestamp"])
            for field in FIELDS[1:]:
                bar[field] = None if bar[field] is None else float(bar[field])
            if any(bar[field] is None for field in ("open", "high", "low", "close")):
                raise ValueError(f"market row {index} is missing OHLC data")
            if not bar["low"] <= min(bar["open"], bar["close"]) <= max(bar["open"], bar["close"]) <= bar["high"]:
                raise ValueError(f"market row {index} violates OHLC bounds")
            normalized.append({"symbol": symbol, **bar})
        normalized.sort(key=lambda row: row["timestamp"])
        if any(a["timestamp"] == b["timestamp"] for a, b in zip(normalized, normalized[1:])):
            raise ValueError("market timestamps must be unique")
        return {"records": normalized, "row_count": len(normalized), "symbol": symbol}
