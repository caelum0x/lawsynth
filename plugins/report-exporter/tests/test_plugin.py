"""Tests for the report-exporter plugin.

Run with the package on the path::

    PYTHONPATH=src pytest
"""

from __future__ import annotations

import json

import pytest

from report_exporter.plugin import ReportExporter

REPORT = {
    "title": "Discovery Report",
    "sections": [
        {"title": "Summary", "body": "Recovered a damped oscillator."},
        {"title": "Fit", "body": "MSE = 3.1e-6."},
    ],
    "provenance": {"run_id": "r-1", "seed": 42},
}


def test_markdown_render_includes_title_and_sections() -> None:
    artifact = ReportExporter().invoke({"report": REPORT, "format": "markdown"})
    assert artifact["media_type"] == "text/markdown; charset=utf-8"
    content = artifact["content"]
    assert content.startswith("# Discovery Report")
    assert "## Summary" in content
    assert "## Provenance" in content
    assert artifact["bytes"] == len(content.encode("utf-8"))


def test_markdown_is_default_format() -> None:
    default = ReportExporter().invoke({"report": REPORT})
    explicit = ReportExporter().invoke({"report": REPORT, "format": "markdown"})
    assert default["content"] == explicit["content"]


def test_html_render_escapes_and_wraps() -> None:
    report = {"title": "A & B", "sections": [{"title": "S", "body": "x < y"}]}
    artifact = ReportExporter().invoke({"report": report, "format": "html"})
    assert artifact["media_type"] == "text/html; charset=utf-8"
    assert "<!doctype html>" in artifact["content"]
    assert "A &amp; B" in artifact["content"]
    assert "x &lt; y" in artifact["content"]


def test_json_render_round_trips() -> None:
    artifact = ReportExporter().invoke({"report": REPORT, "format": "json"})
    assert artifact["media_type"] == "application/json"
    assert json.loads(artifact["content"]) == REPORT


def test_unsupported_format_raises() -> None:
    with pytest.raises(ValueError):
        ReportExporter().invoke({"report": REPORT, "format": "pdf"})


def test_non_mapping_report_raises() -> None:
    with pytest.raises(TypeError):
        ReportExporter().invoke({"report": ["not", "a", "mapping"]})


def test_output_size_limit_enforced() -> None:
    exporter = ReportExporter(max_output_bytes=16)
    with pytest.raises(ValueError):
        exporter.invoke({"report": REPORT, "format": "json"})


def test_render_is_deterministic() -> None:
    first = ReportExporter().invoke({"report": REPORT, "format": "json"})
    second = ReportExporter().invoke({"report": REPORT, "format": "json"})
    assert first["content"] == second["content"]
