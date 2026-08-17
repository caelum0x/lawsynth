"""Model monitoring and anomaly detection for discovered worlds.

Once a world is discovered it becomes a *model of normal behaviour*. ``monitor``
scores a stream of fresh observations against that model: it simulates the world
across the new data, forms the residual (observed - simulated) per state, and
standardizes it with a **robust** scale (median / MAD) so that a genuine shock
stands out instead of inflating the very statistic meant to catch it. Any
timestamp whose standardized residual exceeds ``threshold`` sigma is flagged.

The result is a :class:`MonitorReport`: per-state residual statistics, the list
of flagged anomalies (with the offending time, state and z-score), and an
overall *in-control* vs *drift* verdict — rendered as text or self-contained
HTML. Clean data that matches the model reports in-control; shock-injected data
flags the anomaly at the timestamp it was injected.

Everything is deterministic and offline: the same world and data always produce
the same report.
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from statistics import median
from typing import Mapping, Sequence

from . import report as _report
from .dataset import Dataset
from .errors import ValidationError
from .prepare import interpolate

__all__ = ["monitor", "MonitorReport", "StateResidual", "Anomaly"]

# Scale factor making the MAD a consistent estimator of the standard deviation
# for normally distributed residuals.
_MAD_TO_SIGMA = 1.4826


# --------------------------------------------------------------------------- #
# Result types                                                                #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class StateResidual:
    """Residual summary for a single state over the monitored window."""

    state: str
    n: int
    mean: float
    std: float
    scale: float          # robust (MAD-based) sigma used for standardizing
    max_abs_z: float
    n_flagged: int

    def to_dict(self) -> dict[str, object]:
        return {
            "state": self.state,
            "n": self.n,
            "mean": self.mean,
            "std": self.std,
            "scale": self.scale,
            "max_abs_z": self.max_abs_z,
            "n_flagged": self.n_flagged,
        }


@dataclass(frozen=True, slots=True)
class Anomaly:
    """A single flagged observation exceeding the threshold."""

    time: float
    state: str
    z: float
    observed: float
    simulated: float

    def to_dict(self) -> dict[str, object]:
        return {
            "time": self.time,
            "state": self.state,
            "z": self.z,
            "observed": self.observed,
            "simulated": self.simulated,
        }


@dataclass(frozen=True, slots=True)
class MonitorReport:
    """Residual diagnostics and an in-control / drift verdict for fresh data."""

    name: str
    threshold: float
    states: tuple[str, ...]
    time: tuple[float, ...]
    residuals: tuple[StateResidual, ...]
    z_series: Mapping[str, tuple[float, ...]]
    anomalies: tuple[Anomaly, ...]
    in_control: bool

    @property
    def verdict(self) -> str:
        if self.in_control:
            return "in control"
        n = len(self.anomalies)
        return f"out of control — {n} anomal{'y' if n == 1 else 'ies'} flagged"

    def flagged_times(self) -> tuple[float, ...]:
        """The distinct timestamps at which at least one state was flagged."""
        return tuple(sorted({anomaly.time for anomaly in self.anomalies}))

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "threshold": self.threshold,
            "in_control": self.in_control,
            "verdict": self.verdict,
            "states": list(self.states),
            "residuals": [residual.to_dict() for residual in self.residuals],
            "anomalies": [anomaly.to_dict() for anomaly in self.anomalies],
            "flagged_times": list(self.flagged_times()),
        }

    # -- text view ---------------------------------------------------------- #

    def to_text(self) -> str:
        status = "IN CONTROL" if self.in_control else "OUT OF CONTROL"
        lines = [
            f"Monitor report — {self.name}",
            f"  verdict: {status} (threshold = {self.threshold:g} sigma, "
            f"{len(self.time)} samples)",
            "",
            "Per-state residuals:",
        ]
        header = f"  {'state':<8}{'n':>6}{'mean':>12}{'std':>12}{'robust σ':>12}{'max|z|':>10}{'flagged':>9}"
        lines.append(header)
        lines.append("  " + "-" * (len(header) - 2))
        for residual in self.residuals:
            lines.append(
                f"  {residual.state:<8}{residual.n:>6}{residual.mean:>12.4g}"
                f"{residual.std:>12.4g}{residual.scale:>12.4g}"
                f"{residual.max_abs_z:>10.2f}{residual.n_flagged:>9}"
            )
        lines.append("")
        if self.anomalies:
            lines.append(f"Anomalies flagged ({len(self.anomalies)}):")
            for anomaly in self.anomalies:
                lines.append(
                    f"  t={anomaly.time:.4g}  {anomaly.state}: observed={anomaly.observed:.4g} "
                    f"vs simulated={anomaly.simulated:.4g}  (z={anomaly.z:+.2f})"
                )
        else:
            lines.append("No anomalies — observations track the model within threshold.")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return (
            f"MonitorReport(name={self.name!r}, in_control={self.in_control}, "
            f"anomalies={len(self.anomalies)}, threshold={self.threshold})"
        )

    # -- HTML view ---------------------------------------------------------- #

    def _repr_html_(self) -> str:
        color = "#2f6f4f" if self.in_control else "#a3341f"
        label = "IN CONTROL" if self.in_control else "OUT OF CONTROL"
        # Standardized-residual chart with the +/- threshold envelope drawn as
        # flat reference series so the exceedance is visible at a glance.
        series: dict[str, tuple[float, ...]] = {state: self.z_series[state] for state in self.states}
        series[f"+{self.threshold:g}σ"] = tuple(self.threshold for _ in self.time)
        series[f"-{self.threshold:g}σ"] = tuple(-self.threshold for _ in self.time)
        chart = _report.svg_line_chart(
            self.time, series, width=720, height=320,
            title="Standardized residuals (z) vs. threshold", sort_series=False,
        )
        res_rows = "".join(
            f'<tr><td style="padding:4px 10px;font-weight:600">{escape(residual.state)}</td>'
            f'<td style="padding:4px 10px;text-align:right;font-family:ui-monospace,monospace">{residual.mean:.4g}</td>'
            f'<td style="padding:4px 10px;text-align:right;font-family:ui-monospace,monospace">{residual.scale:.4g}</td>'
            f'<td style="padding:4px 10px;text-align:right;font-family:ui-monospace,monospace">{residual.max_abs_z:.2f}</td>'
            f'<td style="padding:4px 10px;text-align:right">{residual.n_flagged}</td></tr>'
            for residual in self.residuals
        )
        res_table = (
            '<table style="border-collapse:collapse;width:100%;color:#172033">'
            '<thead><tr>'
            + "".join(
                f'<th style="padding:4px 10px;text-align:{"left" if i == 0 else "right"};'
                f'color:#53627a;font-size:11px;text-transform:uppercase">{escape(h)}</th>'
                for i, h in enumerate(["state", "mean resid", "robust σ", "max|z|", "flagged"])
            )
            + f"</tr></thead><tbody>{res_rows}</tbody></table>"
        )
        if self.anomalies:
            items = "".join(
                f'<li style="margin:2px 0">t=<b>{anomaly.time:.4g}</b> · {escape(anomaly.state)}: '
                f'observed {anomaly.observed:.4g} vs simulated {anomaly.simulated:.4g} '
                f'(z={anomaly.z:+.2f})</li>'
                for anomaly in self.anomalies
            )
            anomaly_html = (
                '<div style="margin-top:12px;padding:10px 12px;border-radius:6px;'
                'background:#f6dccf;border-left:3px solid #a3341f">'
                f'<b style="color:#a3341f">{len(self.anomalies)} anomaly flag(s)</b>'
                f'<ul style="margin:6px 0 0;padding-left:18px">{items}</ul></div>'
            )
        else:
            anomaly_html = (
                '<p style="margin-top:12px;color:#2f6f4f;font-weight:600">'
                "No anomalies — observations track the model within threshold.</p>"
            )
        return (
            '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;'
            'padding:16px 18px;margin:8px 0;max-width:860px">'
            f'<h3 style="margin:0 0 4px;color:#155e75">Monitor — {escape(self.name)}</h3>'
            f'<p style="margin:0 0 10px"><span style="color:{color};font-weight:700">{label}</span> '
            f'<span style="color:#53627a">· threshold {self.threshold:g}σ · {len(self.time)} samples</span></p>'
            f"{chart}"
            '<div style="margin-top:10px"><b style="color:#155e75">Per-state residuals</b>'
            f"{res_table}</div>{anomaly_html}</section>"
        )


# --------------------------------------------------------------------------- #
# Core computation                                                            #
# --------------------------------------------------------------------------- #


def _robust_scale(residuals: Sequence[float], center: float) -> float:
    """MAD-based robust sigma; falls back to population std, then to 0."""
    mad = median([abs(r - center) for r in residuals]) if residuals else 0.0
    if mad > 0:
        return _MAD_TO_SIGMA * mad
    # Degenerate spread (e.g. residuals all equal): fall back to std.
    n = len(residuals)
    if n < 2:
        return 0.0
    mean = sum(residuals) / n
    return (sum((r - mean) ** 2 for r in residuals) / n) ** 0.5


def monitor(
    world: object,
    dataset: Dataset,
    *,
    state: Sequence[str],
    threshold: float = 3.0,
    step: float | None = None,
    name: str = "monitor",
) -> MonitorReport:
    """Score ``dataset`` against ``world`` and flag anomalies beyond ``threshold`` sigma.

    Simulates ``world`` across the new dataset (from its first observation), then
    interpolates the simulation onto the observed timestamps so residuals align
    even on an irregular grid. Residuals are standardized with a robust
    median/MAD scale per state. Returns a :class:`MonitorReport`.
    """
    from .study import _default_step, _initial_state, _run_native

    states = tuple(state)
    if threshold <= 0:
        raise ValidationError("threshold must be positive")
    missing = [s for s in states if s not in dataset.columns]
    if missing:
        raise ValidationError(f"states {missing} not present in dataset")

    times = tuple(dataset.time)
    start = float(times[0])
    end = float(times[-1])
    resolved_step = float(step) if step is not None else _default_step(dataset)
    simulated = _run_native(world, _initial_state(dataset, states), start=start, end=end, step=resolved_step)

    residual_summaries: list[StateResidual] = []
    z_series: dict[str, tuple[float, ...]] = {}
    anomalies: list[Anomaly] = []
    for s in states:
        observed = dataset.columns[s]
        sim_on_grid = interpolate(simulated.time, simulated.values[s], times)
        residuals = [obs - sim for obs, sim in zip(observed, sim_on_grid)]
        center = median(residuals)
        scale = _robust_scale(residuals, center)
        if scale > 0:
            zs = tuple((r - center) / scale for r in residuals)
        else:
            zs = tuple(0.0 for _ in residuals)
        z_series[s] = zs
        n = len(residuals)
        mean = sum(residuals) / n
        std = (sum((r - mean) ** 2 for r in residuals) / n) ** 0.5 if n > 1 else 0.0
        flagged = 0
        for t, obs, sim, z in zip(times, observed, sim_on_grid, zs):
            if abs(z) > threshold:
                flagged += 1
                anomalies.append(Anomaly(time=float(t), state=s, z=float(z), observed=float(obs), simulated=float(sim)))
        residual_summaries.append(StateResidual(
            state=s, n=n, mean=mean, std=std, scale=scale,
            max_abs_z=max((abs(z) for z in zs), default=0.0), n_flagged=flagged,
        ))

    anomalies.sort(key=lambda a: (a.time, a.state))
    return MonitorReport(
        name=name,
        threshold=float(threshold),
        states=states,
        time=times,
        residuals=tuple(residual_summaries),
        z_series=z_series,
        anomalies=tuple(anomalies),
        in_control=not anomalies,
    )
