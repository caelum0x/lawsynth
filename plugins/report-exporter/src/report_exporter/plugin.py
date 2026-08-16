"""Deterministic Markdown and standalone HTML report rendering."""

from __future__ import annotations

import html
import json
from collections.abc import Mapping, Sequence
from typing import Any


def _markdown(report: Mapping[str, Any]) -> str:
    title = str(report.get("title", "LawSynth report")).strip()
    sections = report.get("sections", ())
    if not isinstance(sections, Sequence):
        raise TypeError("report sections must be a sequence")
    lines = [f"# {title}", ""]
    for section in sections:
        heading, body = str(section.get("title", "Section")), str(section.get("body", ""))
        lines.extend((f"## {heading}", "", body, ""))
    provenance = report.get("provenance")
    if provenance is not None:
        lines.extend(("## Provenance", "", "```json", json.dumps(provenance, sort_keys=True, indent=2, default=str), "```", ""))
    return "\n".join(lines)


class ReportExporter:
    def __init__(self, *, max_output_bytes: int = 16 * 1024 * 1024) -> None:
        self.max_output_bytes = max_output_bytes

    def invoke(self, request: Mapping[str, Any]) -> dict[str, Any]:
        report = request.get("report")
        if not isinstance(report, Mapping):
            raise TypeError("report must be an object")
        format_name = str(request.get("format", "markdown"))
        markdown = _markdown(report)
        if format_name == "markdown":
            content, media_type = markdown, "text/markdown; charset=utf-8"
        elif format_name == "html":
            paragraphs = "".join(f"<p>{html.escape(line)}</p>" for line in markdown.splitlines() if line)
            content = f'<!doctype html><html lang="en"><head><meta charset="utf-8"><title>{html.escape(str(report.get("title", "LawSynth report")))}</title></head><body><main>{paragraphs}</main></body></html>'
            media_type = "text/html; charset=utf-8"
        elif format_name == "json":
            content, media_type = json.dumps(report, sort_keys=True, indent=2, default=str), "application/json"
        else:
            raise ValueError(f"unsupported report format: {format_name}")
        encoded = content.encode("utf-8")
        if len(encoded) > self.max_output_bytes:
            raise ValueError("report exceeds max_output_bytes")
        return {"content": content, "media_type": media_type, "bytes": len(encoded)}
