"""Locate and stage datasets from the local repository.

LawSynth datasets are generated deterministically from their checked-in
``benchmark.toml`` rather than fetched from a remote host, so this module never
performs network I/O. "Downloading" a dataset means resolving its case directory
inside the repository and, optionally, copying the declarative files into a
staging directory whose checksums match the registry.
"""

from __future__ import annotations

import shutil
from pathlib import Path

from manifest import DatasetEntry


class DatasetNotFound(LookupError):
    """Raised when a dataset id is absent from the registry."""


def find_entry(entries: list[DatasetEntry], dataset_id: str) -> DatasetEntry:
    for entry in entries:
        if entry.id == dataset_id:
            return entry
    known = ", ".join(sorted(entry.id for entry in entries))
    raise DatasetNotFound(f"unknown dataset {dataset_id!r}; known: {known}")


def resolve(entry: DatasetEntry, root: Path) -> Path:
    """Return the absolute case directory for a dataset entry."""
    case_dir = (root / entry.path).resolve()
    if not case_dir.is_dir():
        raise DatasetNotFound(f"case directory for {entry.id!r} is missing: {case_dir}")
    return case_dir


def stage(entry: DatasetEntry, root: Path, dest: Path) -> list[Path]:
    """Copy an entry's declarative files into ``dest`` and return the copies.

    The copy is a plain local file copy; the caller can verify the result with
    :mod:`verify`. No network access occurs.
    """
    case_dir = resolve(entry, root)
    dest.mkdir(parents=True, exist_ok=True)
    staged: list[Path] = []
    for digest in entry.files:
        source = case_dir / digest.path
        target = dest / digest.path
        shutil.copyfile(source, target)
        staged.append(target)
    return staged
