"""Read the container of a ``.lsworld`` bundle.

A ``.lsworld`` file is a stored (uncompressed) ZIP archive with exactly three
logical entries in lexical order:

1. ``manifest.json``
2. ``provenance/checksums.sha256``
3. ``world/world.bin``

See ``specs/bundle/layout.md``.  This module reads the container with the
standard library :mod:`zipfile`, which validates each entry's CRC-32 on read,
and reports any structural deviation from the documented contract instead of
raising an opaque error.
"""

from __future__ import annotations

import zipfile
from dataclasses import dataclass, field
from pathlib import Path

MANIFEST_ENTRY = "manifest.json"
CHECKSUMS_ENTRY = "provenance/checksums.sha256"
WORLD_ENTRY = "world/world.bin"

REQUIRED_ENTRIES = (MANIFEST_ENTRY, CHECKSUMS_ENTRY, WORLD_ENTRY)


class InvalidArchive(ValueError):
    """Raised when a bundle violates the documented container contract."""


@dataclass(frozen=True)
class Archive:
    """The decoded, in-memory contents of a bundle container."""

    path: Path
    entries: dict[str, bytes]
    warnings: tuple[str, ...] = field(default_factory=tuple)

    def entry(self, name: str) -> bytes:
        try:
            return self.entries[name]
        except KeyError as error:
            raise InvalidArchive(f"missing entry: {name}") from error

    @property
    def payload_entries(self) -> dict[str, bytes]:
        """Every entry except the checksum manifest, in lexical order."""
        return {
            name: data
            for name, data in sorted(self.entries.items())
            if name != CHECKSUMS_ENTRY
        }


def _validate_entry_path(name: str) -> None:
    if not name or name != name.strip():
        raise InvalidArchive(f"illegal entry path: {name!r}")
    if "\\" in name:
        raise InvalidArchive(f"backslash in entry path: {name!r}")
    parts = name.split("/")
    if any(part in ("", ".", "..") for part in parts):
        raise InvalidArchive(f"empty or relative component in entry path: {name!r}")


def read_archive(path: str | Path) -> Archive:
    """Read a ``.lsworld`` container and return its validated entries.

    Raises :class:`InvalidArchive` for the failure modes the reader must reject:
    missing required entries, duplicate or path-violating entries, and CRC
    mismatches (surfaced by :mod:`zipfile`).  Compression and archive comments
    are recorded as warnings because they violate the strict writer contract.
    """
    path = Path(path)
    if not path.is_file():
        raise InvalidArchive(f"not a file: {path}")

    warnings: list[str] = []
    entries: dict[str, bytes] = {}
    try:
        with zipfile.ZipFile(path, "r") as archive:
            if archive.comment:
                warnings.append("archive comment present (contract forbids comments)")
            for info in archive.infolist():
                name = info.filename
                _validate_entry_path(name)
                if name in entries:
                    raise InvalidArchive(f"duplicate entry: {name}")
                if info.compress_type != zipfile.ZIP_STORED:
                    warnings.append(f"entry {name} is compressed (contract requires stored)")
                # ZipFile.read validates the CRC-32 for the entry.
                entries[name] = archive.read(name)
    except zipfile.BadZipFile as error:
        raise InvalidArchive(f"not a valid ZIP container: {error}") from error

    missing = [name for name in REQUIRED_ENTRIES if name not in entries]
    if missing:
        raise InvalidArchive(f"missing required entries: {', '.join(missing)}")

    extras = [name for name in entries if name not in REQUIRED_ENTRIES]
    if extras:
        warnings.append(f"unexpected entries: {', '.join(sorted(extras))}")

    return Archive(path=path, entries=entries, warnings=tuple(warnings))
