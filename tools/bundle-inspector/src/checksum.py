"""Parse and verify ``provenance/checksums.sha256``.

The checksum manifest is UTF-8 text with one line per payload entry (every
archive entry except the checksum file itself):

    <64 lowercase SHA-256 hex chars><two ASCII spaces><entry path>\\n

Lines appear in lexical entry order.  The reader requires a line for every
non-checksum entry and no extras, rejects malformed or duplicate paths, and
fails on any digest mismatch.  See ``specs/bundle/checksums.md``.
"""

from __future__ import annotations

import hashlib
from dataclasses import dataclass

from archive import CHECKSUMS_ENTRY, Archive, InvalidArchive


@dataclass(frozen=True)
class ChecksumLine:
    path: str
    expected: str
    actual: str

    @property
    def ok(self) -> bool:
        return self.expected == self.actual


@dataclass(frozen=True)
class ChecksumReport:
    lines: tuple[ChecksumLine, ...]

    @property
    def ok(self) -> bool:
        return all(line.ok for line in self.lines)

    @property
    def failures(self) -> tuple[ChecksumLine, ...]:
        return tuple(line for line in self.lines if not line.ok)


def parse_checksums(text: str) -> dict[str, str]:
    """Parse the checksum manifest into an ordered ``path -> digest`` mapping."""
    result: dict[str, str] = {}
    for number, raw in enumerate(text.splitlines(), start=1):
        if not raw:
            raise InvalidArchive(f"empty checksum line {number}")
        digest, separator, path = raw.partition("  ")
        if separator != "  " or not path:
            raise InvalidArchive(f"malformed checksum line {number}: {raw!r}")
        if len(digest) != 64 or not all(c in "0123456789abcdef" for c in digest):
            raise InvalidArchive(f"line {number} is not a lowercase SHA-256 digest")
        if path in result:
            raise InvalidArchive(f"duplicate checksum path: {path}")
        result[path] = digest
    return result


def verify_archive(archive: Archive) -> ChecksumReport:
    """Recompute SHA-256 over every payload entry and compare with the manifest."""
    declared = parse_checksums(archive.entry(CHECKSUMS_ENTRY).decode("utf-8"))
    payload = archive.payload_entries

    missing = [name for name in payload if name not in declared]
    if missing:
        raise InvalidArchive(f"checksum manifest missing entries: {', '.join(sorted(missing))}")
    extra = [name for name in declared if name not in payload]
    if extra:
        raise InvalidArchive(f"checksum manifest has extra entries: {', '.join(sorted(extra))}")

    lines = tuple(
        ChecksumLine(
            path=name,
            expected=declared[name],
            actual=hashlib.sha256(payload[name]).hexdigest(),
        )
        for name in sorted(payload)
    )
    return ChecksumReport(lines=lines)
