"""Insert rendered release notes into a Keep a Changelog document.

The functions are pure: they take the existing changelog text and return a new
string. Callers own writing to disk, keeping the transformation testable and
free of side effects.
"""

from __future__ import annotations

_UNRELEASED_HEADING = "## Unreleased"


def insert_release(changelog: str, notes: str) -> str:
    """Return ``changelog`` with ``notes`` inserted as the newest release.

    ``notes`` is placed immediately after the ``## Unreleased`` section when one
    exists, otherwise after the top-level ``# Changelog`` title. The original
    text is never mutated in place.
    """
    body = notes.rstrip("\n") + "\n"
    lines = changelog.splitlines()

    anchor = _find_unreleased_end(lines)
    if anchor is None:
        anchor = _find_title_end(lines)

    head = lines[:anchor]
    tail = lines[anchor:]
    rebuilt = "\n".join(head).rstrip("\n")
    merged = f"{rebuilt}\n\n{body}"
    if tail:
        merged += "\n" + "\n".join(tail).lstrip("\n")
    if not merged.endswith("\n"):
        merged += "\n"
    return merged


def _find_unreleased_end(lines: list[str]) -> int | None:
    """Return the index just before the next release heading after Unreleased."""
    start = None
    for index, line in enumerate(lines):
        if line.strip() == _UNRELEASED_HEADING:
            start = index
            break
    if start is None:
        return None
    for index in range(start + 1, len(lines)):
        if lines[index].startswith("## "):
            return index
    return len(lines)


def _find_title_end(lines: list[str]) -> int:
    for index, line in enumerate(lines):
        if line.startswith("# "):
            return index + 1
    return 0
