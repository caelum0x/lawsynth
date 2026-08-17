"""Tests for the CSV variant adapter against small in-memory payloads."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from csv_variant_adapter.plugin import CsvOptions, CsvVariantAdapter, parse_csv


def test_sniffs_comma_delimiter_and_types_numbers() -> None:
    payload = b"time,x,y\n0,1.25,2\n1,3.5,4\n"
    rows = parse_csv(payload)
    assert rows == [
        {"time": 0, "x": 1.25, "y": 2},
        {"time": 1, "x": 3.5, "y": 4},
    ]
    # int stays int, decimals become float.
    assert isinstance(rows[0]["time"], int)
    assert isinstance(rows[0]["x"], float)


def test_european_dialect_with_semicolon_comma_decimal_and_thousands() -> None:
    payload = "time;pressure\n0;1.013,25\n60;995,40\n".encode("utf-8")
    options = CsvOptions(delimiter=";", decimal=",", thousands=".")
    rows = parse_csv(payload, options)
    assert rows == [
        {"time": 0, "pressure": 1013.25},
        {"time": 60, "pressure": 995.40},
    ]


def test_utf8_bom_is_stripped_by_default_encoding() -> None:
    payload = "﻿time,value\n0,10\n".encode("utf-8")
    rows = parse_csv(payload)
    assert list(rows[0]) == ["time", "value"]


def test_blank_cell_becomes_none() -> None:
    rows = parse_csv(b"time,x\n0,\n1,2\n")
    assert rows[0]["x"] is None
    assert rows[1]["x"] == 2


def test_invoke_returns_envelope_with_columns_and_count() -> None:
    adapter = CsvVariantAdapter()
    result = adapter.invoke({"payload": "time;value\n0;1,5\n1;2,5\n",
                             "options": {"delimiter": ";", "decimal": ","}})
    assert result["row_count"] == 2
    assert result["columns"] == ["time", "value"]
    assert result["records"][1] == {"time": 1, "value": 2.5}


def test_records_convert_to_canonical_dataset_shape() -> None:
    adapter = CsvVariantAdapter()
    records = adapter.invoke({"payload": b"time,x\n0,1.0\n1,2.0\n2,3.0\n"})["records"]
    time = tuple(float(row["time"]) for row in records)
    columns = {"x": tuple(float(row["x"]) for row in records)}
    # Canonical Dataset requires strictly increasing time and aligned columns.
    assert all(b > a for a, b in zip(time, time[1:]))
    assert len(columns["x"]) == len(time)


def test_duplicate_header_names_are_rejected() -> None:
    with pytest.raises(ValueError, match="unique"):
        parse_csv(b"time,x,x\n0,1,2\n")


def test_empty_header_is_rejected() -> None:
    with pytest.raises(ValueError, match="header"):
        parse_csv(b"time,,y\n0,1,2\n")


def test_max_bytes_limit_is_enforced() -> None:
    options = CsvOptions(delimiter=",", max_bytes=8)
    with pytest.raises(ValueError, match="max_bytes"):
        parse_csv(b"time,x\n0,1\n1,2\n", options)


def test_invalid_options_are_rejected_before_parsing() -> None:
    with pytest.raises(ValueError, match="decimal"):
        CsvOptions(decimal="x")
    with pytest.raises(ValueError, match="delimiter"):
        CsvOptions(delimiter=";;")


def test_invoke_rejects_non_text_non_bytes_payload() -> None:
    with pytest.raises(TypeError):
        CsvVariantAdapter().invoke({"payload": 123})
