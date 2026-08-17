"""Structural and numeric record validation."""

from __future__ import annotations

from datetime import date, datetime, timezone
from decimal import Decimal

import pytest

from lawsynth_connectors.errors import DataValidationError
from lawsynth_connectors.validation import (
    FieldSpec,
    RecordSchema,
    validate_numeric_dataset,
    validate_records,
)


def test_field_spec_rejects_blank_name() -> None:
    with pytest.raises(ValueError):
        FieldSpec("  ")


def test_record_schema_rejects_duplicate_fields() -> None:
    with pytest.raises(ValueError):
        RecordSchema([FieldSpec("a"), FieldSpec("a")])


def test_validate_records_accepts_matching_types() -> None:
    schema = RecordSchema([FieldSpec("id", "integer", nullable=False), FieldSpec("name", "string")])
    report = validate_records([{"id": 1, "name": "a"}, {"id": 2, "name": "b"}], schema)
    assert report.valid
    assert report.row_count == 2


def test_validate_records_flags_missing_required_and_type_errors() -> None:
    schema = RecordSchema([FieldSpec("id", "integer", nullable=False)])
    report = validate_records([{"id": None}, {"id": "x"}], schema)
    assert not report.valid
    codes = {issue.code for issue in report.issues}
    assert codes == {"required", "type"}
    assert report.missing_by_field["id"] == 1


def test_validate_records_extra_field_when_not_allowed() -> None:
    schema = RecordSchema([FieldSpec("id", "integer")], allow_extra=False)
    report = validate_records([{"id": 1, "extra": 2}], schema)
    assert any(issue.code == "extra" for issue in report.issues)


def test_logical_type_matchers() -> None:
    schema = RecordSchema(
        [
            FieldSpec("b", "boolean"),
            FieldSpec("n", "number"),
            FieldSpec("d", "date"),
            FieldSpec("t", "datetime"),
        ]
    )
    good = {
        "b": True,
        "n": Decimal("1.5"),
        "d": date(2020, 1, 1),
        "t": datetime(2020, 1, 1, tzinfo=timezone.utc),
    }
    assert validate_records([good], schema).valid
    # a bool is not an integer/number; a datetime is not a bare date
    bad = {"b": 1, "n": float("nan"), "d": datetime(2020, 1, 1), "t": "no"}
    assert not validate_records([bad], schema).valid


def test_report_raise_for_errors() -> None:
    schema = RecordSchema([FieldSpec("id", "integer", nullable=False)])
    report = validate_records([{"id": None}], schema)
    with pytest.raises(DataValidationError):
        report.raise_for_errors(connector="x")
    validate_records([{"id": 1}], schema).raise_for_errors()  # valid: no raise


def test_validate_numeric_dataset_accepts_rectangular(numeric_records) -> None:
    rows = validate_numeric_dataset(numeric_records, time_column="time")
    assert len(rows) == 3


def test_validate_numeric_dataset_rejects_empty() -> None:
    with pytest.raises(DataValidationError):
        validate_numeric_dataset([])


def test_validate_numeric_dataset_requires_state_columns() -> None:
    with pytest.raises(DataValidationError):
        validate_numeric_dataset([{"time": 0}], time_column="time")


def test_validate_numeric_dataset_missing_time_column() -> None:
    with pytest.raises(DataValidationError):
        validate_numeric_dataset([{"x": 1.0}], time_column="time")


def test_validate_numeric_dataset_rejects_ragged_rows() -> None:
    with pytest.raises(DataValidationError):
        validate_numeric_dataset([{"x": 1.0}, {"x": 1.0, "y": 2.0}])


def test_validate_numeric_dataset_rejects_non_numeric_and_non_finite() -> None:
    with pytest.raises(DataValidationError):
        validate_numeric_dataset([{"x": "text"}])
    with pytest.raises(DataValidationError):
        validate_numeric_dataset([{"x": float("inf")}])
    with pytest.raises(DataValidationError):
        validate_numeric_dataset([{"x": True}])  # bool is not numeric here
