"""Ensemble (bootstrap) discovery for honest, term-level uncertainty.

Discovery on a single dataset yields one world with one set of coefficients — it
says nothing about how *stable* that structure is. :class:`Ensemble` answers the
question a scientist actually cares about: *which terms are robust, and which are
artifacts of this particular sample?*

``Study.discover_ensemble(n=..., fraction=..., seed=...)`` re-runs discovery on
``n`` deterministic bootstrap resamples of the observations. Each resample is an
``m``-of-``n`` draw *without* replacement (a fraction of the rows, kept in time
order so the time axis stays strictly increasing and valid for the native
engine). Because resample indices are derived purely from ``seed`` — never the
wall clock — the whole ensemble is bit-for-bit reproducible.

The result reports, per law term, its **selection frequency** (how often across
members the term survived thresholding) and its coefficient **mean/std**. A term
selected in every member with a tight coefficient spread is trustworthy; one
that flickers in and out, or whose sign wanders, is not. The ensemble also turns
member disagreement into a **forecast band**: lower/median/upper trajectories
built from simulating every member world forward.
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from random import Random
from typing import Mapping, Sequence

from . import report as _report
from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import NativeError, ValidationError
from .trajectory import TrajectoryData

__all__ = ["Ensemble", "ForecastBand", "TermStat", "build_ensemble"]

# A member subsample must keep at least this many rows for discovery to be
# meaningful; ``fraction`` is clamped upward to honour it.
_MIN_MEMBER_ROWS = 8


# --------------------------------------------------------------------------- #
# Deterministic numeric helpers                                               #
# --------------------------------------------------------------------------- #


def _mean(values: Sequence[float]) -> float:
    return sum(values) / len(values) if values else 0.0


def _std(values: Sequence[float]) -> float:
    """Population standard deviation (0 for <2 values)."""
    n = len(values)
    if n < 2:
        return 0.0
    mean = sum(values) / n
    return (sum((v - mean) ** 2 for v in values) / n) ** 0.5


def _percentile(sorted_values: Sequence[float], q: float) -> float:
    """Linear-interpolated percentile of an already-sorted sequence."""
    n = len(sorted_values)
    if n == 0:
        return float("nan")
    if n == 1:
        return float(sorted_values[0])
    pos = q * (n - 1)
    lo = int(pos)
    hi = min(lo + 1, n - 1)
    frac = pos - lo
    return sorted_values[lo] + frac * (sorted_values[hi] - sorted_values[lo])


# --------------------------------------------------------------------------- #
# Per-term stability statistics                                               #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class TermStat:
    """Cross-member statistics for one law term (``target`` <- ``feature``)."""

    target: str
    feature: str
    selection_frequency: float  # share of members where the term was selected
    mean: float                 # mean coefficient across members that selected it
    std: float                  # std of that coefficient across those members
    count: int                  # members that selected it
    members: int                # total ensemble members

    @property
    def relative_std(self) -> float:
        return self.std / abs(self.mean) if self.mean else float("inf")

    @property
    def robust(self) -> bool:
        """A term is robust when it is almost always selected with a tight spread."""
        return self.selection_frequency >= 0.8 and self.relative_std <= 0.25

    def to_dict(self) -> dict[str, object]:
        return {
            "target": self.target,
            "feature": self.feature,
            "selection_frequency": self.selection_frequency,
            "mean": self.mean,
            "std": self.std,
            "count": self.count,
            "members": self.members,
            "robust": self.robust,
        }


# --------------------------------------------------------------------------- #
# Forecast band                                                               #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class ForecastBand:
    """Lower/median/upper trajectories aggregated across ensemble members."""

    time: tuple[float, ...]
    lower: Mapping[str, tuple[float, ...]]
    median: Mapping[str, tuple[float, ...]]
    upper: Mapping[str, tuple[float, ...]]
    lower_q: float
    upper_q: float
    members: int

    @property
    def states(self) -> tuple[str, ...]:
        return tuple(self.median.keys())

    def to_dict(self) -> dict[str, object]:
        return {
            "time": list(self.time),
            "lower_q": self.lower_q,
            "upper_q": self.upper_q,
            "members": self.members,
            "lower": {k: list(v) for k, v in self.lower.items()},
            "median": {k: list(v) for k, v in self.median.items()},
            "upper": {k: list(v) for k, v in self.upper.items()},
        }

    def _repr_html_(self) -> str:
        charts = []
        for state in self.states:
            series = {
                f"{state} p{int(self.lower_q * 100)}": self.lower[state],
                f"{state} median": self.median[state],
                f"{state} p{int(self.upper_q * 100)}": self.upper[state],
            }
            charts.append(_report.svg_line_chart(
                self.time, series, width=720, height=300,
                title=f"{state}: ensemble forecast band ({self.members} members)",
                sort_series=False,
            ))
        return (
            '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;'
            'padding:14px;margin:8px 0">'
            '<h3 style="margin:0 0 8px;color:#155e75">Ensemble forecast band</h3>'
            + "".join(charts)
            + "</section>"
        )


# --------------------------------------------------------------------------- #
# Ensemble                                                                    #
# --------------------------------------------------------------------------- #


class Ensemble:
    """Bootstrap-discovery result: per-term stability plus a forecast band."""

    __slots__ = ("_name", "_states", "_dataset", "_config", "_members",
                 "_terms", "_n_requested", "_fraction", "_seed", "_member_rows")

    def __init__(
        self,
        *,
        name: str,
        states: Sequence[str],
        dataset: Dataset,
        config: DiscoveryConfig,
        members: Sequence[object],
        terms: Sequence[TermStat],
        n_requested: int,
        fraction: float,
        seed: int,
        member_rows: int,
    ) -> None:
        self._name = name
        self._states = tuple(states)
        self._dataset = dataset
        self._config = config
        self._members = tuple(members)
        self._terms = tuple(terms)
        self._n_requested = n_requested
        self._fraction = fraction
        self._seed = seed
        self._member_rows = member_rows

    # -- accessors ---------------------------------------------------------- #

    @property
    def name(self) -> str:
        return self._name

    @property
    def states(self) -> tuple[str, ...]:
        return self._states

    @property
    def members(self) -> int:
        """Number of members that discovered successfully."""
        return len(self._members)

    @property
    def terms(self) -> tuple[TermStat, ...]:
        """All observed terms, sorted by target then descending |mean|."""
        return self._terms

    def robust_terms(self) -> tuple[TermStat, ...]:
        return tuple(term for term in self._terms if term.robust)

    def consensus_laws(self, *, min_frequency: float = 0.5) -> dict[str, str]:
        """Readable per-target laws from terms selected in >= ``min_frequency`` of members."""
        laws: dict[str, list[TermStat]] = {}
        for term in self._terms:
            if term.selection_frequency >= min_frequency:
                laws.setdefault(term.target, []).append(term)
        readable: dict[str, str] = {}
        for target in sorted(laws):
            pieces = []
            for i, term in enumerate(sorted(laws[target], key=lambda t: -abs(t.mean))):
                magnitude = f"{abs(term.mean):.4g}"
                chunk = magnitude if term.feature == "1" else f"{magnitude}·{term.feature}"
                if i == 0:
                    pieces.append(f"-{chunk}" if term.mean < 0 else chunk)
                else:
                    pieces.append(f"{'-' if term.mean < 0 else '+'} {chunk}")
            readable[target] = f"d{target}/dt = {' '.join(pieces) or '0'}"
        return readable

    # -- forecast band ------------------------------------------------------ #

    def forecast(
        self,
        *,
        horizon: float | None = None,
        initial: Mapping[str, float] | None = None,
        step: float | None = None,
        lower_q: float = 0.1,
        upper_q: float = 0.9,
    ) -> ForecastBand:
        """Simulate every member forward and aggregate into a lower/median/upper band."""
        if not 0.0 <= lower_q < upper_q <= 1.0:
            raise ValidationError("require 0 <= lower_q < upper_q <= 1")
        from math import isfinite

        from .study import _default_step, _initial_state, _run_native

        start = float(self._dataset.time[0])
        span = float(horizon) if horizon is not None else float(self._dataset.time[-1] - self._dataset.time[0])
        if span <= 0:
            raise ValidationError("horizon must be positive")
        resolved_step = float(step) if step is not None else _default_step(self._dataset)
        resolved_initial = dict(initial) if initial is not None else _initial_state(self._dataset, self._states)

        # Simulate every member on the identical grid so samples align per index.
        per_state: dict[str, list[list[float]]] = {state: [] for state in self._states}
        grid: tuple[float, ...] | None = None
        used = 0
        for world in self._members:
            # A member with unstable spurious terms can diverge to a non-finite
            # value; skip it rather than let one blow-up poison the whole band.
            try:
                trajectory = _run_native(world, resolved_initial, start=start, end=start + span, step=resolved_step)
            except NativeError:
                continue
            columns = {state: trajectory.values.get(state) for state in self._states}
            if any(column is None or not all(isfinite(v) for v in column) for column in columns.values()):
                continue
            if grid is None:
                grid = trajectory.time
            elif len(trajectory.time) != len(grid):
                continue
            used += 1
            for state in self._states:
                per_state[state].append(list(columns[state]))
        if grid is None or used < 1:
            raise NativeError("no ensemble members produced a finite forecast")

        lower: dict[str, tuple[float, ...]] = {}
        median: dict[str, tuple[float, ...]] = {}
        upper: dict[str, tuple[float, ...]] = {}
        for state, runs in per_state.items():
            if not runs:
                continue
            lo_col, md_col, up_col = [], [], []
            for i in range(len(grid)):
                values = sorted(run[i] for run in runs)
                lo_col.append(_percentile(values, lower_q))
                md_col.append(_percentile(values, 0.5))
                up_col.append(_percentile(values, upper_q))
            lower[state] = tuple(lo_col)
            median[state] = tuple(md_col)
            upper[state] = tuple(up_col)
        return ForecastBand(grid, lower, median, upper, lower_q, upper_q, used)

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self._name,
            "states": list(self._states),
            "members": self.members,
            "members_requested": self._n_requested,
            "fraction": self._fraction,
            "member_rows": self._member_rows,
            "seed": self._seed,
            "terms": [term.to_dict() for term in self._terms],
            "consensus_laws": self.consensus_laws(),
        }

    def to_text(self) -> str:
        lines = [
            f"Ensemble uncertainty — {self._name}",
            f"  {self.members} members (requested {self._n_requested}) · "
            f"{self._member_rows}/{len(self._dataset.time)} rows each · "
            f"seed={self._seed} · fraction={self._fraction:g}",
            "",
            "Term stability (per law term across resamples):",
        ]
        header = f"  {'target':<8}{'term':<12}{'select%':>9}{'mean':>12}{'std':>12}{'':>10}"
        lines.append(header)
        lines.append("  " + "-" * (len(header) - 2))
        for term in self._terms:
            tag = "  robust" if term.robust else ("  unstable" if term.selection_frequency < 0.6 else "")
            lines.append(
                f"  {term.target:<8}{term.feature:<12}"
                f"{term.selection_frequency * 100:>8.0f}%"
                f"{term.mean:>12.4g}{term.std:>12.4g}{tag:>10}"
            )
        lines.append("")
        lines.append("Consensus laws (terms selected in >= 50% of members):")
        for target, readable in self.consensus_laws().items():
            lines.append(f"  {readable}")
        robust = self.robust_terms()
        lines.append("")
        lines.append(
            f"Robust terms: {len(robust)} of {len(self._terms)} observed "
            f"({', '.join(f'{t.target}<-{t.feature}' for t in robust) or 'none'})."
        )
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return (
            f"Ensemble(name={self._name!r}, members={self.members}, "
            f"terms={len(self._terms)}, robust={len(self.robust_terms())})"
        )

    # -- HTML view ---------------------------------------------------------- #

    def _repr_html_(self) -> str:
        head_cols = ["target", "term", "selection", "mean", "std", "stability"]
        head = "".join(
            f'<th style="padding:6px 10px;text-align:{"left" if i < 2 else "right"};'
            f'color:#53627a;border-bottom:1px solid #cbd5e1;font-size:11px;'
            f'letter-spacing:0.06em;text-transform:uppercase">{escape(h)}</th>'
            for i, h in enumerate(head_cols)
        )
        rows = []
        for term in self._terms:
            if term.robust:
                badge = '<span style="color:#2f6f4f;font-weight:600">robust</span>'
            elif term.selection_frequency < 0.6:
                badge = '<span style="color:#a3341f;font-weight:600">unstable</span>'
            else:
                badge = '<span style="color:#b8822a">borderline</span>'
            bar_w = int(round(term.selection_frequency * 60))
            sel = (
                f'<div style="display:inline-block;width:60px;height:8px;background:#e2e8f0;'
                f'border-radius:4px;vertical-align:middle;margin-right:6px">'
                f'<div style="width:{bar_w}px;height:8px;background:#155e75;border-radius:4px"></div></div>'
                f'{term.selection_frequency * 100:.0f}%'
            )
            rows.append(
                "<tr>"
                f'<td style="padding:6px 10px;font-weight:600">{escape(term.target)}</td>'
                f'<td style="padding:6px 10px;font-family:ui-monospace,monospace">{escape(term.feature)}</td>'
                f'<td style="padding:6px 10px;text-align:right;white-space:nowrap">{sel}</td>'
                f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{term.mean:.4g}</td>'
                f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{term.std:.4g}</td>'
                f'<td style="padding:6px 10px;text-align:right">{badge}</td>'
                "</tr>"
            )
        table = (
            '<table style="border-collapse:collapse;width:100%;color:#172033">'
            f"<thead><tr>{head}</tr></thead><tbody>{''.join(rows)}</tbody></table>"
        )
        try:
            band_html = self.forecast()._repr_html_()
        except Exception:  # forecast band is a convenience, never a hard requirement
            band_html = ""
        return (
            '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;'
            'padding:16px 18px;margin:8px 0;max-width:860px">'
            f'<h3 style="margin:0 0 4px;color:#155e75">Ensemble uncertainty — {escape(self._name)}</h3>'
            f'<p style="margin:0 0 10px;color:#53627a">{self.members} bootstrap members · '
            f'{self._member_rows}/{len(self._dataset.time)} rows each · seed {self._seed}. '
            'A term is <b>robust</b> when it survives in almost every resample with a tight '
            'coefficient spread.</p>'
            f"{table}{band_html}</section>"
        )


# --------------------------------------------------------------------------- #
# Construction                                                                #
# --------------------------------------------------------------------------- #


def _subsample(dataset: Dataset, indices: Sequence[int]) -> Dataset:
    time = tuple(dataset.time[i] for i in indices)
    columns = {name: tuple(values[i] for i in indices) for name, values in dataset.columns.items()}
    return Dataset(time, columns)


def build_ensemble(
    dataset: Dataset,
    states: Sequence[str],
    config: DiscoveryConfig,
    *,
    n: int,
    fraction: float,
    seed: int,
    name: str,
) -> Ensemble:
    """Run ``n`` seeded bootstrap discoveries and summarise term stability."""
    if n < 2:
        raise ValidationError("discover_ensemble requires n >= 2 members")
    if not 0.0 < fraction <= 1.0:
        raise ValidationError("fraction must be in (0, 1]")
    from .study import _discover_world, _extract_terms, _format_feature

    total = len(dataset.time)
    member_rows = max(_MIN_MEMBER_ROWS, round(fraction * total))
    member_rows = min(member_rows, total)
    if member_rows < 2:
        raise ValidationError("dataset too small for ensemble discovery")

    # Accumulate coefficients per (target, feature) across members.
    coeff_lists: dict[tuple[str, str], list[float]] = {}
    members: list[object] = []
    for k in range(n):
        # Member seed is a pure deterministic function of (seed, k) — no clock.
        rng = Random(seed * 1_000_003 + k)
        if member_rows >= total:
            indices = list(range(total))
        else:
            indices = sorted(rng.sample(range(total), member_rows))
        subset = _subsample(dataset, indices)
        try:
            world = _discover_world(subset, states, config)
        except NativeError:
            continue  # deterministic: same seed => same skipped members
        members.append(world)
        for target, expression in dict(world.equations()).items():
            for coeff, factors in _extract_terms(expression):
                feature = _format_feature(factors)
                coeff_lists.setdefault((target, feature), []).append(coeff)

    if len(members) < 2:
        raise NativeError(
            f"only {len(members)} of {n} ensemble members discovered successfully; "
            "cannot quantify uncertainty"
        )

    n_members = len(members)
    terms = [
        TermStat(
            target=target,
            feature=feature,
            selection_frequency=len(coeffs) / n_members,
            mean=_mean(coeffs),
            std=_std(coeffs),
            count=len(coeffs),
            members=n_members,
        )
        for (target, feature), coeffs in coeff_lists.items()
    ]
    terms.sort(key=lambda t: (t.target, -abs(t.mean)))

    return Ensemble(
        name=name,
        states=states,
        dataset=dataset,
        config=config,
        members=members,
        terms=terms,
        n_requested=n,
        fraction=fraction,
        seed=seed,
        member_rows=member_rows,
    )
