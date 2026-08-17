"""Tests for the declarative custom pipeline stage."""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from custom_stage_python.plugin import CustomStage


def _rows() -> list[dict[str, object]]:
    return [
        {"time": 0, "x": 1.0, "scratch": "a"},
        {"time": 1, "x": None, "scratch": "b"},
        {"time": 2, "x": 3.0, "scratch": "c"},
        {"time": 3, "x": 4.0, "scratch": "d"},
    ]


def test_select_keeps_only_named_columns_in_order() -> None:
    out = CustomStage().transform(_rows(), [{"kind": "select", "columns": ["x", "time"]}])
    assert list(out[0]) == ["x", "time"]


def test_rename_maps_column_names() -> None:
    out = CustomStage().transform(
        [{"a": 1, "b": 2}], [{"kind": "rename", "mapping": {"a": "alpha", "b": "beta"}}]
    )
    assert out == [{"alpha": 1, "beta": 2}]


def test_rename_rejects_colliding_destinations() -> None:
    with pytest.raises(ValueError, match="unique"):
        CustomStage().transform([{"a": 1, "b": 2}], [{"kind": "rename", "mapping": {"a": "z", "b": "z"}}])


def test_filter_predicates() -> None:
    stage = CustomStage()
    gte = stage.transform(_rows(), [{"kind": "filter", "field": "time", "operator": "gte", "value": 2}])
    assert [r["time"] for r in gte] == [2, 3]
    membership = stage.transform(_rows(), [{"kind": "filter", "field": "time", "operator": "in", "value": [0, 3]}])
    assert [r["time"] for r in membership] == [0, 3]


def test_drop_null_removes_rows_with_missing_field() -> None:
    out = CustomStage().transform(_rows(), [{"kind": "drop_null", "fields": ["x"]}])
    assert [r["time"] for r in out] == [0, 2, 3]


def test_fill_null_replaces_only_missing_values() -> None:
    out = CustomStage().transform(_rows(), [{"kind": "fill_null", "field": "x", "value": 0.0}])
    assert out[1]["x"] == 0.0
    assert out[0]["x"] == 1.0


def test_operations_compose_in_order_via_invoke() -> None:
    result = CustomStage().invoke(
        {
            "records": _rows(),
            "operations": [
                {"kind": "drop_null", "fields": ["x"]},
                {"kind": "select", "columns": ["time", "x"]},
                {"kind": "filter", "field": "x", "operator": "gt", "value": 1.0},
            ],
        }
    )
    assert result["input_rows"] == 4
    assert result["output_rows"] == 2
    assert result["records"] == [{"time": 2, "x": 3.0}, {"time": 3, "x": 4.0}]


def test_input_is_not_mutated() -> None:
    rows = _rows()
    CustomStage().transform(rows, [{"kind": "fill_null", "field": "x", "value": -1.0}])
    assert rows[1]["x"] is None  # original untouched


def test_unknown_operation_kind_is_rejected() -> None:
    with pytest.raises(ValueError, match="unsupported stage operation"):
        CustomStage().transform(_rows(), [{"kind": "explode"}])


def test_unknown_filter_operator_is_rejected() -> None:
    with pytest.raises(ValueError, match="operator"):
        CustomStage().transform(_rows(), [{"kind": "filter", "field": "x", "operator": "approx", "value": 1}])


def test_operation_limit_is_enforced() -> None:
    stage = CustomStage(max_operations=1)
    with pytest.raises(ValueError, match="max_operations"):
        stage.transform(_rows(), [{"kind": "select", "columns": ["time"]}] * 2)
