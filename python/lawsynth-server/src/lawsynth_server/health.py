from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class Health:
    status: str
    database: str
    storage: str


def check(database: object, storage: object) -> Health:
    try:
        database.connection.execute("SELECT 1")
        storage.root.mkdir(parents=True, exist_ok=True)
    except Exception:
        return Health("degraded", "unavailable", "unavailable")
    return Health("ok", "ok", "ok")
