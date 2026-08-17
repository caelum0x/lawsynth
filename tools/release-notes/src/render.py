"""Render a change set to Keep a Changelog style Markdown."""

from __future__ import annotations

from changes import ChangeSet


def render_notes(changeset: ChangeSet, version: str, date: str) -> str:
    """Return a Markdown release-notes section for a version.

    ``version`` is a bare version string (no leading ``v``) and ``date`` is an
    ISO-8601 day. The heading follows the Keep a Changelog convention.
    """
    lines = [f"## {version} - {date}", ""]

    if changeset.is_empty:
        lines.append("_No user-facing changes were recorded for this release._")
        return "\n".join(lines) + "\n"

    if changeset.breaking:
        lines.append("### BREAKING CHANGES")
        lines.append("")
        lines.extend(f"- {entry}" for entry in changeset.breaking)
        lines.append("")

    for section in changeset.sections:
        lines.append(f"### {section.title}")
        lines.append("")
        lines.extend(f"- {entry}" for entry in section.entries)
        lines.append("")

    # Drop the trailing blank line, then terminate with a single newline.
    while lines and lines[-1] == "":
        lines.pop()
    return "\n".join(lines) + "\n"
