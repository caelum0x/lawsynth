"""Tests for the finance OHLCV normalization adapter."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from finance_data_adapter.plugin import FinanceDataAdapter, _timestamp


def test_normalizes_mapped_fields_and_sorts_by_timestamp() -> None:
    adapter = FinanceDataAdapter()
    result = adapter.invoke(
        {
            "symbol": "aapl",
            "mapping": {"timestamp": "t", "open": "o", "high": "h",
                        "low": "l", "close": "c", "volume": "v"},
            "records": [
                {"t": "2024-01-02T00:01:00+00:00", "o": 101, "h": 104, "l": 100, "c": 103, "v": 1500},
                {"t": 1_704_153_600, "o": 100, "h": 102, "l": 99, "c": 101, "v": 1000},
            ],
        }
    )
    assert result["symbol"] == "AAPL"
    assert result["row_count"] == 2
    first, second = result["records"]
    # epoch 1_704_153_600 == 2024-01-02T00:00:00Z sorts before the 00:01 bar.
    assert first["timestamp"] < second["timestamp"]
    assert first["open"] == 100.0 and isinstance(first["open"], float)
    assert first["symbol"] == "AAPL"


def test_epoch_and_iso_timestamps_normalize_to_utc_iso() -> None:
    assert _timestamp(1_704_153_600).startswith("2024-01-02T00:00:00")
    assert _timestamp("2024-01-02T00:00:00Z").startswith("2024-01-02T00:00:00")


def test_naive_timestamp_is_rejected() -> None:
    with pytest.raises(ValueError, match="timezone"):
        _timestamp("2024-01-02T00:00:00")


def test_ohlc_bounds_are_enforced() -> None:
    adapter = FinanceDataAdapter()
    with pytest.raises(ValueError, match="OHLC bounds"):
        adapter.invoke(
            {
                "symbol": "x",
                # high (5) below close (10) violates the ordering invariant.
                "records": [{"timestamp": 0, "open": 1, "high": 5, "low": 0, "close": 10, "volume": 1}],
            }
        )


def test_missing_ohlc_is_rejected() -> None:
    adapter = FinanceDataAdapter()
    with pytest.raises(ValueError, match="missing OHLC"):
        adapter.invoke(
            {"symbol": "x", "records": [{"timestamp": 0, "open": 1, "high": 2, "low": 0, "volume": 1}]}
        )


def test_duplicate_timestamps_are_rejected() -> None:
    adapter = FinanceDataAdapter()
    bar = {"open": 1, "high": 2, "low": 0, "close": 1.5, "volume": 1}
    with pytest.raises(ValueError, match="unique"):
        adapter.invoke({"symbol": "x", "records": [{"timestamp": 0, **bar}, {"timestamp": 0, **bar}]})


def test_symbol_validation() -> None:
    adapter = FinanceDataAdapter()
    with pytest.raises(ValueError, match="symbol"):
        adapter.invoke({"symbol": "", "records": []})


def test_normalized_close_series_forms_canonical_columns() -> None:
    adapter = FinanceDataAdapter()
    records = adapter.invoke(
        {
            "symbol": "spy",
            "records": [
                {"timestamp": 0, "open": 1, "high": 2, "low": 0.5, "close": 1.5, "volume": 10},
                {"timestamp": 60, "open": 1.5, "high": 2.5, "low": 1.0, "close": 2.0, "volume": 20},
            ],
        }
    )["records"]
    close = tuple(row["close"] for row in records)
    assert close == (1.5, 2.0)
    assert all(isinstance(v, float) for v in close)
