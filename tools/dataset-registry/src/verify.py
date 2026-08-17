"""Verify a registry against the files on disk."""

from __future__ import annotations

import hashlib
from dataclasses import dataclass
from pathlib import Path

from manifest import DatasetEntry


@dataclass(frozen=True)
class Problem:
    dataset: str
    file: str
    kind: str  # "missing" | "changed"


def verify_entry(entry: DatasetEntry, root: Path) -> list[Problem]:
    """Return the checksum/existence problems for one dataset entry."""
    problems: list[Problem] = []
    case_dir = root / entry.path
    for digest in entry.files:
        path = case_dir / digest.path
        if not path.is_file():
            problems.append(Problem(entry.id, digest.path, "missing"))
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != digest.sha256:
            problems.append(Problem(entry.id, digest.path, "changed"))
    return problems


def verify_registry(entries: list[DatasetEntry], root: Path) -> list[Problem]:
    """Verify every entry, returning a flat, ordered list of problems."""
    problems: list[Problem] = []
    for entry in entries:
        problems.extend(verify_entry(entry, root))
    return problems
