"""A cohesive, themed dashboard that composes the individual LawSynth views.

``StudyDashboard`` folds the equation, dependency, trajectory, uncertainty,
candidate-frontier and (optional) scenario views into a single ``_repr_html_``
so a data scientist gets the whole picture of a discovered world in one cell.

Everything is standard-library only: the individual view fragments come from
this package's existing renderers, and the trajectory chart reuses the SDK's
inline-SVG chart builder. No external JS or CSS, no network access — the same
deterministic, offline contract as the rest of LawSynth. Importing this module
does not load the native extension; the compiled engine is only touched when a
dashboard is actually built from an already-discovered world.
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from math import isfinite
from typing import Any, Mapping, Sequence

from .config import NotebookConfig
from .equation_view import render_equations
from .frontier_view import render_frontier
from .graph_view import render_graph
from .regime_view import render_regimes
from .templates import panel
from .themes import palette
from .uncertainty_view import render_uncertainty

__all__ = ["StudyDashboard", "render_dashboard", "build_dashboard_html"]


def _fit_table_html(fit: Mapping[str, Mapping[str, float]], theme: str) -> str:
    colors = palette(theme)
    rows = "".join(
        f'<tr><td style="padding:4px 10px;color:{colors["muted"]}">{escape(state)}</td>'
        f'<td style="padding:4px 10px;text-align:right">R² = {metrics.get("r_squared", float("nan")):.4f}</td>'
        f'<td style="padding:4px 10px;text-align:right">RMSE = {metrics.get("rmse", float("nan")):.4g}</td></tr>'
        for state, metrics in sorted(fit.items())
    )
    return f'<table style="border-collapse:collapse;width:100%;color:{colors["foreground"]}">{rows}</table>'


def _laws_list_html(laws_readable: Sequence[str], theme: str) -> str:
    items = "".join(
        f'<li style="margin:4px 0;font-family:ui-monospace,monospace">{escape(text)}</li>'
        for text in laws_readable
    )
    return f'<ul style="margin:0;padding-left:18px">{items}</ul>'


def _uncertainty_entries(
    fit: Mapping[str, Mapping[str, float]],
    finals: Mapping[str, float],
) -> list[dict[str, float | str]]:
    """±RMSE bands around each state's final simulated value.

    A real, honest uncertainty proxy: the residual RMSE from the fit is used as
    a symmetric band around the forward-simulated final value. States whose fit
    did not produce a finite RMSE are skipped.
    """
    entries: list[dict[str, float | str]] = []
    for state in sorted(fit):
        rmse = fit[state].get("rmse", float("nan"))
        mean = finals.get(state, float("nan"))
        if not (isfinite(rmse) and isfinite(mean)):
            continue
        entries.append({"name": state, "lower": mean - rmse, "upper": mean + rmse, "mean": mean})
    return entries


def _frontier_candidates(laws: Sequence[Any], fit: Mapping[str, Mapping[str, float]]) -> list[dict[str, Any]]:
    """The discovered laws as an error-vs-complexity candidate set.

    Each law becomes one candidate: ``score`` is its fit error (1 − R²) and
    ``complexity`` is its retained-term count, so the frontier view ranks laws
    by how much accuracy each spends on structural complexity.
    """
    candidates: list[dict[str, Any]] = []
    for law in laws:
        r_squared = fit.get(law.target, {}).get("r_squared", 0.0)
        error = 1.0 - r_squared if isfinite(r_squared) else 1.0
        candidates.append({
            "id": law.target,
            "score": error,
            "complexity": float(len(law.terms)),
            "equation": law.readable,
        })
    return candidates


def build_dashboard_html(
    source: Any,
    *,
    theme: str = "light",
    comparison: Any = None,
    regimes: Sequence[Mapping[str, Any]] | None = None,
    horizon: float | None = None,
) -> tuple[str, dict[str, Any]]:
    """Compose the dashboard fragment (and its structured data) for a world.

    ``source`` is anything exposing ``name``, ``states``, ``explain()`` and
    ``simulate()`` — both :class:`lawsynth.Study` (post-discovery) and
    :class:`lawsynth.DiscoveryResult` qualify.
    """
    config = NotebookConfig(theme=theme)
    colors = palette(config.theme)
    # Lazy SDK import: keeps this module free of a hard dependency at import
    # time and preserves the "importing lawsynth loads no native code" contract.
    from lawsynth.report import svg_line_chart

    explanation = source.explain()
    trajectory = source.simulate(horizon=horizon)
    states = tuple(explanation.variables)
    finals = {
        name: float(series[-1])
        for name, series in trajectory.values.items()
        if series
    }

    equations = {law.target: law.expression for law in explanation.laws}
    laws_readable = [law.readable for law in explanation.laws]
    dependencies = {
        node: [dep for dep in deps if dep in explanation.dependencies]
        for node, deps in explanation.dependencies.items()
    }

    # -- individual view fragments (reusing the existing renderers) --------- #
    panels: list[str] = []

    span = explanation.time_span
    overview = (
        f'<dl style="margin:0;display:grid;grid-template-columns:auto 1fr;gap:2px 16px">'
        f'<dt style="color:{colors["muted"]}">samples</dt><dd style="margin:0">{explanation.sample_count}</dd>'
        f'<dt style="color:{colors["muted"]}">time span</dt><dd style="margin:0">[{span[0]:.4g}, {span[1]:.4g}]</dd>'
        f'<dt style="color:{colors["muted"]}">state variables</dt><dd style="margin:0">{escape(", ".join(states))}</dd>'
        f'<dt style="color:{colors["muted"]}">laws discovered</dt><dd style="margin:0">{len(explanation.laws)}</dd>'
        f"</dl>"
    )
    panels.append(panel("Overview", overview, config.theme))

    panels.append(render_equations(equations, config.theme).html)
    panels.append(panel("Readable laws", _laws_list_html(laws_readable, config.theme), config.theme))

    if dependencies:
        panels.append(render_graph(dependencies, config.theme).html)

    panels.append(panel(
        "Fit quality (simulation vs. observations)",
        _fit_table_html(explanation.fit, config.theme),
        config.theme,
    ))

    chart = svg_line_chart(
        trajectory.time, dict(trajectory.values),
        width=760, height=340, title="Simulated trajectory", theme=config.theme,
    )
    panels.append(panel("Trajectory", chart, config.theme))

    uncertainty_entries = _uncertainty_entries(explanation.fit, finals)
    if uncertainty_entries:
        band = render_uncertainty(uncertainty_entries, config.theme)
        note = (
            f'<p style="margin:6px 0 0;color:{colors["muted"]};font-size:12px">'
            "±RMSE band around each state's final simulated value.</p>"
        )
        # Splice the explanatory note into the rendered panel.
        panels.append(band.html.replace("</section>", note + "</section>"))

    frontier_candidates = _frontier_candidates(explanation.laws, explanation.fit)
    if frontier_candidates:
        front = render_frontier(frontier_candidates, config.theme)
        note = (
            f'<p style="margin:6px 0 0;color:{colors["muted"]};font-size:12px">'
            "Discovered laws ranked by fit error (1 − R²) vs. term complexity.</p>"
        )
        panels.append(front.html.replace("</section>", note + "</section>"))

    if regimes:
        panels.append(render_regimes(regimes, config.theme).html)

    if comparison is not None:
        try:
            panels.append(comparison._repr_html_(theme=config.theme))
        except TypeError:  # comparison view without a theme kwarg
            panels.append(comparison._repr_html_())

    # -- outer themed container --------------------------------------------- #
    header = (
        f'<header style="margin:0 0 6px">'
        f'<h2 style="margin:0;font-size:20px;color:{colors["accent"]}">'
        f'LawSynth dashboard — {escape(str(source.name))}</h2>'
        f'<p style="margin:2px 0 0;color:{colors["muted"]}">'
        "Interpretable, deterministic, local-first — the full picture in one view.</p>"
        f"</header>"
    )
    fragment = (
        f'<section class="lawsynth-dashboard" style="background:{colors["background"]};'
        f'color:{colors["foreground"]};border:1px solid {colors["border"]};border-radius:12px;'
        f'padding:16px 18px;margin:8px 0;font:14px/1.5 system-ui,sans-serif">'
        f"{header}{''.join(panels)}</section>"
    )
    data: dict[str, Any] = {
        "name": str(source.name),
        "theme": config.theme,
        "explanation": explanation.to_dict(),
        "trajectory": {"time": list(trajectory.time), "values": {k: list(v) for k, v in trajectory.values.items()}},
        "uncertainty": uncertainty_entries,
        "frontier": frontier_candidates,
    }
    if comparison is not None and hasattr(comparison, "to_dict"):
        data["scenarios"] = comparison.to_dict()
    return fragment, data


@dataclass(frozen=True, slots=True)
class StudyDashboard:
    """A composed, themed dashboard view for a discovered world.

    Renders inline in Jupyter via ``_repr_html_`` and can be serialized to a
    self-contained HTML document via :meth:`to_document` for sharing.
    """

    title: str
    fragment: str
    data: dict[str, Any]
    theme: str = "light"

    def _repr_html_(self) -> str:
        return self.fragment

    def _repr_mimebundle_(self, **_: object) -> dict[str, str]:
        return {"text/html": self.fragment, "text/plain": repr(self)}

    def to_document(self) -> str:
        """Wrap the dashboard fragment in a standalone, portable HTML document."""
        colors = palette(self.theme)
        return (
            '<!doctype html><html lang="en"><head><meta charset="utf-8">'
            '<meta name="viewport" content="width=device-width,initial-scale=1">'
            f"<title>{escape(self.title)}</title></head>"
            f'<body style="background:{colors["border"]};margin:0;padding:24px">'
            f'<main style="max-width:880px;margin:0 auto">{self.fragment}</main>'
            "</body></html>"
        )

    def __repr__(self) -> str:
        return f"StudyDashboard(title={self.title!r}, theme={self.theme!r}, keys={sorted(self.data)})"


def render_dashboard(
    source: Any,
    *,
    theme: str = "light",
    comparison: Any = None,
    regimes: Sequence[Mapping[str, Any]] | None = None,
    horizon: float | None = None,
) -> StudyDashboard:
    """Build a :class:`StudyDashboard` for a discovered ``Study``/``DiscoveryResult``."""
    fragment, data = build_dashboard_html(
        source, theme=theme, comparison=comparison, regimes=regimes, horizon=horizon,
    )
    return StudyDashboard(
        title=f"LawSynth dashboard — {source.name}",
        fragment=fragment,
        data=data,
        theme=NotebookConfig(theme=theme).theme,
    )
