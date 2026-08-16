"""Safe, standalone HTML fragments."""

from __future__ import annotations

from html import escape
from collections.abc import Mapping, Sequence
from typing import Any

from .themes import palette


def text(value: Any) -> str:
    return escape(str(value), quote=True)


def panel(title: str, body: str, theme: str = "light") -> str:
    colors = palette(theme)
    return (f'<section class="lawsynth-notebook" style="background:{colors["background"]};color:{colors["foreground"]};'
            f'border:1px solid {colors["border"]};border-radius:8px;padding:12px;margin:8px 0;font:14px system-ui">'
            f'<h3 style="margin:0 0 8px;color:{colors["accent"]}">{text(title)}</h3>{body}</section>')


def table(headers: Sequence[str], rows: Sequence[Sequence[Any]]) -> str:
    head = "".join(f"<th>{text(header)}</th>" for header in headers)
    body = "".join("<tr>" + "".join(f"<td>{text(cell)}</td>" for cell in row) + "</tr>" for row in rows)
    return f'<table style="border-collapse:collapse;width:100%"><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table>'


def definition_list(values: Mapping[str, Any]) -> str:
    return "<dl>" + "".join(f"<dt>{text(key)}</dt><dd>{text(value)}</dd>" for key, value in values.items()) + "</dl>"
