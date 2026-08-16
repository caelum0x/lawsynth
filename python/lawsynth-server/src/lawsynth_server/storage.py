"""Atomic filesystem content-addressed storage. S3 is a deployment adapter."""

from __future__ import annotations

import os
import tempfile
from pathlib import Path

from .artifacts import Artifact, artifact_from_bytes
from .errors import NotFoundError, ValidationError


class FileObjectStore:
    def __init__(self, root: Path, *, max_bytes: int) -> None:
        self.root, self.max_bytes = Path(root), max_bytes

    def _path(self, sha256: str) -> Path:
        if len(sha256) != 64 or any(c not in "0123456789abcdef" for c in sha256):
            raise ValidationError("invalid SHA-256 key")
        return self.root / "objects" / "sha256" / sha256[:2] / sha256[2:4] / sha256

    def put(self, data: bytes, media_type: str = "application/octet-stream") -> Artifact:
        if len(data) > self.max_bytes:
            raise ValidationError("artifact exceeds configured maximum")
        artifact = artifact_from_bytes(data, media_type)
        target = self._path(artifact.sha256)
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.exists():
            fd, temporary_name = tempfile.mkstemp(prefix=f".{artifact.sha256}.", suffix=".tmp", dir=target.parent)
            temp = Path(temporary_name)
            try:
                with os.fdopen(fd, "wb") as stream:
                    stream.write(data)
                    stream.flush()
                    os.fsync(stream.fileno())
                os.replace(temp, target)
            finally:
                temp.unlink(missing_ok=True)
        return artifact

    def get(self, sha256: str) -> bytes:
        try:
            return self._path(sha256).read_bytes()
        except FileNotFoundError as exc:
            raise NotFoundError("artifact not found") from exc
