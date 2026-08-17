"""Assemble the self-contained interactive World Explorer HTML bundle.

The bundle is a single themed ``<section>`` containing: the readable laws, an
inline-SVG trajectory chart (pre-rendered server-side so a real trajectory is
visible even before / without JavaScript), parameter sliders, initial-condition
inputs, an integration-method selector, and play/reset controls — followed by
one inline ``<script>`` that injects the world-as-JSON payload and the shared
:data:`~lawsynth_notebook.explorer_assets.INTEGRATOR_JS`. No external URLs, no
build step, no kernel: dragging a slider re-integrates and redraws in-browser.
"""

from __future__ import annotations

import hashlib
from collections.abc import Mapping
from html import escape
from typing import Any

from .explorer_assets import INTEGRATOR_JS
from .explorer_math import integrate
from .serialization import canonical_json
from .themes import palette

__all__ = ["build_explorer_html", "explorer_root_id"]

# Chart geometry — kept identical to the JS renderer so the pre-rendered and
# live charts line up exactly.
_W, _H, _ML, _MR, _MT, _MB = 760, 340, 52, 118, 16, 34
_SERIES_COLORS = ["#2563eb", "#dc2626", "#059669", "#d97706", "#7c3aed", "#0891b2", "#db2777", "#65a30d"]


def explorer_root_id(payload: Mapping[str, Any]) -> str:
    """A deterministic, collision-resistant DOM id derived from the payload."""
    digest = hashlib.sha256(canonical_json(dict(payload)).encode("utf-8")).hexdigest()
    return f"ls-explorer-{digest[:12]}"


def _fmt(value: float) -> str:
    if value != value or value in (float("inf"), float("-inf")):
        return "n/a"
    magnitude = abs(value)
    if magnitude != 0 and (magnitude < 0.001 or magnitude >= 100000):
        return f"{value:.2e}"
    return f"{round(value, 4):g}"


def _theme_colors(theme: str) -> dict[str, str]:
    colors = palette(theme)
    return {
        "bg": colors["background"],
        "fg": colors["foreground"],
        "muted": colors["muted"],
        "border": colors["border"],
        "accent": colors["accent"],
        "axis": colors["border"],
        "grid": colors["border"],
    }


def _readable_laws_html(payload: Mapping[str, Any], colors: Mapping[str, str]) -> str:
    grouped: dict[str, list[Mapping[str, Any]]] = {state: [] for state in payload["states"]}
    for param in payload["parameters"]:
        grouped[param["target"]].append(param)
    items = []
    for state in payload["states"]:
        terms = grouped[state]
        pieces = []
        for index, term in enumerate(terms):
            coeff = float(term["value"])
            sign = "-" if coeff < 0 else "+"
            magnitude = _fmt(abs(coeff))
            feature = term["label"]
            chunk = magnitude if feature == "1" else f"{magnitude}·{feature}"
            if index == 0:
                pieces.append(f"-{chunk}" if coeff < 0 else chunk)
            else:
                pieces.append(f"{sign} {chunk}")
        rhs = " ".join(pieces) if pieces else "0"
        items.append(
            f'<li style="margin:3px 0;font-family:ui-monospace,SFMono-Regular,monospace">'
            f'd{escape(state)}/dt = {escape(rhs)}</li>'
        )
    return f'<ul style="margin:0;padding-left:18px">{"".join(items)}</ul>'


def _svg_static(payload: Mapping[str, Any], colors: Mapping[str, str]) -> str:
    """Pre-render the initial trajectory so the chart is real without JS."""
    params = {param["id"]: float(param["value"]) for param in payload["parameters"]}
    time_cfg = payload["time"]
    try:
        traj = integrate(
            payload["states"], payload["laws"], params, payload["initial"],
            start=time_cfg["start"], end=time_cfg["end"], step=time_cfg["step"],
            method=payload.get("method", "rk4"), time_symbol=payload.get("timeSymbol", "t"),
        )
    except Exception:  # pragma: no cover - degrade to an empty frame
        return ""
    values = traj["values"]
    finite = [v for state in payload["states"] for v in values[state] if v == v and abs(v) != float("inf")]
    lo, hi = (min(finite), max(finite)) if finite else (0.0, 1.0)
    if hi - lo < 1e-9:
        lo, hi = lo - 1, hi + 1
    pad = (hi - lo) * 0.08
    lo, hi = lo - pad, hi + pad
    t0, t1 = traj["time"][0], traj["time"][-1]
    if t1 - t0 < 1e-9:
        t1 = t0 + 1

    def px(t: float) -> float:
        return _ML + (t - t0) / (t1 - t0) * (_W - _ML - _MR)

    def py(v: float) -> float:
        return _MT + (1 - (v - lo) / (hi - lo)) * (_H - _MT - _MB)

    parts = [
        f'<rect x="{_ML}" y="{_MT}" width="{_W - _ML - _MR}" height="{_H - _MT - _MB}" '
        f'fill="none" stroke="{colors["axis"]}" />'
    ]
    for g in range(5):
        yv = lo + (hi - lo) * g / 4
        yy = py(yv)
        parts.append(f'<line x1="{_ML}" y1="{yy:.1f}" x2="{_W - _MR}" y2="{yy:.1f}" stroke="{colors["grid"]}" />')
        parts.append(
            f'<text x="{_ML - 6}" y="{yy + 4:.1f}" text-anchor="end" font-size="11" '
            f'fill="{colors["muted"]}" font-family="system-ui">{escape(_fmt(yv))}</text>'
        )
    for si, state in enumerate(payload["states"]):
        color = _SERIES_COLORS[si % len(_SERIES_COLORS)]
        pts = " ".join(
            f"{px(traj['time'][i]):.1f},{py(v):.1f}"
            for i, v in enumerate(values[state]) if v == v and abs(v) != float("inf")
        )
        parts.append(f'<polyline points="{pts}" fill="none" stroke="{color}" stroke-width="2" />')
        ly = _MT + 8 + si * 18
        parts.append(f'<rect x="{_W - _MR + 10}" y="{ly - 8}" width="10" height="10" fill="{color}" />')
        last = values[state][-1]
        parts.append(
            f'<text x="{_W - _MR + 26}" y="{ly + 1}" font-size="12" fill="{colors["fg"]}" '
            f'font-family="system-ui">{escape(state)} = {escape(_fmt(last))}</text>'
        )
    return "".join(parts)


def _controls_html(payload: Mapping[str, Any], colors: Mapping[str, str]) -> str:
    slider_rows = []
    for param in payload["parameters"]:
        pid = escape(param["id"], quote=True)
        label = escape(param["label"])
        target = escape(param["target"])
        slider_rows.append(
            f'<div style="display:grid;grid-template-columns:150px 1fr 70px;align-items:center;gap:8px;margin:4px 0">'
            f'<label style="font-size:12px;color:{colors["muted"]}">'
            f'<span style="color:{colors["fg"]};font-family:ui-monospace,monospace">{label}</span> '
            f'<span>· d{target}/dt</span></label>'
            f'<input class="ls-param" type="range" data-id="{pid}" '
            f'min="{param["min"]}" max="{param["max"]}" step="{param["step"]}" value="{param["value"]}" '
            f'style="width:100%" />'
            f'<output data-val="{pid}" style="font-size:12px;font-family:ui-monospace,monospace;'
            f'color:{colors["fg"]};text-align:right">{escape(_fmt(float(param["value"])))}</output>'
            f'</div>'
        )
    init_rows = []
    for state in payload["states"]:
        value = float(payload["initial"][state])
        init_rows.append(
            f'<label style="font-size:12px;color:{colors["muted"]};margin-right:12px">'
            f'{escape(state)}(0) '
            f'<input class="ls-init" type="number" data-state="{escape(state, quote=True)}" '
            f'value="{value}" step="any" '
            f'style="width:80px;background:{colors["bg"]};color:{colors["fg"]};'
            f'border:1px solid {colors["border"]};border-radius:4px;padding:2px 4px" /></label>'
        )
    method = escape(payload.get("method", "rk4"), quote=True)
    method_select = (
        f'<label style="font-size:12px;color:{colors["muted"]}">integrator '
        f'<select class="ls-method" style="background:{colors["bg"]};color:{colors["fg"]};'
        f'border:1px solid {colors["border"]};border-radius:4px;padding:2px 4px">'
        f'<option value="rk4"{" selected" if method == "rk4" else ""}>RK4</option>'
        f'<option value="euler"{" selected" if method == "euler" else ""}>Euler</option>'
        f'</select></label>'
    )
    button_style = (
        f'background:{colors["accent"]};color:{colors["bg"]};border:none;border-radius:6px;'
        f'padding:5px 12px;font-size:13px;cursor:pointer;margin-right:8px'
    )
    controls = (
        f'<div style="margin-top:10px">'
        f'<div style="font-size:12px;color:{colors["muted"]};margin-bottom:4px">Term weights (drag to re-simulate)</div>'
        f'{"".join(slider_rows)}'
        f'<div style="margin-top:10px">'
        f'<div style="font-size:12px;color:{colors["muted"]};margin-bottom:4px">Initial conditions</div>'
        f'{"".join(init_rows)}</div>'
        f'<div style="margin-top:12px;display:flex;align-items:center;gap:12px;flex-wrap:wrap">'
        f'<button class="ls-play" style="{button_style}">▶ Play</button>'
        f'<button class="ls-reset" style="{button_style}">↺ Reset</button>'
        f'{method_select}</div></div>'
    )
    return controls


def _script(payload: Mapping[str, Any], root_id: str) -> str:
    payload_json = canonical_json(dict(payload)).replace("</", "<\\/")
    root_json = canonical_json(root_id)
    return (
        "<script>(function(){"
        f"var PAYLOAD={payload_json};"
        f"var ROOT_ID={root_json};"
        f"{INTEGRATOR_JS}"
        "})();</script>"
    )


def build_explorer_html(payload: Mapping[str, Any], theme: str = "light") -> str:
    """Return the complete self-contained interactive explorer HTML fragment."""
    colors = _theme_colors(theme)
    enriched = {**dict(payload), **colors}
    root_id = explorer_root_id(payload)
    name = escape(str(payload.get("name", "world")))

    header = (
        f'<header style="margin:0 0 8px">'
        f'<h3 style="margin:0;font-size:18px;color:{colors["accent"]}">World Explorer — {name}</h3>'
        f'<p style="margin:2px 0 0;color:{colors["muted"]};font-size:12px">'
        "Drag a term weight or edit an initial condition to re-simulate live — "
        "fully offline, no kernel round-trip.</p></header>"
    )
    laws_panel = (
        f'<div style="margin:8px 0"><div style="font-size:12px;color:{colors["muted"]};margin-bottom:4px">'
        f'Discovered laws</div>{_readable_laws_html(payload, colors)}</div>'
    )
    svg = (
        f'<svg class="ls-chart" viewBox="0 0 {_W} {_H}" '
        f'style="width:100%;height:auto;display:block;margin:6px 0">'
        f'{_svg_static(payload, colors)}</svg>'
    )
    controls = _controls_html(payload, colors)
    body = f"{header}{laws_panel}{svg}{controls}"
    fragment = (
        f'<section id="{root_id}" class="lawsynth-explorer" '
        f'style="background:{colors["bg"]};color:{colors["fg"]};border:1px solid {colors["border"]};'
        f'border-radius:12px;padding:16px 18px;margin:8px 0;font:14px/1.5 system-ui,sans-serif;max-width:820px">'
        f"{body}</section>"
    )
    return fragment + _script(enriched, root_id)
