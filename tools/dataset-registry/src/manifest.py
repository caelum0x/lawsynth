"""Dataset registry model and indexing.

A dataset entry describes one benchmark case checked in under ``benchmarks/``.
Each case declares itself in a ``benchmark.toml`` (id, title, version, and a
capability contract). The registry records that metadata plus SHA-256 checksums
of the declarative files so the index can be verified later.

Per ``specs/reproducibility/data-hash.md`` the scientific series themselves are
generated on demand rather than stored, so the registry hashes the *declarative*
files (``benchmark.toml``, ``expected.json``, ``baseline.json``) that fully
determine each case.
"""

from __future__ import annotations

import hashlib
import tomllib
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

REGISTRY_VERSION = "0.1"

# Declarative files that define a benchmark case, in a fixed order.
_INDEXED_FILES = ("benchmark.toml", "expected.json", "baseline.json", "README.md")


@dataclass(frozen=True)
class FileDigest:
    path: str  # relative to the case directory
    sha256: str
    bytes: int


@dataclass(frozen=True)
class DatasetEntry:
    id: str
    title: str
    version: int
    capability: str
    path: str  # relative to the registry root
    files: tuple[FileDigest, ...]

    def to_dict(self) -> dict[str, Any]:
        data = asdict(self)
        data["files"] = [asdict(digest) for digest in self.files]
        return data

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "DatasetEntry":
        files = tuple(FileDigest(**digest) for digest in data.get("files", []))
        return cls(
            id=str(data["id"]),
            title=str(data["title"]),
            version=int(data["version"]),
            capability=str(data["capability"]),
            path=str(data["path"]),
            files=files,
        )


def _sha256_file(path: Path) -> FileDigest:
    raw = path.read_bytes()
    return FileDigest(path=path.name, sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))


def _identity(config: dict[str, Any], case_dir: Path, root: Path) -> str:
    """Derive a dataset id, tolerating both benchmark.toml schemas.

    Some cases declare an explicit ``id``; others declare ``family`` + ``name``.
    When neither is present the relative directory path is used as a stable id.
    """
    if "id" in config:
        return str(config["id"])
    family = config.get("family")
    name = config.get("name")
    if family and name:
        return f"{family}/{name}"
    return str(case_dir.relative_to(root)).replace("\\", "/")


def _capability(config: dict[str, Any]) -> str:
    table = config.get("capability")
    if isinstance(table, dict):
        return str(table.get("status", "unknown"))
    return str(config.get("status", "unknown"))


def index_case(case_dir: Path, root: Path) -> DatasetEntry:
    """Build a registry entry for a single benchmark case directory."""
    config = tomllib.loads((case_dir / "benchmark.toml").read_text(encoding="utf-8"))
    dataset_id = _identity(config, case_dir, root)
    files = tuple(
        _sha256_file(case_dir / name)
        for name in _INDEXED_FILES
        if (case_dir / name).is_file()
    )
    return DatasetEntry(
        id=dataset_id,
        title=str(config.get("title", config.get("name", dataset_id))),
        version=int(config.get("version", 1)),
        capability=_capability(config),
        path=str(case_dir.relative_to(root)).replace("\\", "/"),
        files=files,
    )


def index_tree(root: Path) -> list[DatasetEntry]:
    """Index every benchmark case (any directory holding a ``benchmark.toml``)."""
    entries = [
        index_case(config.parent, root)
        for config in sorted(root.rglob("benchmark.toml"))
    ]
    return sorted(entries, key=lambda entry: entry.id)


def registry_document(entries: list[DatasetEntry]) -> dict[str, Any]:
    return {
        "registry_version": REGISTRY_VERSION,
        "datasets": [entry.to_dict() for entry in sorted(entries, key=lambda e: e.id)],
    }


def load_registry(path: Path) -> list[DatasetEntry]:
    import json

    data = json.loads(path.read_text(encoding="utf-8"))
    return [DatasetEntry.from_dict(item) for item in data.get("datasets", [])]
