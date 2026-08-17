"""Render deterministic, dependency-free inline SVG charts for the site."""

from __future__ import annotations

from compare import BOUNDARY, FAIL, PASS, PENDING, REGRESSION, Summary

# Deterministic, colour-blind-safe status palette.
_COLORS = {
    PASS: "#2a9d8f",
    FAIL: "#e76f51",
    REGRESSION: "#e63946",
    PENDING: "#adb5bd",
    BOUNDARY: "#457b9d",
}
_ORDER = (PASS, REGRESSION, FAIL, PENDING, BOUNDARY)

_BAR_HEIGHT = 24
_ROW_GAP = 8
_LABEL_WIDTH = 130
_TRACK_WIDTH = 320


def status_bar_chart(summary: Summary) -> str:
    """Return a horizontal bar chart (SVG) of result counts per status."""
    rows = [status for status in _ORDER if summary.counts.get(status)]
    maximum = max((summary.counts[status] for status in rows), default=1)
    height = max(len(rows) * (_BAR_HEIGHT + _ROW_GAP), _BAR_HEIGHT)
    width = _LABEL_WIDTH + _TRACK_WIDTH + 60

    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'role="img" aria-label="benchmark status counts" font-family="sans-serif" '
        f'font-size="13">'
    ]
    for index, status in enumerate(rows):
        count = summary.counts[status]
        y = index * (_BAR_HEIGHT + _ROW_GAP)
        bar_width = int(_TRACK_WIDTH * count / maximum) if maximum else 0
        color = _COLORS[status]
        text_y = y + _BAR_HEIGHT - 7
        parts.append(f'<text x="0" y="{text_y}" fill="#212529">{status}</text>')
        parts.append(
            f'<rect x="{_LABEL_WIDTH}" y="{y}" width="{bar_width}" '
            f'height="{_BAR_HEIGHT}" fill="{color}" rx="3" />'
        )
        parts.append(
            f'<text x="{_LABEL_WIDTH + bar_width + 6}" y="{text_y}" '
            f'fill="#212529">{count}</text>'
        )
    parts.append("</svg>")
    return "".join(parts)
