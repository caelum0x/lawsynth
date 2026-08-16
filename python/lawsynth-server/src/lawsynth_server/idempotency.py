"""Request-key replay protection with payload fingerprint matching."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass
from threading import RLock
from typing import Callable

from .errors import IdempotencyConflict, ValidationError


@dataclass(frozen=True, slots=True)
class StoredResponse:
    fingerprint: str
    status: int
    body: dict[str, object]


def fingerprint(payload: object) -> str:
    return hashlib.sha256(json.dumps(payload, sort_keys=True, separators=(",", ":"), allow_nan=False).encode()).hexdigest()


class IdempotencyStore:
    def __init__(self) -> None:
        self._records: dict[tuple[str, str], StoredResponse] = {}
        self._lock = RLock()

    def execute(self, organization_id: str, key: str, payload: object, handler: Callable[[], tuple[int, dict[str, object]]]) -> tuple[int, dict[str, object], bool]:
        if not key or len(key) > 200:
            raise ValidationError("Idempotency-Key must be between 1 and 200 characters")
        digest = fingerprint(payload)
        identifier = (organization_id, key)
        with self._lock:
            existing = self._records.get(identifier)
            if existing:
                if existing.fingerprint != digest:
                    raise IdempotencyConflict("key was already used with a different request")
                return existing.status, dict(existing.body), True
            status, body = handler()
            self._records[identifier] = StoredResponse(digest, status, dict(body))
            return status, body, False
