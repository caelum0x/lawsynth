"""Runnable example: normalize vendor market bars into canonical OHLCV.

    python plugins/finance-data-adapter/examples/basic.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from finance_data_adapter.plugin import FinanceDataAdapter  # noqa: E402


def main() -> None:
    # A vendor payload with abbreviated field names and out-of-order bars,
    # mixing epoch-second and ISO-8601 timestamps.
    vendor_rows = [
        {"t": "2024-01-02T00:01:00+00:00", "o": 101, "h": 104, "l": 100, "c": 103, "v": 1500},
        {"t": 1_704_153_600, "o": 100, "h": 102, "l": 99, "c": 101, "v": 1000},
    ]

    adapter = FinanceDataAdapter()
    result = adapter.invoke(
        {
            "symbol": "aapl",
            "mapping": {"timestamp": "t", "open": "o", "high": "h",
                        "low": "l", "close": "c", "volume": "v"},
            "records": vendor_rows,
        }
    )

    print("symbol   :", result["symbol"])
    print("row_count:", result["row_count"])
    for row in result["records"]:
        print(row)


if __name__ == "__main__":
    main()
