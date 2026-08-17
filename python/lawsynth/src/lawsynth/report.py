"""Self-contained HTML report and inline SVG chart generation.

Everything here depends only on the standard library: a report is a single
portable HTML file with rendered equations and an inline SVG trajectory chart,
so a colleague can open it with no server and no external assets.
"""

from __future__ import annotations

from html import escape
from math import isfinite
from typing import Mapping, Sequence

# Deterministic, colour-blind-friendly series palette (Okabe-Ito ordering).
_SERIES_COLORS = (
    "#0072b2", "#d55e00", "#009e73", "#cc79a7",
    "#e69f00", "#56b4e9", "#f0e442", "#000000",
)

_THEME = {
    "light": {"bg": "#ffffff", "fg": "#172033", "muted": "#53627a", "grid": "#e2e8f0", "border": "#cbd5e1", "accent": "#155e75"},
    "dark": {"bg": "#111827", "fg": "#e5e7eb", "muted": "#a5b4cc", "grid": "#2b3648", "border": "#374151", "accent": "#67e8f9"},
}


def _theme(name: str) -> dict[str, str]:
    return _THEME.get(name, _THEME["light"])


def _finite_bounds(values: Sequence[float]) -> tuple[float, float]:
    finite = [float(v) for v in values if isfinite(v)]
    if not finite:
        return 0.0, 1.0
    low, high = min(finite), max(finite)
    if low == high:
        pad = 1.0 if low == 0.0 else abs(low) * 0.1
        return low - pad, high + pad
    return low, high


def svg_line_chart(
    time: Sequence[float],
    series: Mapping[str, Sequence[float]],
    *,
    width: int = 640,
    height: int = 320,
    title: str = "",
    theme: str = "light",
    x_label: str = "t",
) -> str:
    """Render one or more aligned numeric series as a standalone SVG line chart."""
    if not time or not series:
        return '<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"></svg>'
    colors = _theme(theme)
    pad_l, pad_r, pad_t, pad_b = 52, 16, 28 if title else 12, 34
    plot_w = max(width - pad_l - pad_r, 1)
    plot_h = max(height - pad_t - pad_b, 1)

    tmin, tmax = _finite_bounds(time)
    all_values = [v for column in series.values() for v in column]
    ymin, ymax = _finite_bounds(all_values)
    tspan = tmax - tmin or 1.0
    yspan = ymax - ymin or 1.0

    def px(t: float) -> float:
        return pad_l + (float(t) - tmin) / tspan * plot_w

    def py(v: float) -> float:
        return pad_t + (ymax - float(v)) / yspan * plot_h

    parts: list[str] = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" '
        f'viewBox="0 0 {width} {height}" font-family="system-ui,sans-serif" font-size="11">',
        f'<rect width="{width}" height="{height}" fill="{colors["bg"]}"/>',
    ]
    if title:
        parts.append(
            f'<text x="{pad_l}" y="16" fill="{colors["accent"]}" font-size="13" '
            f'font-weight="600">{escape(title)}</text>'
        )

    # Horizontal gridlines + y tick labels (5 divisions).
    for i in range(5):
        yv = ymin + yspan * i / 4
        y = py(yv)
        parts.append(
            f'<line x1="{pad_l:.1f}" y1="{y:.1f}" x2="{pad_l + plot_w:.1f}" y2="{y:.1f}" '
            f'stroke="{colors["grid"]}" stroke-width="1"/>'
        )
        parts.append(
            f'<text x="{pad_l - 6:.1f}" y="{y + 3:.1f}" text-anchor="end" '
            f'fill="{colors["muted"]}">{yv:.3g}</text>'
        )

    # Axes.
    parts.append(
        f'<line x1="{pad_l}" y1="{pad_t + plot_h:.1f}" x2="{pad_l + plot_w:.1f}" '
        f'y2="{pad_t + plot_h:.1f}" stroke="{colors["border"]}" stroke-width="1.5"/>'
    )
    parts.append(
        f'<line x1="{pad_l}" y1="{pad_t:.1f}" x2="{pad_l}" y2="{pad_t + plot_h:.1f}" '
        f'stroke="{colors["border"]}" stroke-width="1.5"/>'
    )
    # x tick labels at the ends.
    parts.append(
        f'<text x="{pad_l:.1f}" y="{height - 8:.1f}" fill="{colors["muted"]}">{tmin:.3g}</text>'
    )
    parts.append(
        f'<text x="{pad_l + plot_w:.1f}" y="{height - 8:.1f}" text-anchor="end" '
        f'fill="{colors["muted"]}">{tmax:.3g}</text>'
    )
    parts.append(
        f'<text x="{pad_l + plot_w / 2:.1f}" y="{height - 8:.1f}" text-anchor="middle" '
        f'fill="{colors["muted"]}">{escape(x_label)}</text>'
    )

    # Series polylines.
    legend: list[str] = []
    for index, (name, column) in enumerate(sorted(series.items())):
        color = _SERIES_COLORS[index % len(_SERIES_COLORS)]
        points = " ".join(
            f"{px(t):.2f},{py(v):.2f}"
            for t, v in zip(time, column)
            if isfinite(t) and isfinite(v)
        )
        if points:
            parts.append(
                f'<polyline points="{points}" fill="none" stroke="{color}" '
                f'stroke-width="2" stroke-linejoin="round"/>'
            )
        lx = pad_l + index * 96
        legend.append(
            f'<rect x="{lx}" y="{pad_t - 2}" width="10" height="10" fill="{color}"/>'
            f'<text x="{lx + 14}" y="{pad_t + 7}" fill="{colors["fg"]}">{escape(name)}</text>'
        )
    # Legend row placed just under the title when there is horizontal room.
    if len(series) > 1:
        parts.append(
            f'<g transform="translate(0,{-pad_t + 2})">' + "".join(legend) + "</g>"
        )
    parts.append("</svg>")
    return "".join(parts)


def equations_table_html(equations: Mapping[str, str], theme: str = "light") -> str:
    """Render discovered laws as an HTML table (target -> derivative expression)."""
    colors = _theme(theme)
    rows = "".join(
        f'<tr><td style="padding:4px 10px;font-weight:600;color:{colors["accent"]}">'
        f"d{escape(target)}/dt</td>"
        f'<td style="padding:4px 10px;font-family:ui-monospace,monospace">= {escape(expr)}</td></tr>'
        for target, expr in sorted(equations.items())
    )
    return (
        f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
        f"<tbody>{rows}</tbody></table>"
    )


def _kv_table(pairs: Sequence[tuple[str, str]], theme: str = "light") -> str:
    colors = _theme(theme)
    rows = "".join(
        f'<tr><td style="padding:3px 10px;color:{colors["muted"]}">{escape(k)}</td>'
        f'<td style="padding:3px 10px;color:{colors["fg"]}">{escape(v)}</td></tr>'
        for k, v in pairs
    )
    return f'<table style="border-collapse:collapse">{rows}</table>'


def build_report(
    *,
    title: str,
    summary: Sequence[tuple[str, str]],
    equations: Mapping[str, str],
    laws_readable: Sequence[str],
    fit: Mapping[str, Mapping[str, float]],
    trajectory_time: Sequence[float],
    trajectory_values: Mapping[str, Sequence[float]],
    dependencies: Mapping[str, Sequence[str]],
    assumptions: Sequence[str],
    theme: str = "light",
) -> str:
    """Assemble a single self-contained HTML document for a discovered world."""
    colors = _theme(theme)
    chart = svg_line_chart(
        trajectory_time, dict(trajectory_values),
        width=760, height=360, title="Simulated trajectory", theme=theme,
    )

    fit_rows = "".join(
        f'<tr><td style="padding:3px 10px;color:{colors["muted"]}">{escape(state)}</td>'
        f'<td style="padding:3px 10px">R² = {metrics.get("r_squared", float("nan")):.4f}</td>'
        f'<td style="padding:3px 10px">RMSE = {metrics.get("rmse", float("nan")):.4g}</td></tr>'
        for state, metrics in sorted(fit.items())
    )
    laws_list = "".join(
        f'<li style="margin:4px 0;font-family:ui-monospace,monospace">{escape(law)}</li>'
        for law in laws_readable
    )
    dep_rows = "".join(
        f'<tr><td style="padding:3px 10px;font-weight:600">{escape(node)}</td>'
        f'<td style="padding:3px 10px">{escape(", ".join(deps) or "—")}</td></tr>'
        for node, deps in sorted(dependencies.items())
    )
    assume_list = "".join(f"<li>{escape(item)}</li>" for item in assumptions)

    def section(heading: str, body: str) -> str:
        return (
            f'<section style="background:{colors["bg"]};border:1px solid {colors["border"]};'
            f'border-radius:10px;padding:16px 18px;margin:14px 0">'
            f'<h2 style="margin:0 0 10px;font-size:16px;color:{colors["accent"]}">{escape(heading)}</h2>'
            f"{body}</section>"
        )

    body = "".join([
        f'<h1 style="font-size:22px;margin:0 0 4px">{escape(title)}</h1>',
        f'<p style="color:{colors["muted"]};margin:0 0 8px">LawSynth executable world — '
        "interpretable, deterministic, local-first.</p>",
        section("Overview", _kv_table(summary, theme)),
        section("Discovered laws", equations_table_html(equations, theme)
                + f'<ul style="margin:10px 0 0;padding-left:18px">{laws_list}</ul>'),
        section("Fit quality (simulation vs. observations)",
                f'<table style="border-collapse:collapse">{fit_rows}</table>'),
        section("Simulated trajectory", chart),
        section("Dependency structure",
                f'<table style="border-collapse:collapse">{dep_rows}</table>'),
        section("Assumptions", f'<ul style="margin:0;padding-left:18px">{assume_list}</ul>'),
    ])
    return (
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">"
        f"<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{escape(title)}</title>"
        f'</head><body style="background:{colors["grid"]};color:{colors["fg"]};'
        'font:14px/1.5 system-ui,sans-serif;margin:0;padding:24px">'
        f'<main style="max-width:840px;margin:0 auto">{body}</main></body></html>'
    )
