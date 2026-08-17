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
from math import isfinite
from os import PathLike
from pathlib import Path
from statistics import median
from typing import Mapping, Sequence

from . import report as _report
from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import LawSynthError, NativeError, ValidationError
from .trajectory import TrajectoryData

__all__ = ["Study", "DiscoveryResult", "Explanation", "Law", "Forecast", "enable_rich_display"]


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
# Shared operations backing both Study and DiscoveryResult                     #
# --------------------------------------------------------------------------- #


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

    def simulate(self, *, horizon: float | None = None, initial: Mapping[str, float] | None = None, start: float | None = None, step: float | None = None) -> TrajectoryData:
        return _simulate(self._world, self._dataset, self._states, horizon=horizon, initial=initial, start=start, step=step)

    def forecast(self, interventions: Mapping[str, float], *, horizon: float | None = None, step: float | None = None) -> Forecast:
        return _forecast(self._world, self._dataset, self._states, interventions=interventions, horizon=horizon, step=step)

    def report(self, path: str | PathLike[str], *, theme: str = "light") -> Path:
        return _write_report(self._world, self._dataset, self._states, path, name=self._name, theme=theme)

    def save(self, path: str | PathLike[str]) -> Path:
        target = Path(path)
        self._world.save(str(target))
        return target

    def _repr_html_(self) -> str:
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

    __slots__ = ("_dataset", "_states", "_name", "_world", "_config")

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

    # -- workflow ----------------------------------------------------------- #

    @property
    def dataset(self) -> Dataset:
        return self._dataset

    @property
    def states(self) -> tuple[str, ...]:
        return self._states

    @property
    def world(self) -> object:
        self._require_world()
        return self._world

    def _require_world(self) -> object:
        if self._world is None:
            raise LawSynthError("call discover() before using the world")
        return self._world

    def discover(self, config: DiscoveryConfig | None = None, **overrides: object) -> DiscoveryResult:
        """Discover an executable world from the study's observations."""
        base = config or DiscoveryConfig()
        if overrides:
            # A slots dataclass has no __dict__; rebuild from its fields explicitly.
            merged = {
                "polynomial_degree": base.polynomial_degree,
                "threshold": base.threshold,
                "solver": base.solver,
                "derivative_method": base.derivative_method,
                "include_trigonometric": base.include_trigonometric,
                "include_rational": base.include_rational,
                "smoothing_radius": base.smoothing_radius,
                "symbolic_depth": base.symbolic_depth,
            }
            unknown = set(overrides) - set(merged)
            if unknown:
                raise ValidationError(f"unknown discovery options: {sorted(unknown)}")
            merged.update(overrides)
            base = DiscoveryConfig(**merged)
        enable_rich_display()
        time, columns = self._dataset.as_native_arguments()
        try:
            from ._native import discover_world
        except ImportError as error:
            raise NativeError("the lawsynth native extension is unavailable; build it first") from error
        try:
            world = discover_world(
                time, columns, list(self._states),
                polynomial_degree=base.polynomial_degree,
                threshold=base.threshold,
                solver=base.solver,
                include_trigonometric=base.include_trigonometric,
                include_rational=base.include_rational,
                smoothing_radius=base.smoothing_radius,
                derivative_method=base.derivative_method,
                savgol_window=5,
                tvreg_lambda=0.1,
                tvreg_iterations=100,
                symbolic_depth=base.symbolic_depth,
            )
        except Exception as error:
            raise NativeError(f"discovery failed: {error}") from error
        self._world = world
        self._config = base
        return DiscoveryResult(world, self._dataset, self._states, base, self._name)

    def explain(self) -> Explanation:
        return _explain(self._require_world(), self._dataset, self._states, name=self._name)

    def simulate(self, *, horizon: float | None = None, initial: Mapping[str, float] | None = None, start: float | None = None, step: float | None = None) -> TrajectoryData:
        return _simulate(self._require_world(), self._dataset, self._states, horizon=horizon, initial=initial, start=start, step=step)

    def forecast(self, interventions: Mapping[str, float], *, horizon: float | None = None, step: float | None = None) -> Forecast:
        """Run a what-if: override initial conditions and compare to baseline."""
        return _forecast(self._require_world(), self._dataset, self._states, interventions=interventions, horizon=horizon, step=step)

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
    _RICH_DISPLAY_WIRED = True
