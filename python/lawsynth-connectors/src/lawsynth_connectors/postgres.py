"""PostgreSQL server-cursor connector with secret-safe authentication."""

from __future__ import annotations

from collections.abc import Iterable
from typing import Any

from ._optional import dependency
from .base import BaseConnector, ConnectorCapabilities, ReadRequest, Record, ResourceInfo
from .errors import DataValidationError, QueryError
from .sql import require_read_only_select


class PostgresConnector(BaseConnector):
    capabilities = ConnectorCapabilities(
        read=True,
        predicates=True,
        projections=True,
        transactions=True,
    )

    def _connection_arguments(self) -> dict[str, Any]:
        arguments: dict[str, Any] = {
            "connect_timeout": max(1, int(self.config.timeout_seconds)),
        }
        password = self.credentials.get("postgres_password")
        if password:
            arguments["password"] = password.reveal()
        application_name = self.config.options.get("application_name", "lawsynth-connectors")
        arguments["application_name"] = str(application_name)
        return arguments

    def _read_records(self, request: ReadRequest) -> Iterable[Record]:
        psycopg = dependency("psycopg", extra="postgres", connector="postgres")
        dsn = str(self.config.options.get("dsn", request.resource))
        query = require_read_only_select(str(request.options.get("query", "")))
        params = request.options.get("params", ())
        if not isinstance(params, (tuple, list, dict)):
            raise DataValidationError("PostgreSQL params must be a sequence or mapping")
        try:
            with psycopg.connect(dsn, **self._connection_arguments()) as connection:
                connection.read_only = True
                with connection.cursor(name="lawsynth_connector") as cursor:
                    cursor.itersize = self.config.batch_size
                    cursor.execute(query, params)
                    names = [column.name for column in cursor.description or ()]
                    if len(names) != len(set(names)):
                        raise DataValidationError(
                            "PostgreSQL query returned duplicate columns"
                        )
                    skipped = 0
                    emitted = 0
                    for row in cursor:
                        if skipped < request.offset:
                            skipped += 1
                            continue
                        yield dict(zip(names, row, strict=True))
                        emitted += 1
                        if request.limit is not None and emitted >= request.limit:
                            return
        except DataValidationError:
            raise
        except Exception as exc:
            raise QueryError(
                "PostgreSQL query failed",
                connector=self.config.name,
                retryable=True,
                details={"exception_type": type(exc).__name__},
            ) from exc

    def _inspect(self, resource: str) -> ResourceInfo:
        psycopg = dependency("psycopg", extra="postgres", connector="postgres")
        dsn = str(self.config.options.get("dsn", resource))
        query = (
            "SELECT table_schema, table_name FROM information_schema.tables "
            "WHERE table_schema NOT IN ('pg_catalog', 'information_schema') "
            "ORDER BY table_schema, table_name"
        )
        with psycopg.connect(dsn, **self._connection_arguments()) as connection:
            connection.read_only = True
            tables = tuple(".".join(row) for row in connection.execute(query))
        return ResourceInfo(resource, True, kind="database", metadata={"tables": tables})
