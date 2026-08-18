"""Experiment-tracking interop for LawSynth discovery runs (MLflow / W&B).

A LawSynth discovery run has three things an MLOps tracker wants to record:

* **params** — the discovery configuration (``polynomial_degree``, ``threshold``,
  ``solver``, ``derivative_method``, the feature-family toggles, …);
* **metrics** — how good the recovered world is (per-state ``r_squared`` / ``rmse``
  fit, plus structural size: law count, additive-term count, AST-node complexity);
* an **artifact** — the portable ``.lsworld`` bundle the run produced.

The anchor of this module is :class:`RunRecord`: a dependency-free, deterministic
snapshot of exactly those three things, built from a :class:`~lawsynth.study.Study`
or :class:`~lawsynth.study.DiscoveryResult`. It serialises to a plain, byte-stable
dict/JSON and is fully usable and testable with *none* of ``mlflow``/``wandb``
installed — it is honest by construction, recording only the params and metrics a
run genuinely has (an absent metric is omitted, never fabricated).

:func:`log_to_mlflow` and :func:`log_to_wandb` push that same record to the two
trackers the ecosystem already uses. Each imports its backend lazily and, when the
backend is absent, raises a clear, typed :class:`MissingDependencyError` (a subclass
of both :class:`~lawsynth.errors.LawSynthError` and the built-in :class:`ImportError`,
mirroring :mod:`lawsynth.export`). Building the record touches no network; only the
final ``log_to_*`` call talks to a tracker.
"""

from __future__ import annotations

import contextlib
import hashlib
import importlib
import json
import re
import tempfile
from dataclasses import dataclass, field
from math import isfinite
from os import PathLike
from pathlib import Path
from typing import Iterator, Mapping, Sequence

from ._content import world_hash
from ._version import __version__
from .errors import LawSynthError

__all__ = [
    "TrackingError",
    "MissingDependencyError",
    "RunArtifact",
    "RunRecord",
    "run_record",
    "log_to_mlflow",
    "log_to_wandb",
]


# --------------------------------------------------------------------------- #
# Errors — mirror lawsynth.export's graceful-absence contract                  #
# --------------------------------------------------------------------------- #


class TrackingError(LawSynthError):
    """Raised when a discovery run cannot be turned into a tracking record."""


class MissingDependencyError(TrackingError, ImportError):
    """Raised when an optional tracking backend (mlflow/wandb) is not installed.

    Subclasses :class:`ImportError` as well as :class:`~lawsynth.errors.LawSynthError`
    so callers can catch it either way and ``pytest.importorskip`` semantics feel
    natural — exactly like :class:`lawsynth.export.MissingDependencyError`.
    """


def _require(module: str, feature: str):
    """Import an optional tracking backend or raise a clear, typed error."""
    try:
        return importlib.import_module(module)
    except ImportError as error:  # pragma: no cover - exercised only when absent
        raise MissingDependencyError(
            f"{feature} requires the optional dependency {module!r}, which is not "
            f"installed. Install it (e.g. `pip install {module}`) to enable this "
            f"integration. LawSynth's core is dependency-free; {module} is only "
            f"needed to log to that experiment tracker."
        ) from error


# --------------------------------------------------------------------------- #
# The anchor: a deterministic, dependency-free run record                      #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class RunArtifact:
    """A reference to a produced ``.lsworld`` bundle (path + content digest).

    ``path``/``sha256``/``size_bytes`` are populated only when the bundle was
    actually written (``run_record(..., artifact_path=...)``); otherwise just the
    intended ``filename`` is recorded, so nothing is fabricated.
    """

    filename: str
    path: str | None = None
    sha256: str | None = None
    size_bytes: int | None = None

    def to_dict(self) -> dict[str, object]:
        return {
            "filename": self.filename,
            "path": self.path,
            "sha256": self.sha256,
            "size_bytes": self.size_bytes,
        }


@dataclass(frozen=True, slots=True)
class RunRecord:
    """A deterministic, dependency-free snapshot of a discovery run.

    ``params`` are the discovery configuration knobs, ``metrics`` are the numeric
    quality/size measures that genuinely exist for the run (absent ones are simply
    missing), ``tags`` are string metadata (variables, engine version, world
    revision hash), and ``artifact`` references the ``.lsworld`` bundle. All three
    mappings are serialised with sorted keys, so :meth:`to_dict` / :meth:`to_json`
    are byte-stable across runs.
    """

    name: str
    params: Mapping[str, object] = field(default_factory=dict)
    metrics: Mapping[str, float] = field(default_factory=dict)
    tags: Mapping[str, str] = field(default_factory=dict)
    artifact: RunArtifact | None = None

    def to_dict(self) -> dict[str, object]:
        """A plain, deterministic dict: every mapping is emitted with sorted keys."""
        return {
            "name": self.name,
            "params": {key: self.params[key] for key in sorted(self.params)},
            "metrics": {key: self.metrics[key] for key in sorted(self.metrics)},
            "tags": {key: self.tags[key] for key in sorted(self.tags)},
            "artifact": self.artifact.to_dict() if self.artifact is not None else None,
        }

    def to_json(self, *, indent: int | None = 2) -> str:
        """Byte-stable JSON (``sort_keys=True``) — identical inputs, identical bytes."""
        return json.dumps(self.to_dict(), sort_keys=True, indent=indent)

    def __repr__(self) -> str:
        return (
            f"RunRecord(name={self.name!r}, params={len(self.params)}, "
            f"metrics={len(self.metrics)}, tags={len(self.tags)}, "
            f"artifact={self.artifact.filename if self.artifact else None!r})"
        )


# --------------------------------------------------------------------------- #
# Extraction — the single internal function both backends and the anchor share #
# --------------------------------------------------------------------------- #

# Discovery config fields, in a fixed order so params are deterministic. Read
# from DiscoveryConfig itself so this list can never drift from the real config.
def _config_field_names() -> tuple[str, ...]:
    from .config import DiscoveryConfig

    return tuple(sorted(DiscoveryConfig.__dataclass_fields__))


def _equations_of(source: object) -> dict[str, str]:
    """Recover ``{target: expression}`` from a Study / DiscoveryResult / World."""
    equations = getattr(source, "equations", None)
    if equations is not None:
        resolved = equations() if callable(equations) else equations
        return {str(k): str(v) for k, v in dict(resolved).items()}
    world = _world_like(source)
    if world is not None and callable(getattr(world, "equations", None)):
        return {str(k): str(v) for k, v in dict(world.equations()).items()}
    raise TrackingError(
        "cannot build a RunRecord: the object exposes no equations() (pass a "
        "Study, a DiscoveryResult, or a native World)"
    )


def _world_like(source: object) -> object | None:
    """The world-ish object to hash: ``source.world`` if present, else ``source``."""
    try:
        world = getattr(source, "world", None)
    except LawSynthError:  # Study.world raises before discover()
        return None
    if world is not None:
        return world
    if callable(getattr(source, "equations", None)):
        return source
    return None


def _config_of(source: object):
    """The DiscoveryConfig backing ``source``, or ``None`` if unavailable."""
    config = getattr(source, "_config", None)
    if config is None:
        return None
    if getattr(type(config), "__dataclass_fields__", None) is None:
        return None
    return config


def _states_of(source: object, equations: Mapping[str, str]) -> tuple[str, ...]:
    states = getattr(source, "states", None)
    if states:
        return tuple(str(s) for s in states)
    return tuple(sorted(equations))


def _name_of(source: object) -> str:
    name = getattr(source, "name", None)
    if isinstance(name, str) and name:
        return name
    return "lawsynth-run"


def _structural_metrics(equations: Mapping[str, str]) -> dict[str, float]:
    """Deterministic, dependency-free size measures from the law strings alone.

    ``law_count`` and ``complexity_nodes`` always resolve (pure stdlib ``ast``);
    ``term_count`` is included only when every law flattens to additive terms, so
    it is omitted rather than guessed for expressions the term-splitter can't read.
    """
    from .algebra import node_count

    metrics: dict[str, float] = {
        "law_count": float(len(equations)),
        "complexity_nodes": float(sum(node_count(expr) for expr in equations.values())),
    }
    term_total = _term_count(equations)
    if term_total is not None:
        metrics["term_count"] = float(term_total)
    return metrics


def _term_count(equations: Mapping[str, str]) -> int | None:
    from .study import _extract_terms

    total = 0
    for expression in equations.values():
        try:
            total += len(_extract_terms(expression))
        except LawSynthError:
            return None
    return total


def _fit_metrics(source: object) -> dict[str, float]:
    """Per-state ``r_squared``/``rmse`` (plus aggregates) — only if genuinely available.

    Fit needs a simulate-able world bound to its dataset (``source.explain()``).
    When that is absent or fails, an empty dict is returned and no fit metric is
    logged — never a fabricated one.
    """
    explain = getattr(source, "explain", None)
    if not callable(explain):
        return {}
    try:
        fit = getattr(explain(), "fit", None)
    except Exception:
        return {}
    if not fit:
        return {}
    metrics: dict[str, float] = {}
    r_squared: list[float] = []
    rmse: list[float] = []
    for state, values in fit.items():
        r2 = _finite(values.get("r_squared"))
        err = _finite(values.get("rmse"))
        if r2 is not None:
            metrics[f"r_squared_{state}"] = r2
            r_squared.append(r2)
        if err is not None:
            metrics[f"rmse_{state}"] = err
            rmse.append(err)
    if r_squared:
        metrics["r_squared_mean"] = sum(r_squared) / len(r_squared)
        metrics["r_squared_min"] = min(r_squared)
    if rmse:
        metrics["rmse_mean"] = sum(rmse) / len(rmse)
        metrics["rmse_max"] = max(rmse)
    return metrics


def _finite(value: object) -> float | None:
    try:
        number = float(value)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return None
    return number if isfinite(number) else None


def _to_record(
    source: object,
    *,
    artifact_path: str | PathLike[str] | None = None,
    tags: Mapping[str, str] | None = None,
    extra_params: Mapping[str, object] | None = None,
    extra_metrics: Mapping[str, float] | None = None,
) -> RunRecord:
    """Turn a Study/DiscoveryResult/World into a :class:`RunRecord` (shared by all)."""
    equations = _equations_of(source)
    name = _name_of(source)
    states = _states_of(source, equations)

    params: dict[str, object] = {}
    config = _config_of(source)
    if config is not None:
        for field_name in _config_field_names():
            params[field_name] = getattr(config, field_name)
    if extra_params:
        params.update({str(k): v for k, v in extra_params.items()})

    metrics: dict[str, float] = _structural_metrics(equations)
    metrics.update(_fit_metrics(source))
    if extra_metrics:
        for key, value in extra_metrics.items():
            number = _finite(value)
            if number is not None:
                metrics[str(key)] = number

    record_tags: dict[str, str] = {
        "framework": "lawsynth",
        "engine_version": __version__,
        "variables": ",".join(states),
    }
    revision = _world_revision(source, equations)
    if revision is not None:
        record_tags["world_revision"] = revision
    if tags:
        record_tags.update({str(k): str(v) for k, v in tags.items()})

    artifact = _capture_artifact(source, artifact_path, name)
    return RunRecord(name=name, params=params, metrics=metrics, tags=record_tags, artifact=artifact)


def _world_revision(source: object, equations: Mapping[str, str]) -> str | None:
    """The content-addressed world revision hash.

    Uses the SDK's :func:`~lawsynth._content.world_hash` when a live world is
    available (so it matches the lineage hash exactly); otherwise reconstructs the
    same canonical payload from the extracted law strings, keeping the record
    dependency-free and native-free while staying byte-identical for a discovered
    world (which inlines all coefficients, so it has no parameters or controls).
    """
    world = _world_like(source)
    if world is not None and callable(getattr(world, "equations", None)):
        try:
            return world_hash(world)
        except Exception:  # pragma: no cover - defensive; metadata is best-effort
            return None
    return _revision_from_equations(equations)


def _revision_from_equations(equations: Mapping[str, str]) -> str | None:
    from ._content import content_digest
    from .worldspec import free_identifiers

    if not equations:
        return None
    states = tuple(equations)
    bound = frozenset(states)
    controls: set[str] = set()
    for expression in equations.values():
        controls |= free_identifiers(expression, bound)
    payload = {
        "states": sorted(states),
        "parameters": [],
        "controls": sorted(controls),
        "equations": sorted((str(t), str(e)) for t, e in equations.items()),
    }
    return content_digest(payload)


# --------------------------------------------------------------------------- #
# Artifact handling — writing / locating the .lsworld bundle                    #
# --------------------------------------------------------------------------- #


def _save_bundle(source: object, target: Path) -> None:
    """Persist ``source``'s world to ``target`` as a ``.lsworld`` bundle."""
    saver = getattr(source, "save", None)
    if callable(saver):
        saver(str(target))
        return
    world = _world_like(source)
    if world is not None and callable(getattr(world, "save", None)):
        world.save(str(target))
        return
    raise TrackingError(
        "source cannot be saved as a .lsworld bundle (needs a discovered world)"
    )


def _capture_artifact(
    source: object, artifact_path: str | PathLike[str] | None, name: str
) -> RunArtifact:
    """Record the artifact reference, writing the bundle when a path is given."""
    if artifact_path is None:
        return RunArtifact(filename=f"{_slug(name)}.lsworld")
    target = Path(artifact_path)
    _save_bundle(source, target)
    data = target.read_bytes()
    return RunArtifact(
        filename=target.name,
        path=str(target),
        sha256=hashlib.sha256(data).hexdigest(),
        size_bytes=len(data),
    )


@contextlib.contextmanager
def _artifact_file(record: RunRecord, source: object) -> Iterator[Path | None]:
    """Yield a readable ``.lsworld`` path for logging, or ``None`` if unavailable.

    Prefers an already-written bundle referenced by the record; otherwise, when a
    live source is available, writes one to a temporary directory that is cleaned
    up afterwards. Yields ``None`` (rather than inventing an artifact) when neither
    is possible.
    """
    if record.artifact is not None and record.artifact.path:
        existing = Path(record.artifact.path)
        if existing.is_file():
            yield existing
            return
    if source is None:
        yield None
        return
    with tempfile.TemporaryDirectory(prefix="lawsynth-track-") as tmp:
        target = Path(tmp) / f"{_slug(record.name)}.lsworld"
        try:
            _save_bundle(source, target)
        except TrackingError:
            yield None
            return
        yield target


def _slug(name: str) -> str:
    """A filesystem/tracker-safe slug (keeps only ``[A-Za-z0-9._-]``)."""
    slug = re.sub(r"[^A-Za-z0-9._-]+", "-", name).strip("-")
    return slug or "lawsynth-run"


def _coerce(record_or_source: object) -> tuple[RunRecord, object | None]:
    """Normalise the public arg into ``(record, live_source_or_None)``."""
    if isinstance(record_or_source, RunRecord):
        return record_or_source, None
    return _to_record(record_or_source), record_or_source


# --------------------------------------------------------------------------- #
# Public anchor + backend loggers                                              #
# --------------------------------------------------------------------------- #


def run_record(
    source: object,
    *,
    artifact_path: str | PathLike[str] | None = None,
    tags: Mapping[str, str] | None = None,
    extra_params: Mapping[str, object] | None = None,
    extra_metrics: Mapping[str, float] | None = None,
) -> RunRecord:
    """Build a deterministic, dependency-free :class:`RunRecord` from a run.

    ``source`` is a :class:`~lawsynth.study.Study`, a
    :class:`~lawsynth.study.DiscoveryResult`, or a native ``World``. When
    ``artifact_path`` is given the ``.lsworld`` bundle is written there and its
    content digest recorded. ``tags``/``extra_params``/``extra_metrics`` let you
    fold in genuine, caller-owned values (e.g. a wall-clock ``training_time`` you
    measured) — LawSynth never invents metrics it cannot observe.

    Needs none of ``mlflow``/``wandb`` and performs no network I/O.
    """
    return _to_record(
        source,
        artifact_path=artifact_path,
        tags=tags,
        extra_params=extra_params,
        extra_metrics=extra_metrics,
    )


def log_to_mlflow(
    record_or_result: object,
    *,
    run_name: str | None = None,
    tracking_uri: str | None = None,
    artifact_path: str = "lawsynth",
) -> str:
    """Log a discovery run to MLflow and return the MLflow run id.

    Logs the record's params, metrics and tags, and uploads the ``.lsworld``
    bundle under ``artifact_path``. Accepts either a :class:`RunRecord` or a live
    Study/DiscoveryResult (the record is built from it). Raises
    :class:`MissingDependencyError` when ``mlflow`` is not installed.
    """
    mlflow = _require("mlflow", "log_to_mlflow()")
    record, source = _coerce(record_or_result)
    if tracking_uri is not None:
        mlflow.set_tracking_uri(tracking_uri)
    run = mlflow.start_run(run_name=run_name or record.name)
    try:
        if record.params:
            mlflow.log_params(dict(record.params))
        if record.metrics:
            mlflow.log_metrics({key: float(value) for key, value in record.metrics.items()})
        if record.tags:
            mlflow.set_tags(dict(record.tags))
        with _artifact_file(record, source) as bundle:
            if bundle is not None:
                mlflow.log_artifact(str(bundle), artifact_path=artifact_path)
        run_id = run.info.run_id
    finally:
        mlflow.end_run()
    return str(run_id)


def log_to_wandb(
    record_or_result: object,
    *,
    project: str | None = None,
    name: str | None = None,
) -> str:
    """Log a discovery run to Weights & Biases and return the W&B run id.

    Params and tags become the run ``config``, metrics become ``summary`` values,
    and the ``.lsworld`` bundle is logged as a ``lawsynth-world`` artifact. Accepts
    a :class:`RunRecord` or a live Study/DiscoveryResult. Raises
    :class:`MissingDependencyError` when ``wandb`` is not installed.
    """
    wandb = _require("wandb", "log_to_wandb()")
    record, source = _coerce(record_or_result)
    config = {**dict(record.params), **dict(record.tags)}
    run = wandb.init(project=project, name=name or record.name, config=config)
    try:
        if record.metrics:
            run.summary.update({key: float(value) for key, value in record.metrics.items()})
        with _artifact_file(record, source) as bundle:
            if bundle is not None:
                artifact = wandb.Artifact(name=f"{_slug(record.name)}-lsworld", type="lawsynth-world")
                artifact.add_file(str(bundle))
                run.log_artifact(artifact)
        run_id = run.id
    finally:
        run.finish()
    return str(run_id)


# --------------------------------------------------------------------------- #
# Attach convenience methods to Study / DiscoveryResult (best-effort, lazy)    #
# --------------------------------------------------------------------------- #


def _install() -> None:
    try:
        from .study import DiscoveryResult, Study
    except Exception:  # pragma: no cover - defensive at import time
        return
    for target in (Study, DiscoveryResult):
        if not hasattr(target, "run_record"):
            target.run_record = lambda self, **kwargs: run_record(self, **kwargs)  # type: ignore[attr-defined]
        if not hasattr(target, "log_to_mlflow"):
            target.log_to_mlflow = lambda self, **kwargs: log_to_mlflow(self, **kwargs)  # type: ignore[attr-defined]
        if not hasattr(target, "log_to_wandb"):
            target.log_to_wandb = lambda self, **kwargs: log_to_wandb(self, **kwargs)  # type: ignore[attr-defined]


_install()
