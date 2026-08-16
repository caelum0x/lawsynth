"""Read-only DuckDB query connector with bounded result streaming."""

from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path
from typing import Any

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import DataValidationError, QueryError, ResourceNotFoundError
from .sql import require_read_only_select


class DuckDBConnector(BaseConnector):
    capabilities = ConnectorCapabilities(
        read=True,
        predicates=True,
        projections=True,
        transactions=True,
    )

    def _database(self, resource: str) -> str:
        database = str(self.config.options.get("database", resource))
        if database != ":memory:" and not Path(database).expanduser().is_file():
            raise ResourceNotFoundError(
                f"DuckDB database does not exist: {database}",
                connector=self.config.name,
            )
        return database

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        duckdb = dependency("duckdb", extra="duckdb", connector="duckdb")
        query = require_read_only_select(str(request.options.get("query", "")))
        params = request.options.get("params", ())
        if not isinstance(params, (tuple, list, dict)):
            raise DataValidationError("DuckDB params must be a sequence or mapping")
        connection: Any = None
        try:
            connection = duckdb.connect(self._database(request.resource), read_only=True)
            result = connection.execute(query, params)
            names = [column[0] for column in result.description or ()]
            if len(names) != len(set(names)):
                raise DataValidationError("DuckDB query returned duplicate columns")
            skipped = 0
            emitted = 0
            while rows := result.fetchmany(self.config.batch_size):
                for row in rows:
                    if skipped < request.offset:
                        skipped += 1
                        continue
                    yield dict(zip(names, row, strict=True))
                    emitted += 1
                    if request.limit is not None and emitted >= request.limit:
                        return
        except (DataValidationError, ResourceNotFoundError):
            raise
        except Exception as exc:
            raise QueryError("DuckDB query failed", connector=self.config.name) from exc
        finally:
            if connection is not None:
                connection.close()

    def _inspect(self, resource: str) -> ResourceInfo:
        duckdb = dependency("duckdb", extra="duckdb", connector="duckdb")
        connection = duckdb.connect(self._database(resource), read_only=True)
        try:
            tables = tuple(row[0] for row in connection.execute("SHOW TABLES").fetchall())
        finally:
            connection.close()
        return ResourceInfo(resource, True, kind="database", metadata={"tables": tables})
