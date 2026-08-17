"""Typed accessor over the domain metadata database.

The API process owns no schema; it borrows the domain ``Database`` (a SQLite
transaction wrapper in local mode, a Postgres adapter in server mode) and
exposes a minimal liveness probe used by readiness reporting.  Connection
lifetime remains owned by :class:`lifespan.ApiLifespan`, which closes it once.
"""

from __future__ import annotations

from lawsynth_server.database import Database


class ApiDatabase:
    """A liveness facade bound to one domain database connection."""

    def __init__(self, database: Database) -> None:
        self._database = database

    def ping(self) -> bool:
        """Return ``True`` when the metadata connection answers a trivial query."""

        try:
            self._database.connection.execute("SELECT 1")
        except Exception:
            return False
        return True
