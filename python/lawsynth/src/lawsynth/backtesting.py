"""Rolling-origin (walk-forward) backtesting for discovered worlds.

Discovery tells you how well a world *fits* the window it was learned on; it says
nothing about how well that world *forecasts*. :class:`Backtest` answers the
question a forecaster actually cares about: *from a point inside the observed
series, how accurately does simulating the world forward predict what happened
next — and how fast does that skill decay with the forecast horizon?*

``Study.backtest(origins=5, horizon=H)`` performs a classic rolling-origin
evaluation. It selects ``origins`` evenly spaced forecast origins across the
observed series; from each origin it seeds the world with the observed state and
simulates forward ``H`` steps, then scores the predicted trajectory against the
actual observations at each lead. Errors are aggregated per state (RMSE / MAE /
R²) and collapsed into a **skill-vs-horizon** curve — the mean forecast error at
each lead ``h = 1..H`` — so the horizon at which the world stops being useful is
explicit. Everything is deterministic and offline: identical inputs reproduce an
identical backtest.
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from math import isfinite
from typing import Mapping, Sequence

from . import report as _report
from .dataset import Dataset
from .errors import NativeError, ValidationError

__all__ = ["Backtest", "OriginResult", "backtest"]


# --------------------------------------------------------------------------- #
# Small deterministic numeric helpers                                         #
# --------------------------------------------------------------------------- #


def _interp(times: Sequence[float], values: Sequence[float], target: float) -> float:
    """Linear interpolation of ``values`` sampled at ``times`` onto ``target``.

    ``times`` is strictly increasing (native trajectory grids always are). The
    endpoints are clamped, so a target just outside the grid returns the nearest
    endpoint rather than extrapolating wildly.
    """
    n = len(times)
    if n == 0:
        return float("nan")
    if target <= times[0]:
        return float(values[0])
    if target >= times[-1]:
        return float(values[-1])
    # Binary search for the bracketing interval.
    lo, hi = 0, n - 1
    while hi - lo > 1:
        mid = (lo + hi) // 2
        if times[mid] <= target:
            lo = mid
        else:
            hi = mid
    span = times[hi] - times[lo]
    if span <= 0:
        return float(values[lo])
    frac = (target - times[lo]) / span
    return float(values[lo]) + frac * (float(values[hi]) - float(values[lo]))


def _rmse(residuals: Sequence[float]) -> float:
    if not residuals:
        return float("nan")
    return (sum(r * r for r in residuals) / len(residuals)) ** 0.5


def _mae(residuals: Sequence[float]) -> float:
    if not residuals:
        return float("nan")
    return sum(abs(r) for r in residuals) / len(residuals)


def _r_squared(actuals: Sequence[float], residuals: Sequence[float]) -> float:
    if not actuals:
        return float("nan")
    mean = sum(actuals) / len(actuals)
    ss_tot = sum((a - mean) ** 2 for a in actuals) or 1e-12
    ss_res = sum(r * r for r in residuals)
    return 1.0 - ss_res / ss_tot


# --------------------------------------------------------------------------- #
# Per-origin result                                                           #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class OriginResult:
    """Forecast accuracy from a single rolling origin."""

    index: int                                  # observation index of the origin
    time: float                                 # observed time at the origin
    leads: int                                  # number of scored leads (<= horizon)
    rmse: Mapping[str, float]                    # per-state RMSE over this origin's leads
    mae: Mapping[str, float]                     # per-state MAE over this origin's leads

    def to_dict(self) -> dict[str, object]:
        return {
            "index": self.index,
            "time": self.time,
            "leads": self.leads,
            "rmse": dict(self.rmse),
            "mae": dict(self.mae),
        }


# --------------------------------------------------------------------------- #
# Backtest                                                                     #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class Backtest:
    """A rolling-origin forecast evaluation of a discovered world."""

    name: str
    states: tuple[str, ...]
    horizon: int
    origins: tuple[OriginResult, ...]
    leads: tuple[int, ...]                        # 1..H actually scored across origins
    rmse: Mapping[str, float]                     # per-state RMSE aggregated over all origins
    mae: Mapping[str, float]                      # per-state MAE aggregated over all origins
    r_squared: Mapping[str, float]                # per-state R² aggregated over all origins
    skill_by_lead: Mapping[str, tuple[float, ...]]  # per-state mean-abs-error at each lead
    skill_combined: tuple[float, ...]             # mean-abs-error at each lead across states

    # -- aggregate scores --------------------------------------------------- #

    @property
    def mean_r_squared(self) -> float:
        finite = [v for v in self.r_squared.values() if isfinite(v)]
        return sum(finite) / len(finite) if finite else float("nan")

    @property
    def verdict(self) -> str:
        """A plain-language read on out-of-sample forecasting skill."""
        score = self.mean_r_squared
        if not isfinite(score):
            return "inconclusive"
        if score >= 0.9:
            return "strong forecasting skill"
        if score >= 0.6:
            return "moderate forecasting skill"
        if score >= 0.2:
            return "weak forecasting skill"
        return "no forecasting skill"

    @property
    def decay(self) -> float:
        """Skill decay: growth in mean error from the first lead to the last.

        Reported as a multiplier (error at H / error at 1); ``1.0`` means the
        world forecasts as well far out as it does one step ahead.
        """
        curve = self.skill_combined
        if len(curve) < 2 or not isfinite(curve[0]) or curve[0] == 0.0:
            return float("nan")
        return curve[-1] / curve[0]

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "states": list(self.states),
            "horizon": self.horizon,
            "origins": [origin.to_dict() for origin in self.origins],
            "leads": list(self.leads),
            "rmse": dict(self.rmse),
            "mae": dict(self.mae),
            "r_squared": dict(self.r_squared),
            "skill_by_lead": {k: list(v) for k, v in self.skill_by_lead.items()},
            "skill_combined": list(self.skill_combined),
            "mean_r_squared": self.mean_r_squared,
            "decay": self.decay,
            "verdict": self.verdict,
        }

    def to_text(self) -> str:
        lines = [
            f"Backtest — {self.name}",
            f"  rolling-origin walk-forward · {len(self.origins)} origins · "
            f"horizon {self.horizon} step(s) · states: {', '.join(self.states)}",
            "",
            "Aggregate forecast accuracy (out-of-sample, across all origins):",
        ]
        header = f"  {'state':<10}{'RMSE':>12}{'MAE':>12}{'R²':>10}"
        lines.append(header)
        lines.append("  " + "-" * (len(header) - 2))
        for state in self.states:
            lines.append(
                f"  {state:<10}{self.rmse[state]:>12.4g}"
                f"{self.mae[state]:>12.4g}{self.r_squared[state]:>10.4f}"
            )
        lines.append("")
        lines.append("Skill vs. horizon (mean |error| across origins & states):")
        lead_header = "  " + "".join(f"h={h:<6}" for h in self.leads)
        lines.append(lead_header)
        lines.append("  " + "".join(f"{v:<8.4g}" for v in self.skill_combined))
        lines.append("")
        lines.append("Per-origin RMSE (first state shown; see to_dict for all):")
        first = self.states[0]
        for origin in self.origins:
            lines.append(
                f"  origin @ t={origin.time:.4g} (idx {origin.index}, {origin.leads} leads): "
                f"{first} RMSE = {origin.rmse[first]:.4g}"
            )
        lines.append("")
        decay = self.decay
        decay_text = f"{decay:.2f}x" if isfinite(decay) else "n/a"
        lines.append(
            f"Verdict: {self.verdict} (mean R² = {self.mean_r_squared:.4f}; "
            f"error grows {decay_text} from lead 1 to {self.horizon})."
        )
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return (
            f"Backtest(name={self.name!r}, origins={len(self.origins)}, "
            f"horizon={self.horizon}, mean_r2={self.mean_r_squared:.3f})"
        )

    # -- HTML view ---------------------------------------------------------- #

    def _skill_chart(self, *, theme: str = "light") -> str:
        leads = tuple(float(h) for h in self.leads)
        series: dict[str, Sequence[float]] = {
            state: self.skill_by_lead[state] for state in self.states
        }
        series["all states"] = self.skill_combined
        return _report.svg_line_chart(
            leads, series, width=720, height=320,
            title="Skill vs. horizon — mean |forecast error| by lead",
            theme=theme, x_label="forecast lead (steps ahead)", sort_series=False,
        )

    def _repr_html_(self, *, theme: str = "light") -> str:
        colors = _report._theme(theme)
        chart = self._skill_chart(theme=theme)

        # Aggregate per-state accuracy table.
        acc_head = "".join(
            f'<th style="padding:6px 10px;text-align:{"left" if i == 0 else "right"};'
            f'color:{colors["muted"]};border-bottom:1px solid {colors["border"]};'
            f'font-size:11px;letter-spacing:0.06em;text-transform:uppercase">{escape(h)}</th>'
            for i, h in enumerate(["state", "RMSE", "MAE", "R²"])
        )
        acc_rows = "".join(
            "<tr>"
            f'<td style="padding:6px 10px;font-weight:600">{escape(state)}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{self.rmse[state]:.4g}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{self.mae[state]:.4g}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{self.r_squared[state]:.4f}</td>'
            "</tr>"
            for state in self.states
        )
        acc_table = (
            f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
            f"<thead><tr>{acc_head}</tr></thead><tbody>{acc_rows}</tbody></table>"
        )

        # Per-origin table.
        origin_head = "".join(
            f'<th style="padding:6px 10px;text-align:{"left" if i == 0 else "right"};'
            f'color:{colors["muted"]};border-bottom:1px solid {colors["border"]};'
            f'font-size:11px;letter-spacing:0.06em;text-transform:uppercase">{escape(h)}</th>'
            for i, h in enumerate(["origin t", "index", "leads", *[f"{s} RMSE" for s in self.states]])
        )
        origin_rows = "".join(
            "<tr>"
            f'<td style="padding:6px 10px;font-family:ui-monospace,monospace">{origin.time:.4g}</td>'
            f'<td style="padding:6px 10px;text-align:right">{origin.index}</td>'
            f'<td style="padding:6px 10px;text-align:right">{origin.leads}</td>'
            + "".join(
                f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{origin.rmse[s]:.4g}</td>'
                for s in self.states
            )
            + "</tr>"
            for origin in self.origins
        )
        origin_table = (
            f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
            f"<thead><tr>{origin_head}</tr></thead><tbody>{origin_rows}</tbody></table>"
        )

        decay = self.decay
        decay_text = f"{decay:.2f}×" if isfinite(decay) else "n/a"
        return (
            f'<section style="font:14px system-ui;border:1px solid {colors["border"]};'
            f'border-radius:10px;padding:16px 18px;margin:8px 0;max-width:860px;'
            f'background:{colors["bg"]};color:{colors["fg"]}">'
            f'<h3 style="margin:0 0 4px;color:{colors["accent"]}">Backtest — {escape(self.name)}</h3>'
            f'<p style="margin:0 0 10px;color:{colors["muted"]}">Rolling-origin walk-forward · '
            f'{len(self.origins)} origins · horizon {self.horizon} step(s). '
            f'<b>{escape(self.verdict)}</b> — mean R² {self.mean_r_squared:.3f}; '
            f'error grows {decay_text} from lead 1 to {self.horizon}.</p>'
            f"{chart}"
            '<div style="margin-top:12px">'
            f'<b style="color:{colors["accent"]}">Out-of-sample accuracy</b>{acc_table}</div>'
            '<div style="margin-top:12px">'
            f'<b style="color:{colors["accent"]}">Per-origin RMSE</b>{origin_table}</div>'
            "</section>"
        )


# --------------------------------------------------------------------------- #
# Construction                                                                 #
# --------------------------------------------------------------------------- #


def _origin_indices(n: int, horizon: int, origins: int) -> list[int]:
    """Evenly spaced origin indices, each with ``horizon`` observations after it."""
    max_origin = n - 1 - horizon
    if max_origin < 0:
        raise ValidationError(
            f"series too short: need at least horizon+2 = {horizon + 2} samples "
            f"to score a horizon of {horizon}, have {n}"
        )
    count = min(origins, max_origin + 1)
    if count <= 1:
        return [0]
    # Evenly spaced integer indices spanning [0, max_origin], de-duplicated.
    picks = sorted({round(i * max_origin / (count - 1)) for i in range(count)})
    return picks


def backtest(
    world: object,
    dataset: Dataset,
    *,
    state: Sequence[str],
    origins: int = 5,
    horizon: int | None = None,
    step: float | None = None,
    name: str = "backtest",
) -> Backtest:
    """Rolling-origin (walk-forward) forecast evaluation of ``world``.

    From each of ``origins`` evenly spaced forecast origins in ``dataset``, seed
    ``world`` with the observed state and simulate forward ``horizon`` steps,
    scoring the predicted trajectory against the actual observations at each lead.

    ``horizon`` is measured in observation steps. When ``None`` it defaults to a
    window that leaves room for the requested number of origins. ``step`` is the
    integration step used per simulation; when ``None`` the median sampling
    interval is used, sub-sampled for interpolation accuracy.
    """
    from .study import _default_step, _run_native

    states = tuple(state)
    if not states:
        raise ValidationError("at least one state variable is required")
    missing = [s for s in states if s not in dataset.columns]
    if missing:
        raise ValidationError(f"state variables not present in dataset: {missing}")
    if origins < 1:
        raise ValidationError("origins must be >= 1")

    times = dataset.time
    n = len(times)
    if horizon is None:
        # Leave room for the requested origins while keeping a usable horizon.
        horizon = max(1, (n - 1) // (origins + 1))
    if horizon < 1:
        raise ValidationError("horizon must be >= 1 step")

    origin_indices = _origin_indices(n, horizon, origins)
    base_step = float(step) if step is not None else _default_step(dataset)
    if base_step <= 0:
        raise ValidationError("step must be positive")

    # Accumulate residuals per state (all origin/lead pairs) and per lead.
    residuals: dict[str, list[float]] = {s: [] for s in states}
    actuals: dict[str, list[float]] = {s: [] for s in states}
    lead_abs_err: dict[str, list[list[float]]] = {
        s: [[] for _ in range(horizon)] for s in states
    }

    origin_results: list[OriginResult] = []
    for index in origin_indices:
        origin_time = float(times[index])
        end_index = index + horizon
        end_time = float(times[end_index])
        initial = {s: float(dataset.columns[s][index]) for s in states}
        # Sub-sample the integration grid so interpolation onto the (possibly
        # irregular) observation timestamps stays accurate.
        span = end_time - origin_time
        sim_step = min(base_step, span / (horizon * 4)) if span > 0 else base_step
        if sim_step <= 0:
            sim_step = base_step
        try:
            trajectory = _run_native(
                world, initial, start=origin_time, end=end_time, step=sim_step
            )
        except NativeError:
            continue  # a diverging world at this origin contributes no scores

        per_origin_res: dict[str, list[float]] = {s: [] for s in states}
        for h in range(1, horizon + 1):
            target_time = float(times[index + h])
            for s in states:
                predicted = _interp(trajectory.time, trajectory.values.get(s, ()), target_time)
                actual = float(dataset.columns[s][index + h])
                if not (isfinite(predicted) and isfinite(actual)):
                    continue
                res = predicted - actual
                residuals[s].append(res)
                actuals[s].append(actual)
                per_origin_res[s].append(res)
                lead_abs_err[s][h - 1].append(abs(res))
        origin_results.append(OriginResult(
            index=index,
            time=origin_time,
            leads=max((len(v) for v in per_origin_res.values()), default=0),
            rmse={s: _rmse(per_origin_res[s]) for s in states},
            mae={s: _mae(per_origin_res[s]) for s in states},
        ))

    if not any(residuals[s] for s in states):
        raise NativeError("no origin produced a finite forecast; cannot backtest")

    rmse = {s: _rmse(residuals[s]) for s in states}
    mae = {s: _mae(residuals[s]) for s in states}
    r_squared = {s: _r_squared(actuals[s], residuals[s]) for s in states}

    # Skill-vs-horizon: mean |error| per lead, per state and combined.
    skill_by_lead: dict[str, tuple[float, ...]] = {}
    for s in states:
        skill_by_lead[s] = tuple(
            (sum(errs) / len(errs)) if errs else float("nan")
            for errs in lead_abs_err[s]
        )
    combined: list[float] = []
    for h in range(horizon):
        pooled = [e for s in states for e in lead_abs_err[s][h]]
        combined.append(sum(pooled) / len(pooled) if pooled else float("nan"))

    return Backtest(
        name=name,
        states=states,
        horizon=horizon,
        origins=tuple(origin_results),
        leads=tuple(range(1, horizon + 1)),
        rmse=rmse,
        mae=mae,
        r_squared=r_squared,
        skill_by_lead=skill_by_lead,
        skill_combined=tuple(combined),
    )
