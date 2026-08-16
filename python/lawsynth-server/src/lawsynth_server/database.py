"""SQLite transaction wrapper used by local mode metadata adapters."""

from __future__ import annotations

import sqlite3
from contextlib import contextmanager
from typing import Iterator


class Database:
    def __init__(self, url: str = ":memory:") -> None:
        if url.startswith("sqlite:///"):
            url = url.removeprefix("sqlite:///")
        elif url != ":memory:":
            raise ValueError("only SQLite URLs are supported by the local server core; provide a Postgres adapter for server deployment")
        self.connection = sqlite3.connect(url, check_same_thread=False, isolation_level=None)
        self.connection.execute("PRAGMA foreign_keys = ON")

    @contextmanager
    def transaction(self) -> Iterator[sqlite3.Connection]:
        self.connection.execute("BEGIN IMMEDIATE")
        try:
            yield self.connection
        except BaseException:
            self.connection.execute("ROLLBACK")
            raise
        else:
            self.connection.execute("COMMIT")

    def close(self) -> None:
        self.connection.close()
