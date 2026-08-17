"""Deterministic, offline data profiling for LawSynth datasets.

``profile(dataset_or_csv, *, time=..., state=...)`` inspects a time series before
discovery and returns a structured :class:`DataProfile`: per-column summary
statistics (count, missing, min, max, mean, std), the dataset's row count, the
time axis's monotonicity and sampling regularity, degenerate/constant columns,
and a list of plain-language quality warnings. It is pure standard library and
renders richly in a notebook via ``_repr_html_`` using the LawSynth brand
palette.

The input may be an already-validated :class:`~lawsynth.dataset.Dataset` or a CSV
(a path, or inline CSV text). Unlike ``Dataset`` — which rejects any missing or
non-finite value — profiling reads a CSV leniently so it can *report* data
quality problems (missing cells, non-numeric values, irregular sampling) that
would otherwise block ingestion.
"""

from __future__ import annotations

import csv
from dataclasses import dataclass
from html import escape
from math import isfinite, sqrt
from os import PathLike
from pathlib import Path
from typing import Mapping, Sequence

from .dataset import Dataset
from .errors import ValidationError

__all__ = ["profile", "DataProfile", "ColumnProfile", "TimeProfile"]


# --------------------------------------------------------------------------- #
# Brand palette (assets/brand/palette.json) + typography stacks.              #
# --------------------------------------------------------------------------- #

_BRAND = {
    "ink": "#18201d",
    "paper": "#f3f0e8",
    "surface": "#fffdf7",
    "line": "#c8c6ba",
    "muted": "#59635e",
    "accent": "#b54b2a",
    "accent_soft": "#e5c3b4",
    "success": "#2f6f4f",
    "warning": "#b8822a",
    "danger": "#a3341f",
}
_SERIF = 'Georgia, "Times New Roman", serif'
_SANS = "Inter, system-ui, sans-serif"
_MONO = 'ui-monospace, SFMono-Regular, "SF Mono", monospace'

# A time axis is "regular" when the coefficient of variation of its per-step
# spacing stays under this tolerance (i.e. steps are effectively uniform).
_REGULARITY_TOLERANCE = 1e-3
# Columns whose spread is this small relative to their level read as constant.
_DEGENERATE_TOLERANCE = 1e-12


def _fmt(value: float) -> str:
    """Format a float compactly and deterministically for display."""
    if value != value:  # NaN
        return "—"
    if not isfinite(value):
        return "∞" if value > 0 else "-∞"
    return f"{value:.4g}"


# --------------------------------------------------------------------------- #
# Per-column summary statistics                                                #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class ColumnProfile:
    """Summary statistics for a single numeric column."""

    name: str
    count: int
    missing: int
    minimum: float
    maximum: float
    mean: float
    std: float
    is_constant: bool

    @property
    def spread(self) -> float:
        """The column's full range (``maximum - minimum``)."""
        if self.count == 0:
            return float("nan")
        return self.maximum - self.minimum

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "count": self.count,
            "missing": self.missing,
            "min": self.minimum,
            "max": self.maximum,
            "mean": self.mean,
            "std": self.std,
            "is_constant": self.is_constant,
        }


def _profile_column(name: str, raw: Sequence[float | None]) -> ColumnProfile:
    """Summarise one column of optional values (``None`` marks a missing cell)."""
    present = [float(v) for v in raw if v is not None and isfinite(v)]
    missing = len(raw) - len(present)
    if not present:
        return ColumnProfile(name, 0, missing, float("nan"), float("nan"), float("nan"), float("nan"), False)
    count = len(present)
    minimum = min(present)
    maximum = max(present)
    mean = sum(present) / count
    variance = sum((value - mean) ** 2 for value in present) / count
    std = sqrt(variance) if variance > 0 else 0.0
    level = max(abs(minimum), abs(maximum), 1.0)
    is_constant = (maximum - minimum) <= _DEGENERATE_TOLERANCE * level
    return ColumnProfile(name, count, missing, minimum, maximum, mean, std, is_constant)


# --------------------------------------------------------------------------- #
# Time-axis analysis                                                           #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class TimeProfile:
    """Structure of the time axis: extent, monotonicity, sampling regularity."""

    count: int
    start: float
    end: float
    monotonic: bool
    regular: bool
    step_mean: float
    step_min: float
    step_max: float
    step_cv: float

    def to_dict(self) -> dict[str, object]:
        return {
            "count": self.count,
            "start": self.start,
            "end": self.end,
            "monotonic": self.monotonic,
            "regular": self.regular,
            "step_mean": self.step_mean,
            "step_min": self.step_min,
            "step_max": self.step_max,
            "step_cv": self.step_cv,
        }


def _profile_time(times: Sequence[float]) -> TimeProfile:
    present = [float(t) for t in times if t is not None and isfinite(t)]
    count = len(present)
    if count == 0:
        nan = float("nan")
        return TimeProfile(0, nan, nan, False, False, nan, nan, nan, nan)
    start, end = present[0], present[-1]
    if count < 2:
        return TimeProfile(count, start, end, True, True, float("nan"), float("nan"), float("nan"), 0.0)
    steps = [b - a for a, b in zip(present, present[1:])]
    monotonic = all(step > 0 for step in steps)
    step_min = min(steps)
    step_max = max(steps)
    step_mean = sum(steps) / len(steps)
    if step_mean != 0:
        step_var = sum((s - step_mean) ** 2 for s in steps) / len(steps)
        step_cv = sqrt(step_var) / abs(step_mean)
    else:
        step_cv = float("inf")
    regular = monotonic and step_cv <= _REGULARITY_TOLERANCE
    return TimeProfile(count, start, end, monotonic, regular, step_mean, step_min, step_max, step_cv)


# --------------------------------------------------------------------------- #
# The full profile                                                             #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class DataProfile:
    """A structured, offline quality report for a time-series dataset."""

    name: str
    rows: int
    time: TimeProfile
    columns: tuple[ColumnProfile, ...]
    warnings: tuple[str, ...]

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "rows": self.rows,
            "time": self.time.to_dict(),
            "columns": [column.to_dict() for column in self.columns],
            "warnings": list(self.warnings),
        }

    def column(self, name: str) -> ColumnProfile:
        """Return the :class:`ColumnProfile` for ``name`` (raises if absent)."""
        for column in self.columns:
            if column.name == name:
                return column
        raise KeyError(name)

    # -- text view ---------------------------------------------------------- #

    def to_text(self) -> str:
        time = self.time
        lines = [
            f"Data profile — {self.name}",
            f"  rows: {self.rows}   columns: {len(self.columns)}",
            f"  time: [{_fmt(time.start)}, {_fmt(time.end)}] over {time.count} samples; "
            f"{'monotonic' if time.monotonic else 'NOT monotonic'}, "
            f"{'regular' if time.regular else 'irregular'} sampling "
            f"(mean Δt={_fmt(time.step_mean)}, cv={_fmt(time.step_cv)})",
            "",
        ]
        header = f"  {'column':<14}{'count':>7}{'missing':>9}{'min':>12}{'max':>12}{'mean':>12}{'std':>12}"
        lines.append(header)
        lines.append("  " + "-" * (len(header) - 2))
        for column in self.columns:
            flag = "  (constant)" if column.is_constant else ""
            lines.append(
                f"  {column.name:<14}{column.count:>7}{column.missing:>9}"
                f"{_fmt(column.minimum):>12}{_fmt(column.maximum):>12}"
                f"{_fmt(column.mean):>12}{_fmt(column.std):>12}{flag}"
            )
        lines.append("")
        if self.warnings:
            lines.append(f"  quality warnings ({len(self.warnings)}):")
            for warning in self.warnings:
                lines.append(f"    ! {warning}")
        else:
            lines.append("  quality warnings: none — data looks ready for discovery.")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return (
            f"DataProfile(name={self.name!r}, rows={self.rows}, "
            f"columns={[c.name for c in self.columns]}, warnings={len(self.warnings)})"
        )

    # -- themed HTML view --------------------------------------------------- #

    def _repr_html_(self) -> str:
        time = self.time
        head_cols = ["column", "count", "missing", "min", "max", "mean", "std"]
        head = "".join(
            f'<th style="padding:6px 10px;text-align:{"left" if i == 0 else "right"};'
            f'font-family:{_MONO};font-size:11px;letter-spacing:0.08em;text-transform:uppercase;'
            f'color:{_BRAND["muted"]};border-bottom:1px solid {_BRAND["line"]}">{escape(h)}</th>'
            for i, h in enumerate(head_cols)
        )
        body_rows = []
        for column in self.columns:
            flagged = column.is_constant or column.missing > 0
            name_color = _BRAND["danger"] if flagged else _BRAND["ink"]
            note = ""
            if column.is_constant:
                note = f' <span style="color:{_BRAND["warning"]};font-size:11px">constant</span>'
            cells = "".join(
                f'<td style="padding:6px 10px;text-align:right;font-family:{_MONO};'
                f'color:{_BRAND["ink"]};border-bottom:1px solid {_BRAND["line"]}">{escape(value)}</td>'
                for value in (
                    str(column.count),
                    str(column.missing),
                    _fmt(column.minimum),
                    _fmt(column.maximum),
                    _fmt(column.mean),
                    _fmt(column.std),
                )
            )
            body_rows.append(
                f'<tr><td style="padding:6px 10px;font-family:{_MONO};font-weight:600;'
                f'color:{name_color};border-bottom:1px solid {_BRAND["line"]}">'
                f"{escape(column.name)}{note}</td>{cells}</tr>"
            )
        table = (
            f'<table style="border-collapse:collapse;width:100%;margin-top:10px">'
            f"<thead><tr>{head}</tr></thead><tbody>{''.join(body_rows)}</tbody></table>"
        )

        time_state = (
            f'{"monotonic" if time.monotonic else "NON-monotonic"} · '
            f'{"regular" if time.regular else "irregular"} sampling'
        )
        time_color = _BRAND["success"] if (time.monotonic and time.regular) else _BRAND["warning"]
        kicker = (
            f'<div style="font-family:{_MONO};font-size:11px;letter-spacing:0.08em;'
            f'text-transform:uppercase;color:{_BRAND["muted"]}">data profile</div>'
        )
        meta = (
            f'<p style="margin:2px 0 0;color:{_BRAND["muted"]};font-size:14px">'
            f'{self.rows} rows · {len(self.columns)} columns · '
            f't ∈ [{_fmt(time.start)}, {_fmt(time.end)}] · '
            f'<span style="color:{time_color};font-weight:600">{escape(time_state)}</span> '
            f'(mean Δt={_fmt(time.step_mean)})</p>'
        )
        if self.warnings:
            items = "".join(
                f'<li style="margin:2px 0">{escape(w)}</li>' for w in self.warnings
            )
            warnings_html = (
                f'<div style="margin-top:12px;padding:10px 12px;border-radius:6px;'
                f'background:{_BRAND["accent_soft"]};border-left:3px solid {_BRAND["accent"]}">'
                f'<b style="color:{_BRAND["accent"]};font-family:{_SERIF}">'
                f'{len(self.warnings)} quality warning{"s" if len(self.warnings) != 1 else ""}</b>'
                f'<ul style="margin:6px 0 0;padding-left:18px;color:{_BRAND["ink"]}">{items}</ul></div>'
            )
        else:
            warnings_html = (
                f'<p style="margin-top:12px;color:{_BRAND["success"]};font-weight:600">'
                "No quality warnings — data looks ready for discovery.</p>"
            )
        return (
            f'<section style="font-family:{_SANS};font-size:14px;line-height:1.5;'
            f'background:{_BRAND["surface"]};color:{_BRAND["ink"]};'
            f'border:1px solid {_BRAND["line"]};border-radius:8px;'
            f'padding:16px 18px;margin:8px 0;max-width:840px">'
            f"{kicker}"
            f'<h3 style="margin:2px 0 0;font-family:{_SERIF};font-weight:650;'
            f'font-size:19px;color:{_BRAND["ink"]}">{escape(self.name)}</h3>'
            f"{meta}{table}{warnings_html}</section>"
        )


# --------------------------------------------------------------------------- #
# Warning synthesis                                                            #
# --------------------------------------------------------------------------- #


def _build_warnings(rows: int, time: TimeProfile, columns: Sequence[ColumnProfile]) -> tuple[str, ...]:
    warnings: list[str] = []
    if rows == 0:
        warnings.append("dataset is empty (no rows).")
    elif rows < 10:
        warnings.append(f"only {rows} rows; discovery is unreliable on very short series.")
    if time.count and not time.monotonic:
        warnings.append("time is not strictly increasing; sort or de-duplicate timestamps before discovery.")
    if time.count >= 2 and time.monotonic and not time.regular:
        warnings.append(
            f"sampling is irregular (Δt varies by cv={_fmt(time.step_cv)}); "
            "finite-difference derivatives assume a near-uniform grid."
        )
    for column in columns:
        if column.count == 0:
            warnings.append(f"column {column.name!r} has no usable values (all missing/non-numeric).")
            continue
        if column.missing:
            warnings.append(
                f"column {column.name!r} has {column.missing} missing/non-numeric "
                f"value(s) of {column.missing + column.count}."
            )
        if column.is_constant:
            warnings.append(
                f"column {column.name!r} is constant (≈{_fmt(column.mean)}); "
                "it carries no dynamics and may be dropped."
            )
    return tuple(warnings)


# --------------------------------------------------------------------------- #
# CSV ingestion (lenient — reports quality rather than rejecting it)           #
# --------------------------------------------------------------------------- #


def _maybe_float(cell: str | None) -> float | None:
    if cell is None:
        return None
    text = cell.strip()
    if not text:
        return None
    try:
        value = float(text)
    except (TypeError, ValueError):
        return None
    return value if isfinite(value) else None


def _read_csv(source: str, *, time: str, state: Sequence[str] | None, delimiter: str) -> tuple[list[float | None], dict[str, list[float | None]]]:
    if "\n" in source or "\r" in source:
        text = source  # inline CSV content
    else:
        path = Path(source)
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as error:
            raise ValidationError(f"cannot read CSV {source}: {error}") from error
    reader = csv.DictReader(text.splitlines(), delimiter=delimiter)
    if reader.fieldnames is None:
        raise ValidationError("CSV is empty or has no header row")
    if time not in reader.fieldnames:
        raise ValidationError(f"time column {time!r} not found; header is {reader.fieldnames}")
    if state is None:
        state_columns = [name for name in reader.fieldnames if name != time]
    else:
        state_columns = list(state)
        missing = [name for name in state_columns if name not in reader.fieldnames]
        if missing:
            raise ValidationError(f"columns {missing} not found; header is {reader.fieldnames}")
    if not state_columns:
        raise ValidationError("no numeric columns to profile besides the time column")
    times: list[float | None] = []
    columns: dict[str, list[float | None]] = {name: [] for name in state_columns}
    for row in reader:
        times.append(_maybe_float(row.get(time)))
        for name in state_columns:
            columns[name].append(_maybe_float(row.get(name)))
    return times, columns


# --------------------------------------------------------------------------- #
# Public entry point                                                           #
# --------------------------------------------------------------------------- #


def _profile_columns(times: Sequence[float | None], columns: Mapping[str, Sequence[float | None]]) -> DataProfile:
    rows = len(times)
    time_profile = _profile_time(times)
    column_profiles = tuple(_profile_column(name, list(values)) for name, values in columns.items())
    warnings = _build_warnings(rows, time_profile, column_profiles)
    # ``name`` is filled in by the caller.
    return DataProfile("dataset", rows, time_profile, column_profiles, warnings)


def profile(
    source: Dataset | str | PathLike[str],
    *,
    time: str = "time",
    state: Sequence[str] | None = None,
    name: str | None = None,
    delimiter: str = ",",
) -> DataProfile:
    """Profile a dataset or CSV and return a structured :class:`DataProfile`.

    ``source`` may be a validated :class:`~lawsynth.dataset.Dataset`, a path to a
    CSV file, or inline CSV text. For CSV input, ``time`` names the time column
    and ``state`` (optional) selects which columns to profile — omit it to
    profile every non-time column. CSV cells that are empty or non-numeric are
    counted as *missing* rather than rejected, so the profile can surface data
    quality problems before discovery.

    The result is deterministic and computed with the standard library only.
    """
    if isinstance(source, Dataset):
        times: list[float | None] = [float(value) for value in source.time]
        if state is None:
            selected = dict(source.columns)
        else:
            missing = [column for column in state if column not in source.columns]
            if missing:
                raise ValidationError(f"columns {missing} not present in dataset")
            selected = {column: source.columns[column] for column in state}
        column_map: dict[str, list[float | None]] = {
            column: [float(value) for value in values] for column, values in selected.items()
        }
        resolved_name = name or "dataset"
    elif isinstance(source, (str, PathLike)):
        raw_times, raw_columns = _read_csv(str(source), time=time, state=state, delimiter=delimiter)
        times = raw_times
        column_map = raw_columns
        resolved_name = name or (Path(str(source)).stem if "\n" not in str(source) else "csv")
    else:  # pragma: no cover - defensive
        raise ValidationError(
            f"profile() expects a Dataset, a CSV path, or CSV text; got {type(source).__name__}"
        )

    result = _profile_columns(times, column_map)
    return DataProfile(resolved_name, result.rows, result.time, result.columns, result.warnings)
