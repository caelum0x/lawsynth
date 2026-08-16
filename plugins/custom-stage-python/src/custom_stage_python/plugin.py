"""Declarative row transformation stage with no dynamic code evaluation."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any


def _predicate(value: Any, operator: str, expected: Any) -> bool:
    if operator == "eq": return value == expected
    if operator == "ne": return value != expected
    if operator == "gt": return value is not None and value > expected
    if operator == "gte": return value is not None and value >= expected
    if operator == "lt": return value is not None and value < expected
    if operator == "lte": return value is not None and value <= expected
    if operator == "in": return value in expected
    raise ValueError(f"unsupported filter operator: {operator}")


class CustomStage:
    """Apply a bounded sequence of select, rename, filter, and fill operations."""

    def __init__(self, *, max_rows: int = 1_000_000, max_operations: int = 100) -> None:
        if max_rows < 1 or max_operations < 1:
            raise ValueError("stage limits must be positive")
        self.max_rows = max_rows
        self.max_operations = max_operations

    def transform(self, records: Sequence[Mapping[str, Any]], operations: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
        if len(records) > self.max_rows:
            raise ValueError("stage input exceeds max_rows")
        if len(operations) > self.max_operations:
            raise ValueError("stage exceeds max_operations")
        rows = [dict(record) for record in records]
        for operation in operations:
            kind = operation.get("kind")
            if kind == "select":
                columns = tuple(map(str, operation.get("columns", ())))
                rows = [{column: row[column] for column in columns} for row in rows]
            elif kind == "rename":
                mapping = dict(operation.get("mapping", {}))
                if len(set(mapping.values())) != len(mapping):
                    raise ValueError("rename destinations must be unique")
                rows = [{str(mapping.get(key, key)): value for key, value in row.items()} for row in rows]
            elif kind == "filter":
                field = str(operation["field"])
                operator = str(operation.get("operator", "eq"))
                expected = operation.get("value")
                rows = [row for row in rows if _predicate(row.get(field), operator, expected)]
            elif kind == "fill_null":
                field, value = str(operation["field"]), operation.get("value")
                for row in rows:
                    if row.get(field) is None: row[field] = value
            elif kind == "drop_null":
                fields = tuple(map(str, operation.get("fields", ())))
                rows = [row for row in rows if all(row.get(field) is not None for field in fields)]
            else:
                raise ValueError(f"unsupported stage operation: {kind!r}")
        return rows

    def invoke(self, request: Mapping[str, Any]) -> dict[str, Any]:
        records, operations = request.get("records", ()), request.get("operations", ())
        if not isinstance(records, Sequence) or not isinstance(operations, Sequence):
            raise TypeError("records and operations must be sequences")
        result = self.transform(records, operations)
        return {"records": result, "input_rows": len(records), "output_rows": len(result)}
