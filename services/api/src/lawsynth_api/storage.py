"""Typed accessor over the domain content-addressed object store.

The API process never re-implements storage; it borrows the domain
``FileObjectStore`` and exposes a narrow, typed surface (get/put by sha256, root
readiness) for the transport's read paths and readiness checks.  This keeps the
storage backend a single deployment decision made in ``lawsynth_server``.
"""

from __future__ import annotations

from pathlib import Path

from lawsynth_server.artifacts import Artifact
from lawsynth_server.storage import FileObjectStore


class ApiStorage:
    """A read/write facade bound to one domain object store."""

    def __init__(self, store: FileObjectStore) -> None:
        self._store = store

    @property
    def root(self) -> Path:
        return self._store.root

    def get(self, sha256: str) -> bytes:
        """Return the stored bytes for a content hash (raises if absent)."""

        return self._store.get(sha256)

    def put(self, data: bytes, media_type: str = "application/octet-stream") -> Artifact:
        """Store bytes and return the content-addressed artifact descriptor."""

        return self._store.put(data, media_type)

    def ensure_root(self) -> bool:
        """Ensure the object root exists; report whether it is usable."""

        try:
            self._store.root.mkdir(parents=True, exist_ok=True)
        except OSError:
            return False
        return True
