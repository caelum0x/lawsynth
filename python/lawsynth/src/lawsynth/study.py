"""A fluent, notebook-friendly façade over the LawSynth product loop.

``Study`` collapses the whole ``observe -> discover -> understand -> use ->
share`` loop into a few lines, backing every step with the real SDK and native
engine calls (dataset ingest, ``discover_world``, native simulation, and the
``.lsworld`` bundle codec). Objects returned from a study render richly in
Jupyter and can be exported to a self-contained HTML report.
"""

from __future__ import annotations

import ast
import csv
from dataclasses import dataclass, field
from html import escape
from math import isfinite
from os import PathLike
from pathlib import Path
from statistics import median
from typing import Mapping, Sequence

from . import report as _report
from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import LawSynthError, NativeError, ValidationError
from .lineage import Lineage
from .trajectory import TrajectoryData

__all__ = [
    "Study", "DiscoveryResult", "Explanation", "Law", "Forecast",
    "ScenarioComparison", "enable_rich_display",
]


def _default_source_name(kind: str, resource: str) -> str:
    stem = Path(resource).stem or resource
    return f"{kind}:{stem}"


def _apply_overrides(base: DiscoveryConfig, overrides: Mapping[str, object]) -> DiscoveryConfig:
    """Return a new config = ``base`` with ``overrides`` layered on (overrides win).

    A slots dataclass has no ``__dict__``, so rebuild from its declared fields
    explicitly rather than copying an instance dict.
    """
    merged = {name: getattr(base, name) for name in DiscoveryConfig.__dataclass_fields__}
    unknown = set(overrides) - set(merged)
    if unknown:
        raise ValidationError(f"unknown discovery options: {sorted(unknown)}")
    merged.update(overrides)
    return DiscoveryConfig(**merged)


# --------------------------------------------------------------------------- #
# Equation understanding — parse native expressions into readable, ranked laws #
# --------------------------------------------------------------------------- #


def _format_coeff(value: float) -> str:
    text = f"{value:.4g}"
    return text


def _format_feature(factors: Sequence[str]) -> str:
    if not factors:
        return "1"
    counts: dict[str, int] = {}
    for name in factors:
        counts[name] = counts.get(name, 0) + 1
    parts = []
    for name in sorted(counts):
        power = counts[name]
        parts.append(name if power == 1 else f"{name}^{power}")
    return "·".join(parts)


def _extract_terms(expression: str) -> tuple[tuple[float, tuple[str, ...]], ...]:
    """Flatten a native world expression into additive (coefficient, factors) terms.

    Native expressions are valid arithmetic (e.g. ``((-3.0e-1*y)+(5.1e-1*x))``),
    so the standard-library ``ast`` parses them safely without evaluation.
    """
    try:
        tree = ast.parse(expression, mode="eval").body
    except SyntaxError as error:  # pragma: no cover - native output is well-formed
        raise ValidationError(f"cannot parse equation {expression!r}") from error

    terms: list[tuple[float, tuple[str, ...]]] = []

    def walk(node: ast.AST, sign: float) -> None:
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Add):
            walk(node.left, sign)
            walk(node.right, sign)
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Sub):
            walk(node.left, sign)
            walk(node.right, -sign)
        elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.UAdd):
            walk(node.operand, sign)
        elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
            walk(node.operand, -sign)
        else:
            coeff = [sign]
            factors: list[str] = []
            _product(node, coeff, factors)
            terms.append((coeff[0], tuple(factors)))

    def _product(node: ast.AST, coeff: list[float], factors: list[str]) -> None:
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.Mult):
            _product(node.left, coeff, factors)
            _product(node.right, coeff, factors)
        elif isinstance(node, ast.Constant) and isinstance(node.value, (int, float)):
            coeff[0] *= float(node.value)
        elif isinstance(node, ast.Name):
            factors.append(node.id)
        elif isinstance(node, ast.UnaryOp) and isinstance(node.op, ast.USub):
            coeff[0] *= -1.0
            _product(node.operand, coeff, factors)
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Pow) and isinstance(node.left, ast.Name) and isinstance(node.right, ast.Constant):
            factors.extend([node.left.id] * int(node.right.value))
        elif isinstance(node, ast.BinOp) and isinstance(node.op, ast.Div) and isinstance(node.right, ast.Constant):
            _product(node.left, coeff, factors)
            coeff[0] /= float(node.right.value)
        else:  # pragma: no cover - defensive against unexpected native output
            raise ValidationError(f"unsupported equation node: {ast.dump(node)}")

    walk(tree, 1.0)
    terms.sort(key=lambda item: -abs(item[0]))
    return tuple(terms)


@dataclass(frozen=True, slots=True)
class Law:
    """A single discovered evolution law with a plain-language reading."""

    target: str
    expression: str
    readable: str
    terms: tuple[tuple[float, str], ...]
    dominant: str

    def to_dict(self) -> dict[str, object]:
        return {
            "target": self.target,
            "expression": self.expression,
            "readable": self.readable,
            "terms": [{"coefficient": c, "feature": f} for c, f in self.terms],
            "dominant_term": self.dominant,
        }


def _build_law(target: str, expression: str) -> Law:
    raw = _extract_terms(expression)
    terms = tuple((coeff, _format_feature(factors)) for coeff, factors in raw)
    pieces = []
    for i, (coeff, feature) in enumerate(terms):
        magnitude = _format_coeff(abs(coeff))
        sign = "-" if coeff < 0 else "+"
        chunk = magnitude if feature == "1" else f"{magnitude}·{feature}"
        if i == 0:
            pieces.append(f"-{chunk}" if coeff < 0 else chunk)
        else:
            pieces.append(f"{sign} {chunk}")
    rhs = " ".join(pieces) if pieces else "0"
    readable = f"d{target}/dt = {rhs}"
    dominant = terms[0][1] if terms else "1"
    return Law(target, expression, readable, terms, dominant)


# --------------------------------------------------------------------------- #
# Explanation — structured, human-readable summary of a discovered world       #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class Explanation:
    """A structured, trustworthy summary of what discovery found."""

    name: str
    laws: tuple[Law, ...]
    variables: tuple[str, ...]
    fit: Mapping[str, Mapping[str, float]]
    dependencies: Mapping[str, tuple[str, ...]]
    assumptions: tuple[str, ...]
    sample_count: int
    time_span: tuple[float, float]

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "variables": list(self.variables),
            "laws": [law.to_dict() for law in self.laws],
            "fit": {k: dict(v) for k, v in self.fit.items()},
            "dependencies": {k: list(v) for k, v in self.dependencies.items()},
            "assumptions": list(self.assumptions),
            "sample_count": self.sample_count,
            "time_span": list(self.time_span),
        }

    def to_text(self) -> str:
        lines = [
            f"Study: {self.name}",
            f"Observed {self.sample_count} samples over t ∈ "
            f"[{self.time_span[0]:.4g}, {self.time_span[1]:.4g}]",
            f"State variables: {', '.join(self.variables)}",
            "",
            "Discovered laws:",
        ]
        for law in self.laws:
            lines.append(f"  {law.readable}")
            lines.append(f"      dominant term: {law.dominant}")
        lines.append("")
        lines.append("Fit quality (simulation vs. observations):")
        for state, metrics in sorted(self.fit.items()):
            lines.append(
                f"  {state}: R² = {metrics['r_squared']:.4f}, "
                f"RMSE = {metrics['rmse']:.4g}"
            )
        lines.append("")
        lines.append("Dependency structure:")
        for node, deps in sorted(self.dependencies.items()):
            lines.append(f"  {node} depends on: {', '.join(deps) or '(none)'}")
        lines.append("")
        lines.append("Assumptions:")
        for item in self.assumptions:
            lines.append(f"  - {item}")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def _repr_html_(self) -> str:
        equations = {law.target: law.expression for law in self.laws}
        return _report.build_report(
            title=f"Explanation — {self.name}",
            summary=[
                ("samples", str(self.sample_count)),
                ("time span", f"[{self.time_span[0]:.4g}, {self.time_span[1]:.4g}]"),
                ("state variables", ", ".join(self.variables)),
            ],
            equations=equations,
            laws_readable=[law.readable for law in self.laws],
            fit=self.fit,
            trajectory_time=(),
            trajectory_values={},
            dependencies={k: list(v) for k, v in self.dependencies.items()},
            assumptions=self.assumptions,
        )


# --------------------------------------------------------------------------- #
# Forecast — baseline vs. counterfactual what-if                               #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class Forecast:
    """A baseline trajectory alongside an intervened (what-if) trajectory."""

    baseline: TrajectoryData
    counterfactual: TrajectoryData
    interventions: Mapping[str, float]
    divergence: Mapping[str, float]

    def to_dict(self) -> dict[str, object]:
        return {
            "interventions": dict(self.interventions),
            "divergence": dict(self.divergence),
            "baseline": {"time": list(self.baseline.time), "values": {k: list(v) for k, v in self.baseline.values.items()}},
            "counterfactual": {"time": list(self.counterfactual.time), "values": {k: list(v) for k, v in self.counterfactual.values.items()}},
        }

    def _overlay_series(self) -> dict[str, tuple[float, ...]]:
        series: dict[str, tuple[float, ...]] = {}
        for name, column in self.baseline.values.items():
            series[name] = column
        for name, column in self.counterfactual.values.items():
            series[f"{name}*"] = column
        return series

    def _repr_html_(self) -> str:
        chart = _report.svg_line_chart(
            self.counterfactual.time, self._overlay_series(),
            width=720, height=340, title="Forecast: baseline vs. what-if (*)",
        )
        rows = "".join(
            f"<tr><td style='padding:3px 10px'>{name}</td>"
            f"<td style='padding:3px 10px'>{value:.4g}</td></tr>"
            for name, value in sorted(self.interventions.items())
        )
        div = "".join(
            f"<tr><td style='padding:3px 10px'>{name}</td>"
            f"<td style='padding:3px 10px'>{value:.4g}</td></tr>"
            for name, value in sorted(self.divergence.items())
        )
        return (
            '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;padding:14px;margin:8px 0">'
            '<h3 style="margin:0 0 8px;color:#155e75">Forecast (what-if)</h3>'
            f"{chart}"
            '<div style="display:flex;gap:24px;margin-top:10px">'
            f'<div><b>Interventions (initial overrides)</b><table>{rows}</table></div>'
            f'<div><b>Final divergence |what-if − baseline|</b><table>{div}</table></div>'
            "</div></section>"
        )


# --------------------------------------------------------------------------- #
# ScenarioComparison — overlay N what-if scenarios against a baseline          #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class ScenarioComparison:
    """A baseline run plus N labeled what-if scenarios, ready to compare.

    Holds one :class:`TrajectoryData` per label (the ``baseline`` plus each
    named scenario) on a shared time grid, together with the initial-condition
    overrides that define each scenario. Renders as a self-contained HTML view:
    per-state SVG line charts overlaying every scenario, plus a divergence table
    quantifying how far each scenario's final state drifts from the baseline.
    """

    states: tuple[str, ...]
    baseline_label: str
    order: tuple[str, ...]
    trajectories: Mapping[str, TrajectoryData]
    interventions: Mapping[str, Mapping[str, float]]

    # -- data access -------------------------------------------------------- #

    @property
    def labels(self) -> tuple[str, ...]:
        """Every label, baseline first, then scenarios in insertion order."""
        return (self.baseline_label, *self.order)

    def final_state(self, label: str) -> dict[str, float]:
        trajectory = self.trajectories[label]
        return {state: float(trajectory.values[state][-1]) for state in self.states if state in trajectory.values}

    def divergence(self, label: str) -> dict[str, float]:
        """Per-state |scenario_final − baseline_final| for ``label``."""
        base = self.final_state(self.baseline_label)
        current = self.final_state(label)
        return {state: abs(current[state] - base[state]) for state in self.states if state in base and state in current}

    def distance(self, label: str) -> float:
        """Euclidean norm of the final-state divergence from baseline."""
        return sum(value * value for value in self.divergence(label).values()) ** 0.5

    def to_rows(self) -> list[dict[str, object]]:
        rows: list[dict[str, object]] = []
        for label in self.labels:
            final = self.final_state(label)
            divergence = self.divergence(label)
            rows.append({
                "scenario": label,
                "interventions": dict(self.interventions.get(label, {})),
                "final": final,
                "divergence": divergence,
                "distance": self.distance(label),
            })
        return rows

    def to_dict(self) -> dict[str, object]:
        return {
            "states": list(self.states),
            "baseline": self.baseline_label,
            "scenarios": list(self.order),
            "rows": self.to_rows(),
        }

    # -- text table --------------------------------------------------------- #

    def table(self) -> str:
        """A plain-text comparison table: final state + divergence from baseline."""
        headers = ["scenario", *[f"{s}(final)" for s in self.states], *[f"Δ{s}" for s in self.states], "‖Δ‖"]
        rows: list[list[str]] = []
        for label in self.labels:
            final = self.final_state(label)
            divergence = self.divergence(label)
            cells = [label]
            cells += [f"{final.get(s, float('nan')):.4g}" for s in self.states]
            cells += [f"{divergence.get(s, 0.0):.4g}" for s in self.states]
            cells.append(f"{self.distance(label):.4g}")
            rows.append(cells)
        widths = [max(len(headers[i]), *(len(row[i]) for row in rows)) for i in range(len(headers))]
        def _fmt(cells: Sequence[str]) -> str:
            return "  ".join(cell.rjust(widths[i]) if i else cell.ljust(widths[i]) for i, cell in enumerate(cells))
        divider = "  ".join("-" * widths[i] for i in range(len(headers)))
        return "\n".join([_fmt(headers), divider, *[_fmt(row) for row in rows]])

    def __str__(self) -> str:
        return self.table()

    # -- HTML view ---------------------------------------------------------- #

    def _repr_html_(self, *, theme: str = "light") -> str:
        colors = _report._theme(theme)
        baseline = self.trajectories[self.baseline_label]
        charts = []
        for state in self.states:
            overlay = {
                label: self.trajectories[label].values[state]
                for label in self.labels
                if state in self.trajectories[label].values
            }
            charts.append(_report.svg_line_chart(
                baseline.time, overlay, width=720, height=300,
                title=f"{state}: scenarios overlaid", theme=theme, sort_series=False,
            ))
        # Divergence table: one row per label, final value + Δ vs baseline per state.
        head_cells = "".join(
            f'<th style="padding:4px 10px;text-align:right;color:{colors["muted"]}">{escape(h)}</th>'
            for h in ["scenario", *[f"{s} final" for s in self.states], *[f"Δ{s}" for s in self.states], "‖Δ‖"]
        )
        body_rows = []
        for label in self.labels:
            final = self.final_state(label)
            divergence = self.divergence(label)
            is_baseline = label == self.baseline_label
            name_cell = (
                f'<td style="padding:4px 10px;font-weight:600;color:{colors["accent"]}">'
                f'{escape(label)}{" (baseline)" if is_baseline else ""}</td>'
            )
            value_cells = "".join(
                f'<td style="padding:4px 10px;text-align:right">{final.get(s, float("nan")):.4g}</td>'
                for s in self.states
            )
            div_cells = "".join(
                f'<td style="padding:4px 10px;text-align:right">{divergence.get(s, 0.0):.4g}</td>'
                for s in self.states
            )
            dist_cell = f'<td style="padding:4px 10px;text-align:right;font-weight:600">{self.distance(label):.4g}</td>'
            body_rows.append(f"<tr>{name_cell}{value_cells}{div_cells}{dist_cell}</tr>")
        table_html = (
            f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
            f'<thead><tr>{head_cells}</tr></thead><tbody>{"".join(body_rows)}</tbody></table>'
        )
        return (
            f'<section style="font:14px system-ui;border:1px solid {colors["border"]};'
            f'border-radius:10px;padding:14px;margin:8px 0;background:{colors["bg"]};color:{colors["fg"]}">'
            f'<h3 style="margin:0 0 8px;color:{colors["accent"]}">Scenario comparison — '
            f'{len(self.order)} what-if{"s" if len(self.order) != 1 else ""} vs. baseline</h3>'
            + "".join(charts)
            + '<div style="margin-top:10px">'
            f'<b style="color:{colors["accent"]}">Final divergence from baseline</b>{table_html}</div>'
            "</section>"
        )

    def __repr__(self) -> str:
        return f"ScenarioComparison(baseline={self.baseline_label!r}, scenarios={list(self.order)}, states={list(self.states)})"


def _compare_scenarios(
    world: object,
    dataset: Dataset,
    states: Sequence[str],
    scenarios: Mapping[str, Mapping[str, float]],
    *,
    baseline_label: str,
    horizon: float | None,
    step: float | None,
) -> ScenarioComparison:
    baseline_initial = _initial_state(dataset, states)
    trajectories: dict[str, TrajectoryData] = {
        baseline_label: _simulate(world, dataset, states, horizon=horizon, initial=baseline_initial, start=None, step=step)
    }
    resolved_interventions: dict[str, dict[str, float]] = {baseline_label: {}}
    for label, overrides in scenarios.items():
        unknown = [key for key in overrides if key not in baseline_initial]
        if unknown:
            raise ValidationError(
                f"scenario {label!r} names unknown state variables {unknown}; valid states are {sorted(baseline_initial)}"
            )
        initial = {**baseline_initial, **{k: float(v) for k, v in overrides.items()}}
        trajectories[label] = _simulate(world, dataset, states, horizon=horizon, initial=initial, start=None, step=step)
        resolved_interventions[label] = {k: float(v) for k, v in overrides.items()}
    return ScenarioComparison(
        states=tuple(states),
        baseline_label=baseline_label,
        order=tuple(scenarios.keys()),
        trajectories=trajectories,
        interventions=resolved_interventions,
    )


# --------------------------------------------------------------------------- #
# Shared operations backing both Study and DiscoveryResult                     #
# --------------------------------------------------------------------------- #


def _resolve_config(
    config: DiscoveryConfig | None,
    recipe: str | None,
    overrides: Mapping[str, object],
) -> DiscoveryConfig:
    """Resolve a discovery config from an optional recipe/config plus overrides.

    ``recipe`` and ``config`` are mutually exclusive: a recipe *is* a starting
    config. Explicit ``overrides`` always layer on top and win.
    """
    if recipe is not None:
        if config is not None:
            raise ValidationError(
                "pass either a recipe or a config, not both; explicit "
                "**overrides refine whichever you choose"
            )
        from .recipes import get as _get_recipe

        return _get_recipe(recipe).merge(dict(overrides))
    base = config or DiscoveryConfig()
    if overrides:
        base = _apply_overrides(base, overrides)
    return base


def _discover_world(dataset: Dataset, states: Sequence[str], config: DiscoveryConfig) -> object:
    """Run the native discovery boundary for ``dataset`` under ``config``.

    This is the single choke-point every discovery path funnels through — the
    fluent :meth:`Study.discover`, bootstrap ensembles, and prepared studies all
    share it, so the lazy-native contract and argument wiring stay in one place.
    """
    time, columns = dataset.as_native_arguments()
    try:
        from ._native import discover_world
    except ImportError as error:
        raise NativeError("the lawsynth native extension is unavailable; build it first") from error
    try:
        return discover_world(
            time, columns, list(states),
            polynomial_degree=config.polynomial_degree,
            threshold=config.threshold,
            solver=config.solver,
            include_trigonometric=config.include_trigonometric,
            include_rational=config.include_rational,
            smoothing_radius=config.smoothing_radius,
            derivative_method=config.derivative_method,
            savgol_window=5,
            tvreg_lambda=0.1,
            tvreg_iterations=100,
            symbolic_depth=config.symbolic_depth,
        )
    except Exception as error:
        raise NativeError(f"discovery failed: {error}") from error


def _default_step(dataset: Dataset) -> float:
    if len(dataset.time) < 2:
        return 0.1
    diffs = [b - a for a, b in zip(dataset.time, dataset.time[1:])]
    return float(median(diffs))


def _initial_state(dataset: Dataset, states: Sequence[str], index: int = 0) -> dict[str, float]:
    return {state: float(dataset.columns[state][index]) for state in states}


def _run_native(world: object, initial: Mapping[str, float], *, start: float, end: float, step: float) -> TrajectoryData:
    try:
        native = world.simulate(dict(initial), start=start, end=end, step=step)
    except Exception as error:  # native raises plain exceptions
        raise NativeError(f"simulation failed: {error}") from error
    return TrajectoryData.from_native(native)


def _simulate(
    world: object,
    dataset: Dataset,
    states: Sequence[str],
    *,
    horizon: float | None,
    initial: Mapping[str, float] | None,
    start: float | None,
    step: float | None,
) -> TrajectoryData:
    start_time = float(start) if start is not None else float(dataset.time[0])
    span = float(horizon) if horizon is not None else float(dataset.time[-1] - dataset.time[0])
    if span <= 0:
        raise ValidationError("horizon must be positive")
    resolved_step = float(step) if step is not None else _default_step(dataset)
    if resolved_step <= 0:
        raise ValidationError("step must be positive")
    resolved_initial = dict(initial) if initial is not None else _initial_state(dataset, states)
    missing = [state for state in states if state not in resolved_initial]
    if missing:
        raise ValidationError(f"missing initial values for {missing}")
    return _run_native(world, resolved_initial, start=start_time, end=start_time + span, step=resolved_step)


def _fit_quality(world: object, dataset: Dataset, states: Sequence[str]) -> dict[str, dict[str, float]]:
    """Simulate across the observed window and score against observations."""
    step = _default_step(dataset)
    trajectory = _run_native(
        world, _initial_state(dataset, states),
        start=float(dataset.time[0]), end=float(dataset.time[-1]), step=step,
    )
    fit: dict[str, dict[str, float]] = {}
    for state in states:
        observed = dataset.columns[state]
        simulated = trajectory.values.get(state, ())
        count = min(len(observed), len(simulated))
        if count == 0:
            fit[state] = {"r_squared": 0.0, "rmse": float("inf")}
            continue
        obs = observed[:count]
        sim = simulated[:count]
        mean = sum(obs) / count
        ss_tot = sum((o - mean) ** 2 for o in obs) or 1e-12
        ss_res = sum((o - s) ** 2 for o, s in zip(obs, sim))
        rmse = (ss_res / count) ** 0.5
        fit[state] = {"r_squared": 1.0 - ss_res / ss_tot, "rmse": rmse}
    return fit


def _dependencies(laws: Sequence[Law], states: Sequence[str]) -> dict[str, tuple[str, ...]]:
    state_set = set(states)
    graph: dict[str, tuple[str, ...]] = {}
    for law in laws:
        used = {factor for _, feature in law.terms for factor in feature.replace("·", " ").replace("^2", "").replace("^3", "").split() if factor in state_set}
        graph[law.target] = tuple(sorted(used))
    return graph


def _explain(world: object, dataset: Dataset, states: Sequence[str], *, name: str) -> Explanation:
    equations = dict(world.equations())
    laws = tuple(_build_law(target, equations[target]) for target in sorted(equations))
    fit = _fit_quality(world, dataset, states)
    dependencies = _dependencies(laws, states)
    assumptions = (
        "Continuous-time dynamics: each law models a first derivative (dX/dt).",
        "Polynomial feature library; only above-threshold terms are retained.",
        "Deterministic and offline — identical inputs reproduce this world exactly.",
        "Fit measured by forward simulation from the first observation; "
        "extrapolation beyond the observed window is not validated.",
    )
    return Explanation(
        name=name,
        laws=laws,
        variables=tuple(states),
        fit=fit,
        dependencies=dependencies,
        assumptions=assumptions,
        sample_count=len(dataset.time),
        time_span=(float(dataset.time[0]), float(dataset.time[-1])),
    )


def _forecast(
    world: object,
    dataset: Dataset,
    states: Sequence[str],
    *,
    interventions: Mapping[str, float],
    horizon: float | None,
    step: float | None,
) -> Forecast:
    baseline_initial = _initial_state(dataset, states)
    unknown = [key for key in interventions if key not in baseline_initial]
    if unknown:
        raise ValidationError(
            f"interventions must name state variables {sorted(baseline_initial)}; got unknown {unknown}"
        )
    counterfactual_initial = {**baseline_initial, **{k: float(v) for k, v in interventions.items()}}
    baseline = _simulate(world, dataset, states, horizon=horizon, initial=baseline_initial, start=None, step=step)
    counterfactual = _simulate(world, dataset, states, horizon=horizon, initial=counterfactual_initial, start=None, step=step)
    divergence = {
        state: abs(counterfactual.values[state][-1] - baseline.values[state][-1])
        for state in states
        if state in baseline.values and state in counterfactual.values
    }
    return Forecast(baseline, counterfactual, dict(interventions), divergence)


def _report_html(world: object, dataset: Dataset, states: Sequence[str], *, name: str, theme: str) -> str:
    explanation = _explain(world, dataset, states, name=name)
    preview = _simulate(world, dataset, states, horizon=None, initial=None, start=None, step=None)
    return _report.build_report(
        title=f"LawSynth Study — {name}",
        summary=[
            ("samples", str(explanation.sample_count)),
            ("time span", f"[{explanation.time_span[0]:.4g}, {explanation.time_span[1]:.4g}]"),
            ("state variables", ", ".join(states)),
            ("laws discovered", str(len(explanation.laws))),
        ],
        equations={law.target: law.expression for law in explanation.laws},
        laws_readable=[law.readable for law in explanation.laws],
        fit=explanation.fit,
        trajectory_time=preview.time,
        trajectory_values=preview.values,
        dependencies={k: list(v) for k, v in explanation.dependencies.items()},
        assumptions=explanation.assumptions,
        theme=theme,
    )


# --------------------------------------------------------------------------- #
# DiscoveryResult — the rendered, reusable output of Study.discover            #
# --------------------------------------------------------------------------- #


class DiscoveryResult:
    """A discovered world plus its originating data, ready to render and use."""

    __slots__ = ("_world", "_dataset", "_states", "_config", "_name")

    def __init__(self, world: object, dataset: Dataset, states: Sequence[str], config: DiscoveryConfig, name: str) -> None:
        self._world = world
        self._dataset = dataset
        self._states = tuple(states)
        self._config = config
        self._name = name

    @property
    def world(self) -> object:
        """The underlying native executable World."""
        return self._world

    @property
    def name(self) -> str:
        return self._name

    @property
    def states(self) -> tuple[str, ...]:
        return self._states

    @property
    def equations(self) -> dict[str, str]:
        return dict(self._world.equations())

    def explain(self) -> Explanation:
        return _explain(self._world, self._dataset, self._states, name=self._name)

    def simplify(self):
        """Simplify each law into a canonical, equivalence-checked form.

        Returns a :class:`~lawsynth.simplification.SimplifiedWorld` report; its
        ``.world`` is a new, equivalent native world and ``.verify(initial)``
        confirms the trajectories match to floating-point precision.
        """
        from .simplification import simplify_world

        return simplify_world(self._world)

    def simulate(self, *, horizon: float | None = None, initial: Mapping[str, float] | None = None, start: float | None = None, step: float | None = None) -> TrajectoryData:
        return _simulate(self._world, self._dataset, self._states, horizon=horizon, initial=initial, start=start, step=step)

    def forecast(self, interventions: Mapping[str, float], *, horizon: float | None = None, step: float | None = None) -> Forecast:
        return _forecast(self._world, self._dataset, self._states, interventions=interventions, horizon=horizon, step=step)

    def backtest(self, *, origins: int = 5, horizon: int | None = None, step: float | None = None):
        """Rolling-origin forecast evaluation (see :meth:`Study.backtest`)."""
        from .backtesting import backtest as _backtest

        return _backtest(
            self._world, self._dataset,
            state=self._states, origins=origins, horizon=horizon, step=step,
            name=self._name,
        )

    def validate(self, *, holdout: float = 0.25, step: float | None = None):
        """Out-of-sample holdout validation (see :meth:`Study.validate`)."""
        from .validation import validate as _validate

        return _validate(
            self._dataset, self._states, self._config,
            holdout=holdout, step=step, name=self._name,
        )

    @property
    def lineage(self) -> Lineage:
        """A content-addressed lineage chain: dataset -> discovery -> world."""
        return Lineage.from_dataset(self._dataset, self._states).record_discovery(
            self._config, self._world
        )

    def report(self, path: str | PathLike[str], *, theme: str = "light") -> Path:
        return _write_report(self._world, self._dataset, self._states, path, name=self._name, theme=theme)

    def dashboard(self, *, theme: str = "light", horizon: float | None = None):
        """Render a cohesive notebook dashboard (requires ``lawsynth-notebook``)."""
        from lawsynth_notebook.dashboard import render_dashboard

        return render_dashboard(self, theme=theme, comparison=None, horizon=horizon)

    def save(self, path: str | PathLike[str]) -> Path:
        target = Path(path)
        self._world.save(str(target))
        return target

    def _repr_html_(self) -> str:
        try:
            return self.dashboard()._repr_html_()
        except Exception:
            return _report_html(self._world, self._dataset, self._states, name=self._name, theme="light")

    def __repr__(self) -> str:
        return f"DiscoveryResult(name={self._name!r}, states={list(self._states)}, laws={len(self.equations)})"


def _write_report(world: object, dataset: Dataset, states: Sequence[str], path: str | PathLike[str], *, name: str, theme: str) -> Path:
    target = Path(path)
    if target.suffix.lower() not in {".html", ".htm"}:
        raise ValidationError("report path must end in .html or .htm")
    document = _report_html(world, dataset, states, name=name, theme=theme)
    target.write_text(document, encoding="utf-8")
    return target


# --------------------------------------------------------------------------- #
# Study — the fluent façade                                                     #
# --------------------------------------------------------------------------- #


class Study:
    """A fluent workflow over one dataset: discover, explain, forecast, share."""

    __slots__ = ("_dataset", "_states", "_name", "_world", "_config", "_scenarios", "_lineage")

    def __init__(self, dataset: Dataset, states: Sequence[str], *, name: str = "study") -> None:
        if not states:
            raise ValidationError("at least one state variable is required")
        missing = [state for state in states if state not in dataset.columns]
        if missing:
            raise ValidationError(f"state variables not present in dataset: {missing}")
        self._dataset = dataset
        self._states = tuple(states)
        self._name = name
        self._world: object | None = None
        self._config: DiscoveryConfig | None = None
        self._scenarios: dict[str, dict[str, float]] = {}
        # Governance lineage is captured as the study progresses: it is rooted at
        # the source dataset's content hash and extended at discover().
        self._lineage: Lineage = Lineage.from_dataset(dataset, self._states)

    # -- construction ------------------------------------------------------- #

    @classmethod
    def from_dataset(cls, dataset: Dataset, *, state: Sequence[str], name: str = "study") -> "Study":
        return cls(dataset, state, name=name)

    @classmethod
    def from_columns(cls, time: Sequence[float], columns: Mapping[str, Sequence[float]], *, state: Sequence[str], name: str = "study") -> "Study":
        return cls(Dataset.from_columns(time, columns), state, name=name)

    @classmethod
    def from_csv(cls, path: str | PathLike[str], *, time: str, state: Sequence[str], name: str | None = None, delimiter: str = ",") -> "Study":
        """Ingest a CSV into a validated :class:`Dataset` and build a study."""
        source = Path(path)
        state = list(state)
        if not state:
            raise ValidationError("at least one state column is required")
        try:
            raw = source.read_text(encoding="utf-8")
        except OSError as error:
            raise ValidationError(f"cannot read CSV {source}: {error}") from error
        reader = csv.DictReader(raw.splitlines(), delimiter=delimiter)
        if reader.fieldnames is None:
            raise ValidationError("CSV is empty or has no header row")
        required = [time, *state]
        missing = [column for column in required if column not in reader.fieldnames]
        if missing:
            raise ValidationError(f"CSV is missing required columns {missing}; found {reader.fieldnames}")
        times: list[float] = []
        columns: dict[str, list[float]] = {column: [] for column in state}
        for line_number, row in enumerate(reader, start=2):
            try:
                times.append(float(row[time]))
                for column in state:
                    columns[column].append(float(row[column]))
            except (TypeError, ValueError) as error:
                raise ValidationError(f"non-numeric value on CSV line {line_number}: {error}") from error
        if not times:
            raise ValidationError("CSV contains a header but no data rows")
        dataset = Dataset.from_columns(times, columns)
        return cls(dataset, state, name=name or source.stem)

    @classmethod
    def from_source(
        cls,
        kind: str,
        resource: str,
        *,
        time: str,
        state: Sequence[str],
        options: Mapping[str, object] | None = None,
        credentials: object = None,
        name: str | None = None,
    ) -> "Study":
        """Ingest observations from any registered connector into a study.

        Bridges the ``lawsynth_connectors`` library through
        :func:`lawsynth.load_source`: it creates the named connector, reads a
        ``time``/``state`` projection in bounded batches, coerces the raw
        records into finite floats, and binds the resulting validated dataset
        to a study ready to :meth:`discover`. Example::

            Study.from_source("filesystem", "obs.csv", time="t", state=["x", "y"],
                              options={"root": "."}).discover().explain()
        """
        from .sources import load_source

        states = list(state)
        dataset = load_source(
            kind,
            resource,
            time=time,
            state=states,
            options=options,
            credentials=credentials,
        )
        return cls(dataset, states, name=name or _default_source_name(kind, resource))

    # -- workflow ----------------------------------------------------------- #

    @property
    def dataset(self) -> Dataset:
        return self._dataset

    @property
    def name(self) -> str:
        return self._name

    @property
    def states(self) -> tuple[str, ...]:
        return self._states

    def profile(self):
        """Profile the study's dataset (quality report before discovery).

        Returns a :class:`~lawsynth.profile.DataProfile` with per-column
        statistics, time monotonicity and sampling regularity, degenerate
        columns, and quality warnings. Pure standard library and deterministic;
        works before :meth:`discover`.
        """
        from .profile import profile as _profile

        return _profile(self._dataset, name=self._name)

    def prepare(
        self,
        *,
        trim: tuple[float, float] | None = None,
        resample_dt: float | None = None,
        smooth: int | None = None,
        detrend: bool | Sequence[str] = False,
        columns: Sequence[str] | None = None,
        name: str | None = None,
    ) -> "Study":
        """Return a new study on a cleaned copy of this study's dataset.

        Applies, in order, window ``trim`` -> uniform ``resample_dt`` (linear
        interpolation onto a regular grid) -> moving-average ``smooth`` (window
        in samples) -> ``detrend`` (remove a per-column linear trend). Every
        operation is pure standard library and deterministic. The original study
        is left untouched; the returned study starts undiscovered on the cleaned
        data, ready to :meth:`discover`.
        """
        from .prepare import preprocess

        cleaned = preprocess(
            self._dataset,
            trim=trim,
            resample_dt=resample_dt,
            smooth=smooth,
            detrend=detrend,
            columns=columns,
        )
        return Study(cleaned, self._states, name=name or f"{self._name}+prepared")

    @property
    def world(self) -> object:
        self._require_world()
        return self._world

    def _require_world(self) -> object:
        if self._world is None:
            raise LawSynthError("call discover() before using the world")
        return self._world

    def simplify(self):
        """Simplify each law of the discovered world (see :meth:`DiscoveryResult.simplify`)."""
        from .simplification import simplify_world

        return simplify_world(self._require_world())

    def discover(
        self,
        config: DiscoveryConfig | None = None,
        *,
        recipe: str | None = None,
        **overrides: object,
    ) -> DiscoveryResult:
        """Discover an executable world from the study's observations.

        Pass ``recipe="ecology"`` (etc.) to start from a curated, per-domain
        preset — see :mod:`lawsynth.recipes`. Any explicit ``**overrides`` layer
        on top of the recipe (or ``config``) and always win. ``recipe`` and
        ``config`` are mutually exclusive: a recipe *is* a starting config.
        """
        base = _resolve_config(config, recipe, overrides)
        enable_rich_display()
        world = _discover_world(self._dataset, self._states, base)
        self._world = world
        self._config = base
        # Extend the lineage chain with the discovery config + world revision.
        self._lineage = self._lineage.record_discovery(base, world)
        return DiscoveryResult(world, self._dataset, self._states, base, self._name)

    def explain(self) -> Explanation:
        return _explain(self._require_world(), self._dataset, self._states, name=self._name)

    def simulate(self, *, horizon: float | None = None, initial: Mapping[str, float] | None = None, start: float | None = None, step: float | None = None) -> TrajectoryData:
        return _simulate(self._require_world(), self._dataset, self._states, horizon=horizon, initial=initial, start=start, step=step)

    def forecast(self, interventions: Mapping[str, float], *, horizon: float | None = None, step: float | None = None) -> Forecast:
        """Run a what-if: override initial conditions and compare to baseline."""
        return _forecast(self._require_world(), self._dataset, self._states, interventions=interventions, horizon=horizon, step=step)

    def backtest(self, *, origins: int = 5, horizon: int | None = None, step: float | None = None):
        """Rolling-origin (walk-forward) forecast evaluation of the world.

        Selects ``origins`` evenly spaced forecast origins across the observed
        series; from each, seeds the discovered world with the observed state and
        simulates forward ``horizon`` steps, scoring the forecast against the
        actual observations (RMSE/MAE/R² per state) and building a skill-vs-horizon
        decay curve. Returns a :class:`~lawsynth.backtest.Backtest`. Deterministic
        and offline — extrapolation quality, not just in-window fit.
        """
        from .backtesting import backtest as _backtest

        return _backtest(
            self._require_world(), self._dataset,
            state=self._states, origins=origins, horizon=horizon, step=step,
            name=self._name,
        )

    def validate(self, *, holdout: float = 0.25, step: float | None = None):
        """Out-of-sample **holdout** validation of the discovery procedure.

        Splits the observations in time, re-discovers a world on the leading
        ``1 - holdout`` fraction under this study's config, and scores its forecast
        on the held-out tail (RMSE/MAE/R² per state). Unlike in-window fit, this
        estimates whether the recovered structure *generalizes*. Returns a
        :class:`~lawsynth.validation.Validation`. Deterministic and offline.
        """
        from .validation import validate as _validate

        self._require_world()
        config = self._config if self._config is not None else DiscoveryConfig()
        return _validate(
            self._dataset, self._states, config,
            holdout=holdout, step=step, name=self._name,
        )

    @property
    def lineage(self):
        """The content-addressed lineage chain captured so far for this study.

        Rooted at the source dataset's content hash and extended at
        :meth:`discover` with the discovery config, engine version and world
        revision hash. See :class:`~lawsynth.lineage.Lineage`.
        """
        return self._lineage

    def model_card(self, **options: object):
        """Assemble a standardized governance **model card** for this world.

        Orchestrates the real SDK evaluations (``explain`` + holdout ``validate``
        + rolling-origin ``backtest`` + ``discover_ensemble`` + optional
        ``monitor``) and assembles them into a :class:`~lawsynth.governance.ModelCard`,
        honestly omitting any section whose evaluation was disabled or could not
        run. Keyword options are forwarded to
        :func:`lawsynth.governance.model_card`.
        """
        from .governance import model_card as _model_card

        self._require_world()
        return _model_card(self, **options)  # type: ignore[arg-type]

    def save_to_project(self, project: object, name: str, *, tags: Sequence[str] = (), note: str = ""):
        """Add this study's discovered world to a :class:`~lawsynth.project.Project`.

        A convenience over :meth:`Project.add` + :meth:`Project.save`: registers
        the world under ``name`` with optional ``tags``/``note``, persists the
        workspace to disk, and returns the ``project`` for chaining.
        """
        self._require_world()
        project.add(name, self, tags=tags, note=note)  # type: ignore[attr-defined]
        project.save()  # type: ignore[attr-defined]
        return project

    # -- uncertainty via ensemble discovery --------------------------------- #

    def discover_ensemble(
        self,
        *,
        n: int = 16,
        fraction: float = 0.8,
        seed: int = 0,
        config: DiscoveryConfig | None = None,
        recipe: str | None = None,
        **overrides: object,
    ):
        """Discover on ``n`` seeded bootstrap resamples to quantify uncertainty.

        Each member re-discovers the world on a deterministic subsample (an
        ``m``-of-``n`` draw *without* replacement, so the time axis stays valid)
        of ``fraction`` of the rows. Returns an :class:`~lawsynth.ensemble.Ensemble`
        reporting, per law term, its selection frequency and coefficient
        mean/std across members — so robust terms are separated from unstable
        ones. Resample indices are derived purely from ``seed`` (never the
        clock), so the whole ensemble reproduces exactly.
        """
        from .ensemble import build_ensemble

        base = _resolve_config(config, recipe, overrides)
        return build_ensemble(
            self._dataset, self._states, base,
            n=n, fraction=fraction, seed=seed, name=self._name,
        )

    # -- model monitoring / anomaly detection ------------------------------- #

    def monitor(self, new_dataset: Dataset, *, threshold: float = 3.0):
        """Score fresh observations against the discovered world.

        Simulates the world across ``new_dataset``, computes robust standardized
        residuals per state, and flags any timestamp whose residual exceeds
        ``threshold`` sigma. Returns a :class:`~lawsynth.monitor.MonitorReport`
        with per-state residual statistics, the flagged anomalies, and an
        in-control / drift verdict.
        """
        from .monitor import monitor as _monitor

        return _monitor(
            self._require_world(), new_dataset,
            state=self._states, threshold=threshold, name=self._name,
        )

    # -- streaming / online discovery --------------------------------------- #

    def stream(
        self,
        *,
        window: int = 60,
        step: int | None = None,
        threshold: float = 4.0,
        sustain: int = 2,
        config: DiscoveryConfig | None = None,
        growing: bool = False,
    ):
        """Process this study's series as a stream, maintaining models over it.

        Advances a window across the time column, keeps a current model, and
        re-discovers only on a *sustained* standardized-residual drift (a regime
        change) over ``sustain`` consecutive windows — distinct from a transient
        outlier. Every update emits an immutable change record (prior/new world
        revision hash, triggering window, and a per-law term/coefficient diff).
        Returns a :class:`~lawsynth.streaming.StreamHistory`. Deterministic and
        offline: replaying the identical series yields identical models and change
        records. Does not require a prior :meth:`discover` call — the first window
        seeds the model.
        """
        from .streaming import stream_discover

        return stream_discover(
            self._dataset,
            time="time",
            state=self._states,
            window=window,
            step=step,
            threshold=threshold,
            sustain=sustain,
            config=config,
            growing=growing,
            name=self._name,
        )

    # -- scenario boards ---------------------------------------------------- #

    def add_scenario(self, label: str, *, interventions: Mapping[str, float]) -> "Study":
        """Register a named what-if defined by initial-condition overrides.

        ``interventions`` maps state variables to the initial values used for
        that scenario; every other state starts from the observed baseline.
        Returns ``self`` so scenarios can be chained fluently. The baseline
        (no-intervention) run is always implicit and never needs registering.
        """
        if not label or not isinstance(label, str):
            raise ValidationError("scenario label must be a non-empty string")
        if label == "baseline":
            raise ValidationError("'baseline' is reserved for the no-intervention run")
        if not interventions:
            raise ValidationError(f"scenario {label!r} needs at least one intervention")
        unknown = [key for key in interventions if key not in self._states]
        if unknown:
            raise ValidationError(
                f"scenario {label!r} names unknown state variables {unknown}; valid states are {list(self._states)}"
            )
        overrides = {key: float(value) for key, value in interventions.items()}
        for key, value in overrides.items():
            if not isfinite(value):
                raise ValidationError(f"scenario {label!r} value for {key!r} must be finite")
        # Immutable update: rebuild the registry rather than mutating in place.
        self._scenarios = {**self._scenarios, label: overrides}
        return self

    @property
    def scenarios(self) -> dict[str, dict[str, float]]:
        """A copy of the registered scenario overrides, keyed by label."""
        return {label: dict(overrides) for label, overrides in self._scenarios.items()}

    def clear_scenarios(self) -> "Study":
        self._scenarios = {}
        return self

    def compare_scenarios(
        self,
        *,
        horizon: float | None = None,
        step: float | None = None,
        baseline_label: str = "baseline",
    ) -> ScenarioComparison:
        """Simulate the baseline and every registered scenario, then compare.

        Returns a :class:`ScenarioComparison` holding one trajectory per label on
        a shared time grid, with per-scenario final states and their divergence
        from the baseline. Requires at least one registered scenario.
        """
        if not self._scenarios:
            raise ValidationError("no scenarios registered; call add_scenario() first")
        return _compare_scenarios(
            self._require_world(), self._dataset, self._states, self._scenarios,
            baseline_label=baseline_label, horizon=horizon, step=step,
        )

    def dashboard(self, *, theme: str = "light", horizon: float | None = None):
        """Render a cohesive notebook dashboard for the discovered world.

        Requires the optional ``lawsynth-notebook`` package. Any registered
        scenarios are compared and folded into the dashboard automatically.
        """
        self._require_world()
        from lawsynth_notebook.dashboard import render_dashboard

        comparison = self.compare_scenarios(horizon=horizon) if self._scenarios else None
        return render_dashboard(self, theme=theme, comparison=comparison, horizon=horizon)

    def report(self, path: str | PathLike[str], *, theme: str = "light") -> Path:
        """Write a self-contained HTML report of the discovered world."""
        return _write_report(self._require_world(), self._dataset, self._states, path, name=self._name, theme=theme)

    def save(self, path: str | PathLike[str]) -> Path:
        """Persist the discovered world as a portable ``.lsworld`` bundle."""
        target = Path(path)
        self._require_world().save(str(target))
        return target

    @classmethod
    def load(cls, path: str | PathLike[str], *, dataset: Dataset, state: Sequence[str], name: str | None = None) -> "Study":
        """Load a persisted world and rebind it to its originating dataset."""
        from ._native import World
        world = World.load(str(Path(path)))
        study = cls(dataset, state, name=name or Path(path).stem)
        study._world = world
        return study

    def _repr_html_(self) -> str:
        if self._world is None:
            span = (float(self._dataset.time[0]), float(self._dataset.time[-1]))
            return (
                '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;padding:14px;margin:8px 0">'
                f'<h3 style="margin:0 0 8px;color:#155e75">Study — {self._name}</h3>'
                f"<p>{len(self._dataset.time)} samples over t ∈ [{span[0]:.4g}, {span[1]:.4g}]; "
                f"states: {', '.join(self._states)}.</p>"
                '<p style="color:#53627a">Call <code>discover()</code> to find its laws.</p></section>'
            )
        # Prefer the rich composed dashboard when the notebook package is
        # available; degrade to the compact self-contained report otherwise.
        try:
            return self.dashboard()._repr_html_()
        except Exception:  # notebook package absent or optional render failed
            return _report_html(self._world, self._dataset, self._states, name=self._name, theme="light")

    def __repr__(self) -> str:
        state = "discovered" if self._world is not None else "not discovered"
        return f"Study(name={self._name!r}, states={list(self._states)}, {state})"


# --------------------------------------------------------------------------- #
# Rich display wiring — teach native + SDK objects to render in Jupyter        #
# --------------------------------------------------------------------------- #

_RICH_DISPLAY_WIRED = False


def _native_world_repr_html(self: object) -> str:
    equations = dict(self.equations())
    laws = [_build_law(target, equations[target]) for target in sorted(equations)]
    laws_html = "".join(f"<li style='font-family:ui-monospace,monospace'>{law.readable}</li>" for law in laws)
    return (
        '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;padding:14px;margin:8px 0">'
        '<h3 style="margin:0 0 8px;color:#155e75">Executable World</h3>'
        + _report.equations_table_html(equations)
        + f'<ul style="margin:10px 0 0;padding-left:18px">{laws_html}</ul></section>'
    )


def _native_trajectory_repr_html(self: object) -> str:
    data = TrajectoryData.from_native(self)
    return _trajectory_data_repr_html(data)


def _trajectory_data_repr_html(self: TrajectoryData) -> str:
    chart = _report.svg_line_chart(self.time, dict(self.values), width=680, height=320, title="Trajectory")
    return (
        '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;padding:12px;margin:8px 0">'
        f'<h3 style="margin:0 0 6px;color:#155e75">Trajectory — {len(self.time)} samples</h3>{chart}</section>'
    )


def enable_rich_display() -> None:
    """Attach ``_repr_html_`` to the native and SDK trajectory/world types.

    Idempotent and best-effort: rich Jupyter output is a convenience, never a
    correctness requirement, so failures to patch are silently ignored.
    """
    global _RICH_DISPLAY_WIRED
    if _RICH_DISPLAY_WIRED:
        return
    try:
        TrajectoryData._repr_html_ = _trajectory_data_repr_html  # type: ignore[attr-defined]
    except (AttributeError, TypeError):  # pragma: no cover
        pass
    try:
        from . import _native
    except ImportError:  # pragma: no cover - native optional at import time
        _native = None  # type: ignore[assignment]
    if _native is not None:
        for cls_name, handler in (("World", _native_world_repr_html), ("Trajectory", _native_trajectory_repr_html)):
            try:
                setattr(getattr(_native, cls_name), "_repr_html_", handler)
            except (AttributeError, TypeError):  # pragma: no cover
                pass
    # Attach the simplification and editing methods to the native World so that
    # ``world.simplify()`` / ``world.rename(...)`` etc. work on discovered worlds.
    try:
        from . import composition, simplification  # noqa: F401
    except Exception:  # pragma: no cover - native optional at import time
        pass
    _RICH_DISPLAY_WIRED = True
