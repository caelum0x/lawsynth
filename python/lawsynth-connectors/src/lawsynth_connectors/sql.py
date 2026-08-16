"""Safe DB-API query helpers and a read-only SQLite connector."""

from __future__ import annotations

import re
import sqlite3
from collections.abc import Iterable, Mapping, Sequence
from pathlib import Path
from typing import Any

from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import DataValidationError, QueryError, ResourceNotFoundError

_COMMENT = re.compile(r"--[^\n]*|/\*.*?\*/", re.DOTALL)
_MUTATING = re.compile(
    r"\b(?:alter|attach|create|delete|detach|drop|insert|merge|replace|truncate|update|vacuum)\b",
    re.IGNORECASE,
)
_IDENTIFIER = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


def require_read_only_select(query: str) -> str:
    """Validate one read-only SELECT/WITH statement and return normalized SQL."""
    text = _COMMENT.sub(" ", query).strip().rstrip(";").strip()
    if not text:
        raise DataValidationError("SQL query cannot be empty")
    if ";" in text:
        raise DataValidationError("SQL connector accepts exactly one statement")
    if not re.match(r"^(?:select|with)\b", text, re.IGNORECASE):
        raise DataValidationError("SQL connector accepts SELECT or WITH queries")
    if _MUTATING.search(text):
        raise DataValidationError("SQL query contains a mutating operation")
    return text


def quote_identifier(identifier: str) -> str:
    if not _IDENTIFIER.fullmatch(identifier):
        raise DataValidationError(f"invalid SQL identifier: {identifier!r}")
    return f'"{identifier}"'


def build_select(
    table: str,
    *,
    columns: Sequence[str] = (),
    filters: Mapping[str, Any] = {},
    limit: int | None = None,
    offset: int = 0,
) -> tuple[str, Mapping[str, Any]]:
    """Build a parameterized equality-filtered SELECT for simple resources."""
    projection = ", ".join(map(quote_identifier, columns)) if columns else "*"
    query = f"SELECT {projection} FROM {quote_identifier(table)}"
    parameters: dict[str, Any] = {}
    if filters:
        predicates: list[str] = []
        for index, (name, value) in enumerate(sorted(filters.items())):
            parameter = f"p{index}"
            predicates.append(f"{quote_identifier(name)} = :{parameter}")
            parameters[parameter] = value
        query += " WHERE " + " AND ".join(predicates)
    if limit is not None:
        query += " LIMIT :_limit"
        parameters["_limit"] = limit
    if offset:
        query += " OFFSET :_offset"
        parameters["_offset"] = offset
    return query, parameters


class SQLiteConnector(BaseConnector):
    """Stream SQLite rows through URI read-only mode."""

    capabilities = ConnectorCapabilities(
        read=True,
        predicates=True,
        projections=True,
        transactions=True,
    )

    def _database(self, resource: str) -> Path:
        configured = self.config.options.get("database")
        path = Path(str(configured or resource)).expanduser().resolve()
        if not path.is_file():
            raise ResourceNotFoundError(
                f"SQLite database does not exist: {path}", connector=self.config.name
            )
        return path

    def _query(self, request: ReadRequest) -> tuple[str, Any]:
        supplied = request.options.get("query")
        if supplied is not None:
            query = require_read_only_select(str(supplied))
            parameters = request.options.get("params", {})
            if not isinstance(parameters, (tuple, list, dict)):
                raise DataValidationError("SQL params must be a sequence or mapping")
            return query, parameters
        return build_select(
            str(request.options.get("table", request.resource)),
            columns=request.columns,
            filters=request.filters,
            limit=request.limit,
            offset=request.offset,
        )

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        database = self._database(request.resource)
        query, parameters = self._query(request)
        try:
            with sqlite3.connect(f"file:{database}?mode=ro", uri=True) as connection:
                connection.row_factory = sqlite3.Row
                connection.execute("PRAGMA query_only = ON")
                cursor = connection.execute(query, parameters)
                names = [column[0] for column in cursor.description or ()]
                if len(names) != len(set(names)):
                    raise DataValidationError("SQL query returned duplicate column names")
                skipped = 0
                emitted = 0
                while rows := cursor.fetchmany(self.config.batch_size):
                    for row in rows:
                        if request.options.get("query") is not None and skipped < request.offset:
                            skipped += 1
                            continue
                        yield dict(row)
                        emitted += 1
                        if request.limit is not None and emitted >= request.limit:
                            return
        except (DataValidationError, ResourceNotFoundError):
            raise
        except sqlite3.Error as exc:
            raise QueryError(
                "SQLite query failed", connector=self.config.name
            ) from exc

    def _inspect(self, resource: str) -> ResourceInfo:
        database = self._database(resource)
        with sqlite3.connect(f"file:{database}?mode=ro", uri=True) as connection:
            tables = tuple(
                row[0]
                for row in connection.execute(
                    "SELECT name FROM sqlite_schema "
                    "WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' "
                    "ORDER BY name"
                )
            )
        return ResourceInfo(
            resource,
            True,
            kind="database",
            byte_count=database.stat().st_size,
            metadata={"tables": tables, "path": database.as_posix()},
        )
