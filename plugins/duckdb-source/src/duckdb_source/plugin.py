"""Read-only DuckDB source plugin with streamed result bounds."""

from __future__ import annotations

import re
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

_MUTATION = re.compile(r"\b(insert|update|delete|create|drop|alter|copy|attach|install|load)\b", re.I)


def _query(value: Any) -> str:
    query = str(value).strip().rstrip(";")
    if not re.match(r"^(select|with)\b", query, re.I) or ";" in query or _MUTATION.search(query):
        raise ValueError("only one read-only SELECT/WITH query is allowed")
    return query


class DuckDBSource:
    def __init__(self, *, max_rows: int = 1_000_000, batch_size: int = 10_000) -> None:
        if max_rows < 1 or batch_size < 1:
            raise ValueError("query limits must be positive")
        self.max_rows, self.batch_size = max_rows, batch_size

    def invoke(self, request: Mapping[str, Any]) -> dict[str, Any]:
        try:
            import duckdb
        except ImportError as exc:
            raise RuntimeError("duckdb-source requires the duckdb package") from exc
        database = Path(str(request.get("database", ""))).expanduser().resolve()
        if not database.is_file():
            raise FileNotFoundError(database)
        params = request.get("params", ())
        if not isinstance(params, (Sequence, Mapping)):
            raise TypeError("query params must be a sequence or mapping")
        connection = duckdb.connect(str(database), read_only=True)
        rows: list[dict[str, Any]] = []
        try:
            result = connection.execute(_query(request.get("query")), params)
            columns = [column[0] for column in result.description or ()]
            if len(columns) != len(set(columns)):
                raise ValueError("query columns must be unique")
            while batch := result.fetchmany(self.batch_size):
                rows.extend(dict(zip(columns, row, strict=True)) for row in batch)
                if len(rows) > self.max_rows:
                    raise ValueError("query result exceeds max_rows")
        finally:
            connection.close()
        return {"records": rows, "row_count": len(rows), "columns": columns}
