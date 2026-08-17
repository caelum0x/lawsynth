"""Streaming / online discovery (P7).

Discovering and *maintaining* a model on continuously arriving data. This module
processes a series as if it streamed: it advances an explicit window across the
time column, keeps a **current** model, and re-discovers only on a *sustained*
standardized-residual drift — a regime/law change — as opposed to a transient
outlier (which :mod:`lawsynth.monitor` handles). Every model update emits an
immutable :class:`ChangeRecord` documenting the transition: the prior and new
world revision hashes, the triggering window, and a per-law diff of which
terms/coefficients changed. Reading the change-record stream reconstructs the
full model history.

Window policy (explicit, in the time column's units, never wall clock):

* **sliding** (default): a fixed-width window of ``window`` samples advanced by
  ``step`` samples.
* **growing** (``growing=True``): an anchored window that grows by ``step``
  from a minimum training size up to a hard cap of ``window`` samples, then
  slides at the cap.

Determinism under replay: every step is deterministic and offline — no wall
clock, no ambient randomness (the discovery boundary performs no resampling on
the default path). Replaying the identical series through the identical
window/config yields the identical sequence of models and change records.

Honesty: this is **not** incremental learning. Each model is re-discovered from
scratch over its triggering window (a batched re-run), not updated in place. The
change-record stream still reconstructs the full model history and replay is
reproducible; the "online efficiency" claim is not made.
"""

from __future__ import annotations

from dataclasses import dataclass
from html import escape
from os import PathLike
from typing import Mapping, Sequence

from ._content import SHORT, world_hash
from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import ValidationError

__all__ = [
    "stream_discover",
    "StreamHistory",
    "StreamModel",
    "ChangeRecord",
    "TermChange",
]

# Re-discover on a window whose residual RMS exceeds this many times the current
# model's own fit-residual scale, sustained over `sustain` consecutive windows.
_DEFAULT_THRESHOLD = 4.0
_DEFAULT_SUSTAIN = 2
_DEFAULT_WINDOW = 60
# Floor on a model's fit-residual scale as a fraction of the fitted signal RMS,
# so a near-perfect fit cannot amplify rounding noise into a spurious drift.
_MIN_RELATIVE_SCALE = 1e-3


# --------------------------------------------------------------------------- #
# Result types                                                                #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class TermChange:
    """One per-law coefficient change across a model transition."""

    target: str
    feature: str
    prior: float
    new: float
    kind: str  # "added" | "removed" | "changed"

    def to_dict(self) -> dict[str, object]:
        return {
            "target": self.target,
            "feature": self.feature,
            "prior": self.prior,
            "new": self.new,
            "kind": self.kind,
        }


@dataclass(frozen=True, slots=True)
class StreamModel:
    """A model discovered at one point in the stream."""

    sequence: int
    kind: str  # "initial" | "update"
    revision: str
    window_index: int
    rows: tuple[int, int]
    time_span: tuple[float, float]
    equations: Mapping[str, str]

    def to_dict(self) -> dict[str, object]:
        return {
            "sequence": self.sequence,
            "kind": self.kind,
            "revision": self.revision,
            "window_index": self.window_index,
            "rows": list(self.rows),
            "time_span": list(self.time_span),
            "equations": dict(self.equations),
        }


@dataclass(frozen=True, slots=True)
class ChangeRecord:
    """An immutable record of one model update, reconstructing model history."""

    sequence: int
    prior_revision: str | None
    new_revision: str
    window_index: int
    rows: tuple[int, int]
    time_span: tuple[float, float]
    sustained_windows: int
    drift_ratio: float
    diff: tuple[TermChange, ...]
    equations: Mapping[str, str]

    def to_dict(self) -> dict[str, object]:
        return {
            "sequence": self.sequence,
            "prior_revision": self.prior_revision,
            "new_revision": self.new_revision,
            "trigger": {
                "window_index": self.window_index,
                "rows": list(self.rows),
                "time_span": list(self.time_span),
                "sustained_windows": self.sustained_windows,
                "drift_ratio": self.drift_ratio,
            },
            "diff": [change.to_dict() for change in self.diff],
            "equations": dict(self.equations),
        }


class StreamHistory:
    """The models and change records produced by a streaming run.

    Holds the ordered list of :class:`StreamModel` (the initial model plus one
    per re-discovery) and the :class:`ChangeRecord` stream (one per update).
    Reading the change records reconstructs the full model history.
    """

    __slots__ = ("_models", "_changes", "_windows", "_name", "_policy")

    def __init__(
        self,
        models: Sequence[StreamModel],
        changes: Sequence[ChangeRecord],
        *,
        windows: int,
        name: str,
        policy: str,
    ) -> None:
        self._models = tuple(models)
        self._changes = tuple(changes)
        self._windows = windows
        self._name = name
        self._policy = policy

    # -- accessors ---------------------------------------------------------- #

    @property
    def models(self) -> tuple[StreamModel, ...]:
        return self._models

    @property
    def changes(self) -> tuple[ChangeRecord, ...]:
        """The change-record stream (one per model update)."""
        return self._changes

    @property
    def change_points(self) -> tuple[tuple[int, float], ...]:
        """(window index, start time) of each detected regime change."""
        return tuple((c.window_index, c.time_span[0]) for c in self._changes)

    @property
    def current(self) -> StreamModel:
        """The latest model in the stream."""
        return self._models[-1]

    @property
    def name(self) -> str:
        return self._name

    def __len__(self) -> int:
        return len(self._models)

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self._name,
            "policy": self._policy,
            "windows_processed": self._windows,
            "models_produced": len(self._models),
            "change_points": [
                {"window_index": index, "time": time} for index, time in self.change_points
            ],
            "models": [model.to_dict() for model in self._models],
            "changes": [change.to_dict() for change in self._changes],
            "note": (
                "each model is re-discovered from scratch over its triggering "
                "window (a batched re-run), not incrementally updated"
            ),
        }

    def to_text(self) -> str:
        lines = [
            f"Stream history — {self._name}",
            f"  policy: {self._policy}",
            f"  windows processed: {self._windows}",
            f"  models produced: {len(self._models)} "
            f"(1 initial + {len(self._changes)} re-discovery)",
        ]
        if self._changes:
            listed = ", ".join(
                f"window {index} (t={time:.4g})" for index, time in self.change_points
            )
            lines.append(f"  change points: {listed}")
        else:
            lines.append("  change points: none (dynamics stable across the stream)")
        for model in self._models:
            tag = model.kind
            lines.append(f"  [{model.sequence}] {tag} @ window {model.window_index} "
                         f"rev {model.revision[:SHORT]}")
            for target in sorted(model.equations):
                lines.append(f"        d{target}/dt = {model.equations[target]}")
        for change in self._changes:
            named = ", ".join(
                f"{c.target}:{c.feature}({c.kind})" for c in change.diff
            ) or "(no term-level change)"
            lines.append(
                f"  change {change.sequence}: {change.prior_revision[:SHORT]} -> "
                f"{change.new_revision[:SHORT]} at window {change.window_index}; diff: {named}"
            )
        lines.append(
            "  NOTE: models are re-discovered from scratch over each triggering "
            "window (a batched re-run), not incrementally updated."
        )
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return (
            f"StreamHistory(name={self._name!r}, models={len(self._models)}, "
            f"changes={len(self._changes)})"
        )

    # -- HTML timeline ------------------------------------------------------ #

    def _repr_html_(self) -> str:
        nodes = []
        for model in self._models:
            is_update = model.kind == "update"
            color = "#a3341f" if is_update else "#155e75"
            eqs = "".join(
                f'<li style="font-family:ui-monospace,monospace;font-size:12px">'
                f"d{escape(t)}/dt = {escape(model.equations[t])}</li>"
                for t in sorted(model.equations)
            )
            change_html = ""
            if is_update:
                change = next((c for c in self._changes if c.new_revision == model.revision), None)
                if change is not None and change.diff:
                    items = "".join(
                        f'<li style="font-size:12px">{escape(c.target)} · '
                        f"<b>{escape(c.feature)}</b> {escape(c.kind)}: "
                        f"{c.prior:.4g} → {c.new:.4g}</li>"
                        for c in change.diff
                    )
                    change_html = (
                        '<div style="margin-top:6px;padding:6px 8px;border-radius:6px;'
                        'background:#f6dccf;border-left:3px solid #a3341f">'
                        f'<b style="color:#a3341f">regime change</b>'
                        f'<ul style="margin:4px 0 0;padding-left:18px">{items}</ul></div>'
                    )
            nodes.append(
                '<div style="border:1px solid #cbd5e1;border-radius:8px;padding:10px 12px;'
                'margin:6px 0;background:#fff">'
                f'<div style="font-weight:700;color:{color}">[{model.sequence}] {escape(model.kind)}'
                f' · window {model.window_index} · t∈[{model.time_span[0]:.4g}, '
                f'{model.time_span[1]:.4g}] · rev {escape(model.revision[:SHORT])}</div>'
                f'<ul style="margin:6px 0 0;padding-left:18px">{eqs}</ul>{change_html}</div>'
            )
        return (
            '<section style="font:14px system-ui;border:1px solid #cbd5e1;border-radius:10px;'
            'padding:16px 18px;margin:8px 0;max-width:820px">'
            f'<h3 style="margin:0 0 4px;color:#155e75">Stream — {escape(self._name)}</h3>'
            f'<p style="margin:0 0 8px;color:#53627a">{escape(self._policy)} · '
            f"{self._windows} windows · {len(self._models)} model(s) · "
            f"{len(self._changes)} change point(s)</p>"
            + "".join(nodes)
            + '<p style="margin:8px 0 0;color:#8a93a6;font-size:12px">'
            "Models are re-discovered from scratch over each triggering window "
            "(a batched re-run), not incrementally updated.</p></section>"
        )


# --------------------------------------------------------------------------- #
# Core algorithm                                                               #
# --------------------------------------------------------------------------- #


def stream_discover(
    dataset_or_csv: Dataset | str | PathLike[str],
    *,
    time: str,
    state: Sequence[str],
    window: int = _DEFAULT_WINDOW,
    step: int | None = None,
    threshold: float = _DEFAULT_THRESHOLD,
    sustain: int = _DEFAULT_SUSTAIN,
    config: DiscoveryConfig | None = None,
    growing: bool = False,
    name: str = "stream",
) -> StreamHistory:
    """Discover and maintain models over a series that arrives as a stream.

    ``dataset_or_csv`` is a :class:`~lawsynth.dataset.Dataset` or a path to a CSV
    (parsed with ``time``/``state`` columns). A window advances across the time
    column; the first full window seeds the model, and re-discovery fires only on
    a **sustained** residual drift — a window whose residual RMS under the current
    model exceeds ``threshold`` times that model's own fit-residual scale, over
    ``sustain`` consecutive windows. Returns a :class:`StreamHistory`.

    Deterministic and offline: replaying the identical series through the
    identical window/config yields identical models and change records.
    """
    states = list(state)
    if not states:
        raise ValidationError("at least one state variable is required")
    if window < 1:
        raise ValidationError("window must be at least 1")
    resolved_step = int(step) if step is not None else window
    if resolved_step < 1:
        raise ValidationError("step must be at least 1")
    if sustain < 1:
        raise ValidationError("sustain must be at least 1")
    if not (threshold > 0):
        raise ValidationError("threshold must be positive")
    base_config = config or DiscoveryConfig()

    dataset = _as_dataset(dataset_or_csv, time, states)
    missing = [s for s in states if s not in dataset.columns]
    if missing:
        raise ValidationError(f"state variables not present in dataset: {missing}")

    samples = len(dataset.time)
    min_train = _min_train(window, base_config.polynomial_degree)
    if samples < min_train:
        raise ValidationError(
            f"need at least {min_train} observations to seed a model (got {samples})"
        )

    ranges = _window_ranges(samples, window, resolved_step, growing, min_train)
    if not ranges:
        raise ValidationError("no windows produced; check window/step against the sample count")

    # Deferred to keep the module import cheap and reuse the single discovery
    # choke-point and native simulation helpers.
    from .study import _discover_world

    models: list[StreamModel] = []
    changes: list[ChangeRecord] = []
    time_axis = dataset.time

    # Seed the first model on the first window.
    seed_range = ranges[0]
    seed_world = _discover_world(_slice(dataset, states, seed_range), states, base_config)
    seed_equations = dict(seed_world.equations())
    seed_terms = _term_maps(seed_equations)
    seed_scale = _fit_scales(seed_world, dataset, states, seed_range)
    current = _CurrentModel(seed_world, world_hash(seed_world), seed_terms, seed_scale)
    models.append(
        StreamModel(
            sequence=0,
            kind="initial",
            revision=current.revision,
            window_index=0,
            rows=(seed_range[0], seed_range[1]),
            time_span=(time_axis[seed_range[0]], time_axis[seed_range[1] - 1]),
            equations=seed_equations,
        )
    )

    sequence = 1
    streak = 0
    for index in range(1, len(ranges)):
        rng = ranges[index]
        drift = _window_drift(current, dataset, states, rng)
        if drift > threshold:
            streak += 1
        else:
            streak = 0
        if streak < sustain:
            continue
        # Sustained drift confirmed: re-discover over the triggering window.
        new_world = _discover_world(_slice(dataset, states, rng), states, base_config)
        new_equations = dict(new_world.equations())
        new_terms = _term_maps(new_equations)
        diff = _diff_terms(current.terms, new_terms)
        new_revision = world_hash(new_world)
        span = (time_axis[rng[0]], time_axis[rng[1] - 1])
        changes.append(
            ChangeRecord(
                sequence=sequence,
                prior_revision=current.revision,
                new_revision=new_revision,
                window_index=index,
                rows=(rng[0], rng[1]),
                time_span=span,
                sustained_windows=streak,
                drift_ratio=drift,
                diff=tuple(diff),
                equations=new_equations,
            )
        )
        models.append(
            StreamModel(
                sequence=sequence,
                kind="update",
                revision=new_revision,
                window_index=index,
                rows=(rng[0], rng[1]),
                time_span=span,
                equations=new_equations,
            )
        )
        current = _CurrentModel(
            new_world,
            new_revision,
            new_terms,
            _fit_scales(new_world, dataset, states, rng),
        )
        sequence += 1
        streak = 0

    policy = (
        f"growing (grows by {resolved_step} to a cap of {window} samples)"
        if growing
        else f"sliding (width {window} step {resolved_step})"
    )
    policy += f"; threshold K={threshold:g}, sustain {sustain}"
    return StreamHistory(models, changes, windows=len(ranges), name=name, policy=policy)


class _CurrentModel:
    """Mutable-by-replacement holder for the in-flight model and its baselines."""

    __slots__ = ("world", "revision", "terms", "scale")

    def __init__(
        self,
        world: object,
        revision: str,
        terms: Mapping[str, Mapping[str, float]],
        scale: Mapping[str, float],
    ) -> None:
        self.world = world
        self.revision = revision
        self.terms = terms
        self.scale = scale


def _as_dataset(
    source: Dataset | str | PathLike[str], time: str, states: Sequence[str]
) -> Dataset:
    if isinstance(source, Dataset):
        return source
    from .study import Study

    return Study.from_csv(source, time=time, state=list(states)).dataset


def _min_train(window: int, degree: int) -> int:
    return min(window, max(8, 4 * (degree + 1)))


def _window_ranges(
    samples: int, window: int, step: int, growing: bool, min_train: int
) -> list[tuple[int, int]]:
    ranges: list[tuple[int, int]] = []
    if growing:
        end = min(min_train, samples)
        while True:
            start = max(0, end - window)
            ranges.append((start, end))
            if end == samples:
                break
            end = min(end + step, samples)
    else:
        if window > samples:
            return ranges
        start = 0
        while start + window <= samples:
            ranges.append((start, start + window))
            start += step
    return ranges


def _slice(dataset: Dataset, states: Sequence[str], rng: tuple[int, int]) -> Dataset:
    lo, hi = rng
    time = dataset.time[lo:hi]
    columns = {s: dataset.columns[s][lo:hi] for s in states}
    return Dataset.from_columns(time, columns)


def _rms(values: Sequence[float]) -> float:
    if not values:
        return 0.0
    return (sum(v * v for v in values) / len(values)) ** 0.5


def _residual_rms(
    world: object, dataset: Dataset, states: Sequence[str], rng: tuple[int, int]
) -> dict[str, float]:
    """Per-state residual RMS of ``world`` over a window (seeded from its first row)."""
    from .prepare import interpolate
    from .study import _default_step, _run_native

    lo, hi = rng
    times = dataset.time[lo:hi]
    initial = {s: float(dataset.columns[s][lo]) for s in states}
    step = _default_step(_slice(dataset, states, rng))
    scales: dict[str, float] = {}
    try:
        simulated = _run_native(world, initial, start=float(times[0]), end=float(times[-1]), step=step)
    except Exception:
        # A world that cannot be integrated over the window has diverged.
        return {s: float("inf") for s in states}
    for s in states:
        predicted = interpolate(simulated.time, simulated.values.get(s, ()), times)
        observed = dataset.columns[s][lo:hi]
        residual = [o - p for o, p in zip(observed, predicted)]
        scales[s] = _rms(residual)
    return scales


def _fit_scales(
    world: object, dataset: Dataset, states: Sequence[str], rng: tuple[int, int]
) -> dict[str, float]:
    """The model's fit-residual scale per state, floored to a fraction of signal RMS."""
    residuals = _residual_rms(world, dataset, states, rng)
    lo, hi = rng
    scale: dict[str, float] = {}
    for s in states:
        signal_rms = max(_rms(dataset.columns[s][lo:hi]), 1.0)
        floor = signal_rms * _MIN_RELATIVE_SCALE
        scale[s] = max(residuals.get(s, float("inf")), floor)
    return scale


def _window_drift(
    current: _CurrentModel, dataset: Dataset, states: Sequence[str], rng: tuple[int, int]
) -> float:
    """Peak per-state standardized drift of the current model over a window."""
    residuals = _residual_rms(current.world, dataset, states, rng)
    drift = 0.0
    for s in states:
        scale = current.scale.get(s, float("inf"))
        residual = residuals.get(s, float("inf"))
        ratio = residual / scale if scale > 0 else float("inf")
        drift = max(drift, ratio)
    return drift


def _term_maps(equations: Mapping[str, str]) -> dict[str, dict[str, float]]:
    """Per-target additive ``feature -> coefficient`` maps (reusing Study's parser)."""
    from .study import _build_law

    maps: dict[str, dict[str, float]] = {}
    for target, expression in equations.items():
        law = _build_law(target, expression)
        terms: dict[str, float] = {}
        for coefficient, feature in law.terms:
            terms[feature] = terms.get(feature, 0.0) + coefficient
        maps[target] = {f: c for f, c in terms.items() if abs(c) > 0.0}
    return maps


def _diff_terms(
    prior: Mapping[str, Mapping[str, float]], new: Mapping[str, Mapping[str, float]]
) -> list[TermChange]:
    """Every feature whose coefficient moved between the two models, per law."""
    changes: list[TermChange] = []
    for target in sorted(set(prior) | set(new)):
        prior_terms = prior.get(target, {})
        new_terms = new.get(target, {})
        for feature in sorted(set(prior_terms) | set(new_terms)):
            before = prior_terms.get(feature, 0.0)
            after = new_terms.get(feature, 0.0)
            tolerance = 1e-9 * (1.0 + max(abs(before), abs(after)))
            if abs(before - after) <= tolerance:
                continue
            kind = "added" if before == 0.0 else "removed" if after == 0.0 else "changed"
            changes.append(TermChange(target=target, feature=feature, prior=before, new=after, kind=kind))
    return changes
