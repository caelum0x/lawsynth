"""Render a Markdown dataset card (datasheet) from a registry entry."""

from __future__ import annotations

from manifest import DatasetEntry


def render_card(entry: DatasetEntry) -> str:
    """Return a deterministic Markdown datasheet for one dataset entry."""
    lines = [
        f"# {entry.title}",
        "",
        f"- **ID:** `{entry.id}`",
        f"- **Version:** {entry.version}",
        f"- **Capability:** {entry.capability}",
        f"- **Path:** `{entry.path}`",
        "",
        "## Files",
        "",
        "| File | SHA-256 | Bytes |",
        "| --- | --- | --- |",
    ]
    for digest in sorted(entry.files, key=lambda item: item.path):
        lines.append(f"| `{digest.path}` | `{digest.sha256[:16]}…` | {digest.bytes} |")
    lines.append("")
    lines.append(
        "_Series data is generated deterministically from `benchmark.toml`; "
        "only declarative files are indexed._"
    )
    return "\n".join(lines) + "\n"
