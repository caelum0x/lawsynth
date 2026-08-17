"""Runnable example: reshape ingested records with a declarative stage.

    python plugins/custom-stage-python/examples/basic.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from custom_stage_python.plugin import CustomStage  # noqa: E402


def main() -> None:
    records = [
        {"t": 0, "temp_c": 20.0, "raw": "keep-out"},
        {"t": 1, "temp_c": None, "raw": "keep-out"},
        {"t": 2, "temp_c": 22.5, "raw": "keep-out"},
        {"t": 3, "temp_c": 30.0, "raw": "keep-out"},
    ]

    operations = [
        {"kind": "fill_null", "field": "temp_c", "value": 21.0},
        {"kind": "rename", "mapping": {"t": "time", "temp_c": "temperature"}},
        {"kind": "select", "columns": ["time", "temperature"]},
        {"kind": "filter", "field": "temperature", "operator": "lte", "value": 25.0},
    ]

    stage = CustomStage()
    result = stage.invoke({"records": records, "operations": operations})

    print("input_rows :", result["input_rows"])
    print("output_rows:", result["output_rows"])
    for row in result["records"]:
        print(row)


if __name__ == "__main__":
    main()
