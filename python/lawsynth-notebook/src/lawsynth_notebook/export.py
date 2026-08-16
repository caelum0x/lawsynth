"""Reproducible exports generated from rendered data, never executable code."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .display import RenderedArtifact
from .errors import ArtifactValidationError
from .serialization import canonical_json


def export_html(view: RenderedArtifact, path: str | Path) -> Path:
    target = Path(path)
    if target.suffix.lower() not in {".html", ".htm"}:
        raise ArtifactValidationError("HTML export path must end in .html or .htm")
    document = f"<!doctype html><html><head><meta charset=\"utf-8\"><title>{view.title}</title></head><body>{view.html}</body></html>"
    target.write_text(document, encoding="utf-8")
    return target


def export_json(view: RenderedArtifact, path: str | Path) -> Path:
    target = Path(path)
    target.write_text(canonical_json(view.data) + "\n", encoding="utf-8")
    return target


def reproducible_notebook_cell(view: RenderedArtifact) -> dict[str, Any]:
    """Return a standard nbformat code cell that recreates a static JSON view."""
    source = ["from lawsynth_notebook import render_json\n", f"view = render_json({view.title!r}, {canonical_json(view.data)})\n", "view\n"]
    return {"cell_type": "code", "execution_count": None, "metadata": {"lawsynth": {"reproducible": True}}, "outputs": [], "source": source}
