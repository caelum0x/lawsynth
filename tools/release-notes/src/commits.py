"""Parse Conventional Commits into a structured, typed form.

Reference: https://www.conventionalcommits.org/en/v1.0.0/

The parser is pure and offline. Reading history from ``git`` is optional and
isolated in :func:`read_git_log`; every other function operates on plain
strings so the tool is fully testable without a repository.
"""

from __future__ import annotations

import re
import subprocess
from dataclasses import dataclass

# type(scope)!: subject   — scope and the breaking "!" marker are optional.
_HEADER = re.compile(
    r"^(?P<type>[a-z]+)"
    r"(?:\((?P<scope>[^)]+)\))?"
    r"(?P<bang>!)?"
    r": (?P<subject>.+)$"
)

# A commit is separated from the next by this record separator when read from
# ``git log`` with a custom format (see read_git_log).
_RECORD_SEPARATOR = "\x1e"


@dataclass(frozen=True)
class Commit:
    """A parsed commit message."""

    type: str
    scope: str | None
    subject: str
    breaking: bool
    body: str = ""

    @property
    def conventional(self) -> bool:
        return self.type != "other"


def parse_commit(message: str) -> Commit:
    """Parse a single commit message.

    Non-conventional messages are preserved with ``type == "other"`` and the
    first line as the subject so nothing is silently dropped.
    """
    text = message.strip()
    if not text:
        raise ValueError("cannot parse an empty commit message")
    header, _, body = text.partition("\n")
    body = body.strip()

    match = _HEADER.match(header.strip())
    if match is None:
        return Commit(type="other", scope=None, subject=header.strip(), breaking=False, body=body)

    breaking = match.group("bang") == "!" or "BREAKING CHANGE" in body
    return Commit(
        type=match.group("type"),
        scope=match.group("scope"),
        subject=match.group("subject").strip(),
        breaking=breaking,
        body=body,
    )


def parse_messages(messages: list[str]) -> list[Commit]:
    """Parse many messages, skipping blank entries."""
    return [parse_commit(message) for message in messages if message.strip()]


def read_git_log(revision_range: str, *, repo: str | None = None) -> list[Commit]:
    """Read and parse commits from local ``git`` history.

    ``revision_range`` is any range ``git log`` accepts, e.g. ``v0.1.0..HEAD``.
    This is the only function that shells out; it uses the local repository and
    never touches the network.
    """
    fmt = f"%B{_RECORD_SEPARATOR}"
    result = subprocess.run(
        ["git", "log", f"--format={fmt}", revision_range],
        cwd=repo,
        text=True,
        capture_output=True,
        check=True,
    )
    raw = [block.strip() for block in result.stdout.split(_RECORD_SEPARATOR)]
    return parse_messages(raw)
