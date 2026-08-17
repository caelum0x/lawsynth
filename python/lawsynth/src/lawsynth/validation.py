"""Out-of-sample **holdout** validation for discovered worlds.

In-window fit (see :class:`~lawsynth.study.Explanation`) tells you how well a
world reproduces the data it was *learned on* — it says nothing about whether the
discovery procedure generalizes. :func:`validate` answers the honest question:
*re-fit the model on a leading window, then score it on data it never saw.*

``Study.validate(holdout=0.25)`` splits the observed series in time: discovery is
re-run on the leading ``1 - holdout`` fraction, the discovered world is seeded
with the boundary observation, and simulated forward across the held-out tail.
The predicted trajectory is scored against the actual held-out observations
(RMSE / MAE / R² per state). Everything is deterministic and offline — identical
inputs reproduce an identical :class:`Validation`.
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from math import isfinite
from typing import Mapping, Sequence

from . import report as _report
from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import NativeError, ValidationError

__all__ = ["Validation", "validate"]

# A holdout split must leave at least this many rows on each side to be
# meaningful; the split point is clamped to honour it.
_MIN_TRAIN_ROWS = 8
_MIN_TEST_ROWS = 2


def _rmse(residuals: Sequence[float]) -> float:
    return (sum(r * r for r in residuals) / len(residuals)) ** 0.5 if residuals else float("nan")


def _mae(residuals: Sequence[float]) -> float:
    return sum(abs(r) for r in residuals) / len(residuals) if residuals else float("nan")


def _r_squared(actuals: Sequence[float], residuals: Sequence[float]) -> float:
    if not actuals:
        return float("nan")
    mean = sum(actuals) / len(actuals)
    ss_tot = sum((a - mean) ** 2 for a in actuals) or 1e-12
    ss_res = sum(r * r for r in residuals)
    return 1.0 - ss_res / ss_tot


@dataclass(frozen=True, slots=True)
class Validation:
    """Out-of-sample holdout accuracy for a re-fit discovered world."""

    name: str
    states: tuple[str, ...]
    holdout_fraction: float
    train_samples: int
    test_samples: int
    train_span: tuple[float, float]
    test_span: tuple[float, float]
    rmse: Mapping[str, float]
    mae: Mapping[str, float]
    r_squared: Mapping[str, float]

    @property
    def mean_r_squared(self) -> float:
        finite = [v for v in self.r_squared.values() if isfinite(v)]
        return sum(finite) / len(finite) if finite else float("nan")

    @property
    def verdict(self) -> str:
        """A plain-language read on out-of-sample generalization."""
        score = self.mean_r_squared
        if not isfinite(score):
            return "inconclusive"
        if score >= 0.9:
            return "strong generalization"
        if score >= 0.6:
            return "moderate generalization"
        if score >= 0.2:
            return "weak generalization"
        return "no generalization"

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "states": list(self.states),
            "holdout_fraction": self.holdout_fraction,
            "train_samples": self.train_samples,
            "test_samples": self.test_samples,
            "train_span": list(self.train_span),
            "test_span": list(self.test_span),
            "rmse": dict(self.rmse),
            "mae": dict(self.mae),
            "r_squared": dict(self.r_squared),
            "mean_r_squared": self.mean_r_squared,
            "verdict": self.verdict,
        }

    def to_text(self) -> str:
        lines = [
            f"Holdout validation — {self.name}",
            f"  train {self.train_samples} samples t ∈ "
            f"[{self.train_span[0]:.4g}, {self.train_span[1]:.4g}] · "
            f"test {self.test_samples} samples t ∈ "
            f"[{self.test_span[0]:.4g}, {self.test_span[1]:.4g}] "
            f"({self.holdout_fraction * 100:.0f}% holdout)",
            "",
            "Out-of-sample accuracy (scored on the held-out tail):",
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
        lines.append(f"Verdict: {self.verdict} (mean R² = {self.mean_r_squared:.4f}).")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return (
            f"Validation(name={self.name!r}, holdout={self.holdout_fraction:g}, "
            f"mean_r2={self.mean_r_squared:.3f})"
        )

    def _repr_html_(self, *, theme: str = "light") -> str:
        colors = _report._theme(theme)
        head = "".join(
            f'<th style="padding:6px 10px;text-align:{"left" if i == 0 else "right"};'
            f'color:{colors["muted"]};border-bottom:1px solid {colors["border"]};'
            f'font-size:11px;letter-spacing:0.06em;text-transform:uppercase">{escape(h)}</th>'
            for i, h in enumerate(["state", "RMSE", "MAE", "R²"])
        )
        rows = "".join(
            "<tr>"
            f'<td style="padding:6px 10px;font-weight:600">{escape(state)}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{self.rmse[state]:.4g}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{self.mae[state]:.4g}</td>'
            f'<td style="padding:6px 10px;text-align:right;font-family:ui-monospace,monospace">{self.r_squared[state]:.4f}</td>'
            "</tr>"
            for state in self.states
        )
        return (
            f'<section style="font:14px system-ui;border:1px solid {colors["border"]};'
            f'border-radius:10px;padding:16px 18px;margin:8px 0;max-width:860px;'
            f'background:{colors["bg"]};color:{colors["fg"]}">'
            f'<h3 style="margin:0 0 4px;color:{colors["accent"]}">Holdout validation — {escape(self.name)}</h3>'
            f'<p style="margin:0 0 10px;color:{colors["muted"]}">Re-fit on {self.train_samples} '
            f'leading sample(s), scored on the held-out final {self.test_samples} '
            f'({self.holdout_fraction * 100:.0f}% holdout). <b>{escape(self.verdict)}</b> — '
            f'mean R² {self.mean_r_squared:.3f}.</p>'
            f'<table style="border-collapse:collapse;width:100%;color:{colors["fg"]}">'
            f"<thead><tr>{head}</tr></thead><tbody>{rows}</tbody></table></section>"
        )


def validate(
    dataset: Dataset,
    states: Sequence[str],
    config: DiscoveryConfig,
    *,
    holdout: float = 0.25,
    step: float | None = None,
    name: str = "validation",
) -> Validation:
    """Holdout out-of-sample validation of the discovery procedure.

    Splits ``dataset`` in time, re-discovers a world on the leading
    ``1 - holdout`` fraction under ``config``, seeds it with the boundary
    observation, simulates across the held-out tail, and scores the forecast
    against the actual held-out observations. Deterministic and offline.
    """
    from .study import _default_step, _discover_world, _run_native
    from .prepare import interpolate

    states = tuple(states)
    if not states:
        raise ValidationError("at least one state variable is required")
    missing = [s for s in states if s not in dataset.columns]
    if missing:
        raise ValidationError(f"state variables not present in dataset: {missing}")
    if not 0.0 < holdout < 1.0:
        raise ValidationError("holdout must be in (0, 1)")

    time = dataset.time
    n = len(time)
    split = int(round(n * (1.0 - holdout)))
    # Clamp so both sides keep a workable number of rows.
    split = max(_MIN_TRAIN_ROWS, min(split, n - _MIN_TEST_ROWS))
    if split < _MIN_TRAIN_ROWS or n - split < _MIN_TEST_ROWS:
        raise ValidationError(
            f"series too short for a {holdout:g} holdout: need at least "
            f"{_MIN_TRAIN_ROWS + _MIN_TEST_ROWS} samples, have {n}"
        )

    train_indices = range(split)
    train = Dataset(
        tuple(time[i] for i in train_indices),
        {name_: tuple(values[i] for i in train_indices) for name_, values in dataset.columns.items()},
    )
    world = _discover_world(train, states, config)

    boundary = split - 1
    start = float(time[boundary])
    end = float(time[-1])
    resolved_step = float(step) if step is not None else _default_step(dataset)
    if resolved_step <= 0:
        raise ValidationError("step must be positive")
    initial = {s: float(dataset.columns[s][boundary]) for s in states}
    try:
        trajectory = _run_native(world, initial, start=start, end=end, step=resolved_step)
    except NativeError as error:
        raise NativeError(f"holdout forecast diverged; cannot validate: {error}") from error

    test_indices = range(split, n)
    test_times = tuple(float(time[i]) for i in test_indices)
    rmse: dict[str, float] = {}
    mae: dict[str, float] = {}
    r_squared: dict[str, float] = {}
    for s in states:
        predicted = interpolate(trajectory.time, trajectory.values.get(s, ()), test_times)
        actuals: list[float] = []
        residuals: list[float] = []
        for i, pred in zip(test_indices, predicted):
            actual = float(dataset.columns[s][i])
            if isfinite(pred) and isfinite(actual):
                actuals.append(actual)
                residuals.append(pred - actual)
        rmse[s] = _rmse(residuals)
        mae[s] = _mae(residuals)
        r_squared[s] = _r_squared(actuals, residuals)

    if all(not isfinite(v) for v in r_squared.values()):
        raise NativeError("no held-out sample produced a finite forecast; cannot validate")

    return Validation(
        name=name,
        states=states,
        holdout_fraction=float(holdout),
        train_samples=split,
        test_samples=n - split,
        train_span=(float(time[0]), float(time[boundary])),
        test_span=(test_times[0], test_times[-1]),
        rmse=rmse,
        mae=mae,
        r_squared=r_squared,
    )
