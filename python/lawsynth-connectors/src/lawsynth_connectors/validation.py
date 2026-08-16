"""Structural record validation independent of dataframe libraries."""

from __future__ import annotations

import math
from collections import Counter
from collections.abc import Iterable, Mapping, Sequence
from dataclasses import dataclass
from datetime import date, datetime
from decimal import Decimal
from typing import Any, Literal

from .errors import DataValidationError

LogicalType = Literal[
    "any",
    "boolean",
    "integer",
    "number",
    "string",
    "date",
    "datetime",
]


@dataclass(frozen=True, slots=True)
class FieldSpec:
    name: str
    logical_type: LogicalType = "any"
    nullable: bool = True

    def __post_init__(self) -> None:
        if not self.name.strip():
            raise ValueError("field name cannot be empty")


@dataclass(frozen=True, slots=True)
class RecordSchema:
    fields: Sequence[FieldSpec]
    allow_extra: bool = True

    def __post_init__(self) -> None:
        names = [field.name for field in self.fields]
        if len(names) != len(set(names)):
            raise ValueError("record schema contains duplicate field names")


@dataclass(frozen=True, slots=True)
class ValidationIssue:
    row: int
    field: str
    code: str
    message: str


@dataclass(frozen=True, slots=True)
class ValidationReport:
    row_count: int
    issues: Sequence[ValidationIssue]
    missing_by_field: Mapping[str, int]

    @property
    def valid(self) -> bool:
        return not self.issues

    def raise_for_errors(self, *, connector: str | None = None) -> None:
        if self.valid:
            return
        first = self.issues[0]
        raise DataValidationError(
            f"record validation failed with {len(self.issues)} issue(s): {first.message}",
            connector=connector,
            details={"row": first.row, "field": first.field, "code": first.code},
        )


def _matches(value: Any, logical_type: LogicalType) -> bool:
    if logical_type == "any":
        return True
    if logical_type == "boolean":
        return isinstance(value, bool)
    if logical_type == "integer":
        return isinstance(value, int) and not isinstance(value, bool)
    if logical_type == "number":
        return (
            isinstance(value, (int, float, Decimal))
            and not isinstance(value, bool)
            and (not isinstance(value, float) or math.isfinite(value))
        )
    if logical_type == "string":
        return isinstance(value, str)
    if logical_type == "datetime":
        return isinstance(value, datetime)
    if logical_type == "date":
        return isinstance(value, date) and not isinstance(value, datetime)
    return False


def validate_records(
    records: Iterable[Mapping[str, Any]],
    schema: RecordSchema,
    *,
    max_issues: int = 100,
) -> ValidationReport:
    if max_issues < 1:
        raise ValueError("max_issues must be positive")

    issues: list[ValidationIssue] = []
    missing: Counter[str] = Counter()
    expected = {field.name for field in schema.fields}
    row_count = 0

    for row_index, record in enumerate(records):
        row_count += 1
        for field in schema.fields:
            value = record.get(field.name)
            if value is None:
                missing[field.name] += 1
                if not field.nullable and len(issues) < max_issues:
                    issues.append(
                        ValidationIssue(
                            row_index,
                            field.name,
                            "required",
                            f"field {field.name!r} is required",
                        )
                    )
            elif not _matches(value, field.logical_type) and len(issues) < max_issues:
                issues.append(
                    ValidationIssue(
                        row_index,
                        field.name,
                        "type",
                        f"field {field.name!r} expected {field.logical_type}, "
                        f"received {type(value).__name__}",
                    )
                )

        if not schema.allow_extra:
            for extra in sorted(set(record) - expected):
                if len(issues) < max_issues:
                    issues.append(
                        ValidationIssue(
                            row_index,
                            extra,
                            "extra",
                            f"unexpected field {extra!r}",
                        )
                    )

    return ValidationReport(
        row_count=row_count,
        issues=tuple(issues),
        missing_by_field=dict(missing),
    )


def validate_numeric_dataset(
    records: Iterable[Mapping[str, Any]],
    *,
    time_column: str | None = None,
    connector: str | None = None,
) -> tuple[dict[str, float | int | str], ...]:
    """Validate data accepted by the Python LawSynth discovery API.

    Discovery accepts a rectangular list of mappings: one optional identifying
    time column and one or more finite numeric state columns.  This adapter
    intentionally does no interpolation or coercion; an external source that
    cannot represent a number faithfully is rejected at the boundary.
    """
    materialized = [dict(row) for row in records]
    if not materialized:
        raise DataValidationError("dataset contains no records", connector=connector)
    columns = tuple(materialized[0])
    if not columns:
        raise DataValidationError("dataset has no columns", connector=connector)
    expected = set(columns)
    if time_column is not None and time_column not in expected:
        raise DataValidationError(
            f"time column {time_column!r} is absent", connector=connector
        )
    numeric_columns = [column for column in columns if column != time_column]
    if not numeric_columns:
        raise DataValidationError("dataset has no state columns", connector=connector)
    for index, record in enumerate(materialized):
        if set(record) != expected:
            raise DataValidationError(
                "dataset records must have a stable rectangular schema",
                connector=connector,
                details={"row": index},
            )
        for column in numeric_columns:
            value = record[column]
            if isinstance(value, bool) or not isinstance(value, (int, float, Decimal)):
                raise DataValidationError(
                    f"state column {column!r} must be numeric",
                    connector=connector,
                    details={"row": index, "field": column},
                )
            if not math.isfinite(float(value)):
                raise DataValidationError(
                    f"state column {column!r} contains a non-finite value",
                    connector=connector,
                    details={"row": index, "field": column},
                )
    return tuple(materialized)
