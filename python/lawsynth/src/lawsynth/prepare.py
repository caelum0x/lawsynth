"""Deterministic, offline data preparation for LawSynth datasets.

``preprocess(dataset, ...)`` returns a *new* cleaned :class:`~lawsynth.dataset.Dataset`
by composing four pure-Python, standard-library operations over the dataset's
arrays:

* **trim** — keep only samples inside a ``(t_start, t_end)`` window;
* **resample** — re-grid onto a uniform ``dt`` by linear interpolation on the
  time axis;
* **smooth** — centered moving-average with a sample ``window`` (edges shrink
  the window rather than pad);
* **detrend** — subtract a per-column least-squares linear trend.

Operations are applied in that order and never mutate the input dataset — each
step builds fresh tuples, in keeping with LawSynth's immutable data model. All
results are deterministic: identical inputs reproduce the same cleaned dataset.

Cleaning is the front half of the discovery loop: noisy or unevenly sampled
observations degrade the finite-difference derivatives discovery relies on, so
``study.prepare(smooth=...)`` before ``discover()`` typically recovers a system
with a materially better fit than discovery on the raw series.
"""

from __future__ import annotations

from bisect import bisect_right
from typing import Mapping, Sequence

from .dataset import Dataset
from .errors import ValidationError

__all__ = ["preprocess", "interpolate"]


# --------------------------------------------------------------------------- #
# Shared linear interpolation on a strictly increasing time grid              #
# --------------------------------------------------------------------------- #


def interpolate(
    src_time: Sequence[float],
    src_values: Sequence[float],
    target_time: Sequence[float],
) -> tuple[float, ...]:
    """Linearly interpolate ``src_values`` (sampled at ``src_time``) onto ``target_time``.

    ``src_time`` must be strictly increasing. Targets outside the source range
    clamp to the nearest endpoint (no extrapolation). Deterministic.
    """
    n = len(src_time)
    if n == 0 or len(src_values) != n:
        raise ValidationError("interpolate: source time and values must align and be non-empty")
    times = list(src_time)
    result: list[float] = []
    for t in target_time:
        if t <= times[0]:
            result.append(float(src_values[0]))
            continue
        if t >= times[-1]:
            result.append(float(src_values[-1]))
            continue
        # times[j-1] < t <= times[j]
        j = bisect_right(times, t)
        t0, t1 = times[j - 1], times[j]
        v0, v1 = float(src_values[j - 1]), float(src_values[j])
        span = t1 - t0
        weight = 0.0 if span == 0 else (t - t0) / span
        result.append(v0 + weight * (v1 - v0))
    return tuple(result)


# --------------------------------------------------------------------------- #
# Individual preparation operations (each returns fresh arrays)               #
# --------------------------------------------------------------------------- #


def _trim(
    time: Sequence[float],
    columns: Mapping[str, Sequence[float]],
    window: tuple[float, float],
) -> tuple[tuple[float, ...], dict[str, tuple[float, ...]]]:
    start, end = float(window[0]), float(window[1])
    if end <= start:
        raise ValidationError("trim window must satisfy t_start < t_end")
    keep = [i for i, t in enumerate(time) if start <= t <= end]
    if len(keep) < 2:
        raise ValidationError(
            f"trim window [{start:g}, {end:g}] retains {len(keep)} sample(s); need at least 2"
        )
    new_time = tuple(time[i] for i in keep)
    new_columns = {name: tuple(values[i] for i in keep) for name, values in columns.items()}
    return new_time, new_columns


def _resample(
    time: Sequence[float],
    columns: Mapping[str, Sequence[float]],
    dt: float,
) -> tuple[tuple[float, ...], dict[str, tuple[float, ...]]]:
    dt = float(dt)
    if dt <= 0:
        raise ValidationError("resample_dt must be positive")
    t0, t_end = float(time[0]), float(time[-1])
    span = t_end - t0
    steps = int(span / dt + 1e-9)
    if steps < 1:
        raise ValidationError(
            f"resample_dt={dt:g} is larger than the time span {span:g}; produces <2 samples"
        )
    grid = tuple(t0 + k * dt for k in range(steps + 1))
    new_columns = {name: interpolate(time, values, grid) for name, values in columns.items()}
    return grid, new_columns


def _smooth(
    columns: Mapping[str, Sequence[float]],
    window: int,
    selected: Sequence[str],
) -> dict[str, tuple[float, ...]]:
    window = int(window)
    if window < 1:
        raise ValidationError("smooth window must be a positive integer")
    radius = window // 2
    target = set(selected)
    smoothed: dict[str, tuple[float, ...]] = {}
    for name, values in columns.items():
        if radius == 0 or name not in target:
            smoothed[name] = tuple(float(v) for v in values)
            continue
        n = len(values)
        out: list[float] = []
        for i in range(n):
            lo = max(0, i - radius)
            hi = min(n, i + radius + 1)
            window_slice = values[lo:hi]
            out.append(sum(window_slice) / (hi - lo))
        smoothed[name] = tuple(out)
    return smoothed


def _detrend(
    time: Sequence[float],
    columns: Mapping[str, Sequence[float]],
    selected: Sequence[str],
) -> dict[str, tuple[float, ...]]:
    target = set(selected)
    n = len(time)
    t_mean = sum(time) / n
    t_var = sum((t - t_mean) ** 2 for t in time)
    detrended: dict[str, tuple[float, ...]] = {}
    for name, values in columns.items():
        if name not in target or t_var == 0:
            detrended[name] = tuple(float(v) for v in values)
            continue
        y_mean = sum(values) / n
        cov = sum((t - t_mean) * (v - y_mean) for t, v in zip(time, values))
        slope = cov / t_var
        intercept = y_mean - slope * t_mean
        detrended[name] = tuple(v - (intercept + slope * t) for t, v in zip(time, values))
    return detrended


# --------------------------------------------------------------------------- #
# Public pipeline                                                             #
# --------------------------------------------------------------------------- #


def preprocess(
    dataset: Dataset,
    *,
    trim: tuple[float, float] | None = None,
    resample_dt: float | None = None,
    smooth: int | None = None,
    detrend: bool | Sequence[str] = False,
    columns: Sequence[str] | None = None,
) -> Dataset:
    """Return a new cleaned :class:`~lawsynth.dataset.Dataset`.

    Applies (in order) ``trim`` -> ``resample_dt`` -> ``smooth`` -> ``detrend``.
    ``columns`` restricts *smoothing and detrending* to the named columns
    (default: all); trimming and resampling always apply to the whole dataset so
    the time axis stays consistent. ``detrend`` may be ``True`` (all selected
    columns) or an explicit sequence of column names. The input dataset is never
    mutated.
    """
    time: tuple[float, ...] = tuple(dataset.time)
    cols: dict[str, tuple[float, ...]] = {name: tuple(values) for name, values in dataset.columns.items()}

    if columns is not None:
        unknown = [name for name in columns if name not in cols]
        if unknown:
            raise ValidationError(f"prepare: unknown columns {unknown}; have {sorted(cols)}")
        selected: Sequence[str] = list(columns)
    else:
        selected = list(cols)

    if trim is not None:
        time, cols = _trim(time, cols, trim)
    if resample_dt is not None:
        time, cols = _resample(time, cols, resample_dt)
    if smooth is not None:
        cols = _smooth(cols, smooth, selected)
    if detrend:
        detrend_cols = selected if detrend is True else list(detrend)
        unknown = [name for name in detrend_cols if name not in cols]
        if unknown:
            raise ValidationError(f"detrend: unknown columns {unknown}; have {sorted(cols)}")
        cols = _detrend(time, cols, detrend_cols)

    return Dataset(time, cols)
