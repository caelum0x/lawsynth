"""Runnable example: normalize a European-style CSV into canonical records.

    python plugins/csv-variant-adapter/examples/basic.py
"""

from __future__ import annotations

import sys
from pathlib import Path

# Make the plugin importable without installation (src/ layout, PEP 420 package).
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from csv_variant_adapter.plugin import CsvVariantAdapter  # noqa: E402


def main() -> None:
    # A CSV emitted by a European locale tool: ';' delimiter, ',' decimal,
    # '.' thousands separator, and a UTF-8 BOM (handled by utf-8-sig).
    payload = "﻿time;pressure\n0;1.013,25\n60;1.008,10\n120;995,40\n"

    adapter = CsvVariantAdapter()
    result = adapter.invoke(
        {
            "payload": payload,
            "options": {"delimiter": ";", "decimal": ",", "thousands": "."},
        }
    )

    print("columns:", result["columns"])
    print("row_count:", result["row_count"])
    for row in result["records"]:
        print(row)

    # Shape the records the way lawsynth.dataset.Dataset.from_columns expects.
    records = result["records"]
    time = tuple(float(row["time"]) for row in records)
    columns = {"pressure": tuple(float(row["pressure"]) for row in records)}
    print("\ncanonical dataset arguments:")
    print("time    =", time)
    print("columns =", columns)


if __name__ == "__main__":
    main()
