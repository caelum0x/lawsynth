"""Discovery-as-a-service: the run workflow that turns a dataset into a world.

This module owns the *execution* half of the ``runs`` resource.  The domain
(:mod:`lawsynth_server`) persists runs, datasets, and worlds and enforces tenant
isolation; the transport (:mod:`app`) authenticates, scopes, and frames.  What
lives here is the honest, in-process orchestration that makes a run *real*:

Workflow (states)
    ``POST /v1/runs`` with a dataset reference creates a run in ``queued`` and
    returns immediately.  A daemon worker thread then transitions the run
    ``queued -> running -> succeeded|failed``: it runs native discovery, stores
    the resulting world through the domain world repository, computes a result
    summary (mse, complexity, law count, world id) and records it on the run.
    Every transition is published to the :class:`EventBus` as an ``ApiEvent``
    (``run_queued``/``run_started``/``run_succeeded``/``run_failed``) so
    ``GET /v1/events`` streams the lifecycle.

Honest boundaries
    Discovery runs the *native* engine in this process -- there is no distributed
    scheduler and none is pretended.  If the compiled runtime is absent the
    submit is rejected with ``503 native_unavailable`` before a run is recorded,
    exactly like the simulate-backed product paths.  A run that fails at
    execution time records ``failed`` with a clear reason instead of a
    fabricated world.

The dataset may be an already-uploaded ``dataset_id`` or an inline dataset
(structured ``time``/``columns`` or a ``csv`` string); inline data is
materialised through the same validating domain dataset repository so it is
tenant-isolated and reproducible.
"""

from __future__ import annotations

import csv
import io
import json
import math
import time as _time
from dataclasses import dataclass
from threading import RLock, Thread
from typing import Mapping, Sequence
from uuid import uuid4

from lawsynth_server.dependencies import Services
from lawsynth_server.errors import ConflictError, NativeUnavailableError, ServerError, ValidationError
from lawsynth_server.native import discover_world, simulate_world

from . import laws
from .events import EventBus, EventKind

# Discovery knobs accepted by ``lawsynth_server.native.discover_world`` (which
# validates against the same set); this module normalises friendly aliases and a
# ``recipe`` preset down to these before forwarding.
_ALLOWED_CONFIG = frozenset(
    {
        "polynomial_degree",
        "threshold",
        "solver",
        "include_trigonometric",
        "include_rational",
        "smoothing_radius",
        "derivative_method",
        "savgol_window",
        "tvreg_lambda",
        "tvreg_iterations",
        "symbolic_depth",
    }
)
_CONFIG_ALIASES = {"degree": "polynomial_degree"}


def native_available() -> bool:
    """True when the compiled LawSynth runtime can be imported and used.

    Mirrors the probe used by the simulate-backed product paths: a source-only
    install resolves ``import lawsynth`` but not ``lawsynth.World``.
    """

    try:
        import lawsynth

        _ = lawsynth.World
        return True
    except Exception:
        return False


@dataclass(frozen=True, slots=True)
class _Plan:
    """A validated, side-effect-free discovery plan produced at submit time."""

    run_name: str
    world_name: str
    project_id: str | None
    states: tuple[str, ...]
    config: Mapping[str, object]
    dataset_ref: str | None
    inline_name: str | None
    inline_schema: tuple[str, ...] | None
    inline_time: tuple[float, ...] | None
    inline_columns: Mapping[str, list[float]] | None


class DiscoveryService:
    """Owns discovery-run submission, execution, and world retrieval."""

    def __init__(self, services: Services, events: EventBus) -> None:
        self._services = services
        self._events = events
        self._lock = RLock()
        self._threads: list[Thread] = []

    # -- lifecycle ---------------------------------------------------------- #

    def close(self) -> None:
        """Join outstanding worker threads so shutdown is orderly (best effort)."""

        with self._lock:
            threads = list(self._threads)
        for thread in threads:
            thread.join(timeout=5)

    # -- submit ------------------------------------------------------------- #

    def submit(self, organization_id: str, body: object, idempotency_key: str, request_id: str) -> dict[str, object]:
        """Validate a discovery request, record a ``queued`` run, and kick it off.

        Validation (and dataset-reference resolution) happens synchronously so a
        bad request is a ``422``/``404`` at submit.  The record-and-launch step
        runs under idempotency so a retried key returns the same queued run
        without starting a second worker.
        """

        plan = self._parse(organization_id, body)
        if not native_available():
            raise NativeUnavailableError(
                "the LawSynth native runtime is unavailable; install the built lawsynth package"
            )

        def create() -> tuple[int, dict[str, object]]:
            dataset_record, dataset_id = self._materialize_dataset(organization_id, plan)
            values: dict[str, object] = {
                "name": plan.run_name,
                "status": "queued",
                "dataset_id": dataset_id,
                "metadata": self._metadata(plan, phase="queued"),
            }
            if plan.project_id is not None:
                values["project_id"] = plan.project_id
            run = self._services.runs.create(organization_id, values)
            run_id = str(run["id"])
            self._emit(organization_id, EventKind.RUN_QUEUED, run_id, {"id": run_id, "status": "queued"})
            self._start_worker(organization_id, run_id, plan, dataset_record)
            return 201, run

        status, run, replayed = self._services.idempotency.execute(
            organization_id,
            idempotency_key,
            {"method": "POST", "path": "/v1/runs", "body": body},
            create,
        )
        return {
            "status": status,
            "headers": {"X-Request-ID": request_id, "Idempotency-Replayed": str(replayed).lower()},
            "body": run,
        }

    # -- run world retrieval ----------------------------------------------- #

    def run_world(self, organization_id: str, run_id: str, request_id: str) -> dict[str, object]:
        """Return the world a completed run discovered, with product links.

        ``404`` when the run is unknown, ``409`` when the run has not produced a
        world yet (still queued/running) or failed without one.
        """

        run = self._services.runs.get(organization_id, run_id)
        status = run.get("status")
        world_id = run.get("world_id")
        if not isinstance(world_id, str) or not world_id:
            if status in {"queued", "running"}:
                raise ConflictError(f"run has not produced a world yet (status={status})")
            reason = ""
            metadata = run.get("metadata")
            if isinstance(metadata, Mapping) and isinstance(metadata.get("error"), str):
                reason = f": {metadata['error']}"
            raise ConflictError(f"run did not produce a world (status={status}){reason}")
        world = self._services.worlds.get(organization_id, world_id)
        return {
            "status": 200,
            "headers": {"X-Request-ID": request_id},
            "body": {
                "run_id": run_id,
                "world_id": world_id,
                "world": world,
                "links": {
                    "self": f"/v1/worlds/{world_id}",
                    "explain": f"/v1/worlds/{world_id}/explain",
                    "report": f"/v1/worlds/{world_id}/report",
                },
            },
        }

    # -- parsing / validation (no side effects) ---------------------------- #

    def _parse(self, organization_id: str, body: object) -> _Plan:
        if not isinstance(body, Mapping):
            raise ValidationError("discovery run body must be an object")
        states = self._states(body.get("states"))
        config = _normalize_config(body.get("discovery"))
        run_name = self._optional_name(body.get("name"), "name") or f"discovery-{uuid4().hex[:8]}"
        world_name = self._optional_name(body.get("world_name"), "world_name") or f"{run_name}-world-{uuid4().hex[:6]}"

        project_id = body.get("project_id")
        if project_id is not None:
            if not isinstance(project_id, str):
                raise ValidationError("project_id must be a string")
            self._services.projects.get(organization_id, project_id)

        dataset_ref = body.get("dataset_id")
        inline = body.get("dataset")
        if dataset_ref is not None and inline is not None:
            raise ValidationError("provide either dataset_id or an inline dataset, not both")

        if dataset_ref is not None:
            if not isinstance(dataset_ref, str):
                raise ValidationError("dataset_id must be a string")
            record = self._services.datasets.get(organization_id, dataset_ref)
            self._verify_states_present(record.get("columns"), states)
            return _Plan(run_name, world_name, project_id, states, config, dataset_ref, None, None, None, None)

        if inline is None:
            raise ValidationError("a discovery run requires a dataset_id or an inline dataset")
        itime, icolumns, iname = self._parse_inline(inline)
        self._verify_states_present(icolumns, states)
        schema = tuple(sorted(icolumns))
        return _Plan(run_name, world_name, project_id, states, config, None, iname, schema, itime, icolumns)

    @staticmethod
    def _states(value: object) -> tuple[str, ...]:
        if not isinstance(value, Sequence) or isinstance(value, (str, bytes)) or not value:
            raise ValidationError("states must be a non-empty list of identifiers")
        states = list(value)
        if any(not isinstance(name, str) or not name.isidentifier() for name in states):
            raise ValidationError("states must contain identifiers")
        if len(set(states)) != len(states):
            raise ValidationError("states cannot contain duplicates")
        return tuple(states)

    @staticmethod
    def _optional_name(value: object, field: str) -> str | None:
        if value is None:
            return None
        if not isinstance(value, str) or not value.strip():
            raise ValidationError(f"{field} must be a non-empty string")
        return value.strip()

    @staticmethod
    def _verify_states_present(columns: object, states: Sequence[str]) -> None:
        if not isinstance(columns, Mapping):
            raise ValidationError("dataset has no observations to discover from")
        missing = [name for name in states if name not in columns]
        if missing:
            raise ValidationError("discovery states must be dataset columns", details={"fields": sorted(missing)})

    def _parse_inline(self, inline: object) -> tuple[tuple[float, ...], dict[str, list[float]], str]:
        if not isinstance(inline, Mapping):
            raise ValidationError("inline dataset must be an object")
        name = self._optional_name(inline.get("name"), "dataset name") or f"run-dataset-{uuid4().hex[:8]}"
        if "csv" in inline:
            itime, columns = self._from_csv(inline)
        elif "time" in inline or "columns" in inline:
            time_value, columns_value = inline.get("time"), inline.get("columns")
            if not isinstance(time_value, list) or not isinstance(columns_value, Mapping):
                raise ValidationError("inline dataset requires a 'time' list and a 'columns' object")
            itime = tuple(time_value)  # type: ignore[arg-type]
            columns = {str(key): list(series) for key, series in columns_value.items()}  # type: ignore[arg-type]
        else:
            raise ValidationError("inline dataset must provide 'csv' or 'time' and 'columns'")
        if not columns:
            raise ValidationError("inline dataset must contain at least one observation column")
        return itime, columns, name

    @staticmethod
    def _from_csv(inline: Mapping[str, object]) -> tuple[tuple[float, ...], dict[str, list[float]]]:
        text = inline.get("csv")
        if not isinstance(text, str) or not text.strip():
            raise ValidationError("inline csv must be a non-empty string")
        rows = [row for row in csv.reader(io.StringIO(text)) if any(cell.strip() for cell in row)]
        if len(rows) < 3:
            raise ValidationError("inline csv needs a header row and at least two observations")
        header = [cell.strip() for cell in rows[0]]
        if any(not name for name in header) or len(set(header)) != len(header):
            raise ValidationError("inline csv header must have unique, non-empty column names")
        requested = inline.get("time_column")
        if requested is not None and not isinstance(requested, str):
            raise ValidationError("time_column must be a string")
        time_column = requested or ("t" if "t" in header else "time" if "time" in header else header[0])
        if time_column not in header:
            raise ValidationError(f"csv time column {time_column!r} is not present")
        index = {name: position for position, name in enumerate(header)}

        def column(name: str) -> list[float]:
            values: list[float] = []
            for row in rows[1:]:
                if len(row) != len(header):
                    raise ValidationError("every inline csv row must match the header width")
                try:
                    values.append(float(row[index[name]]))
                except ValueError as error:
                    raise ValidationError(f"csv value in column {name!r} is not a number") from error
            return values

        itime = tuple(column(time_column))
        columns = {name: column(name) for name in header if name != time_column}
        if not columns:
            raise ValidationError("inline csv must have at least one non-time column")
        return itime, columns

    # -- dataset materialisation (side-effecting; inside idempotency) ------- #

    def _materialize_dataset(self, organization_id: str, plan: _Plan) -> tuple[dict[str, object], str]:
        if plan.dataset_ref is not None:
            record = self._services.datasets.get(organization_id, plan.dataset_ref)
            return record, plan.dataset_ref
        record = self._services.datasets.create(
            organization_id,
            {
                "name": plan.inline_name,
                "schema": list(plan.inline_schema or ()),
                "time": list(plan.inline_time or ()),
                "columns": {name: list(series) for name, series in (plan.inline_columns or {}).items()},
            },
        )
        return record, str(record["id"])

    # -- execution (worker thread) ----------------------------------------- #

    def _start_worker(self, organization_id: str, run_id: str, plan: _Plan, dataset_record: Mapping[str, object]) -> None:
        thread = Thread(
            target=self._execute,
            args=(organization_id, run_id, plan, dict(dataset_record)),
            name=f"discovery-{run_id}",
            daemon=True,
        )
        with self._lock:
            self._threads.append(thread)
        thread.start()

    def _execute(self, organization_id: str, run_id: str, plan: _Plan, dataset_record: Mapping[str, object]) -> None:
        try:
            self._services.runs.update(
                organization_id, run_id, {"status": "running", "metadata": self._metadata(plan, phase="running")}
            )
            self._emit(organization_id, EventKind.RUN_STARTED, run_id, {"id": run_id, "status": "running"})

            _, spec = discover_world(dataset_record, list(plan.states), dict(plan.config))
            world = self._services.worlds.create(
                organization_id,
                {
                    "name": plan.world_name,
                    "project_id": plan.project_id,
                    "dataset_id": dataset_record.get("id"),
                    **spec,
                },
            )
            world_id = str(world["id"])
            summary = self._summarize(world, dataset_record, plan.states)
            self._services.runs.update(
                organization_id,
                run_id,
                {
                    "status": "succeeded",
                    "world_id": world_id,
                    "metadata": self._metadata(plan, phase="succeeded", summary=summary),
                },
            )
            self._emit(
                organization_id,
                EventKind.RUN_SUCCEEDED,
                run_id,
                {"id": run_id, "status": "succeeded", "world_id": world_id},
            )
        except NativeUnavailableError as error:
            self._fail(organization_id, run_id, plan, f"native runtime unavailable: {error.message}")
        except ServerError as error:
            self._fail(organization_id, run_id, plan, error.message)
        except Exception as error:  # pragma: no cover - defensive; never leak a worker crash
            self._fail(organization_id, run_id, plan, f"discovery failed: {error}")

    def _fail(self, organization_id: str, run_id: str, plan: _Plan, reason: str) -> None:
        try:
            self._services.runs.update(
                organization_id,
                run_id,
                {"status": "failed", "metadata": self._metadata(plan, phase="failed", error=reason)},
            )
        except Exception:
            pass
        self._emit(
            organization_id,
            EventKind.RUN_FAILED,
            run_id,
            {"id": run_id, "status": "failed", "reason": reason[:500]},
        )

    # -- result summary ----------------------------------------------------- #

    def _summarize(self, world: Mapping[str, object], dataset: Mapping[str, object], states: Sequence[str]) -> dict[str, object]:
        equations = world.get("equations")
        equations = dict(equations) if isinstance(equations, Mapping) else {}
        read = laws.read_laws(equations)
        return {
            "world_id": world.get("id"),
            "laws": len(read),
            "complexity": {"laws": len(read), "total_terms": laws.total_terms(read)},
            "mse": self._fit_mse(world, dataset, states),
            "equations": equations,
        }

    @staticmethod
    def _fit_mse(world: Mapping[str, object], dataset: Mapping[str, object], states: Sequence[str]) -> float | None:
        """In-sample mean squared error from re-simulating the discovered world.

        Honest fit indicator: simulate from the first observation across the
        observed window and compare index-wise.  Returns ``None`` (never a
        fabricated number) if the world cannot be simulated or the result is not
        finite.
        """

        observations = dataset.get("columns")
        times = dataset.get("time")
        if not isinstance(observations, Mapping) or not isinstance(times, list) or len(times) < 2:
            return None
        try:
            deltas = sorted(float(times[i + 1]) - float(times[i]) for i in range(len(times) - 1))
            step = deltas[len(deltas) // 2]
            if step <= 0:
                return None
            initial = {name: float(observations[name][0]) for name in states if name in observations}
            if not initial:
                return None
            trajectory = simulate_world(
                world,
                {"initial": initial, "start": float(times[0]), "horizon": float(times[-1]), "step": step},
            )
            simulated = trajectory.get("values", {})
            total, count = 0.0, 0
            for name in states:
                observed = observations.get(name)
                series = simulated.get(name) if isinstance(simulated, Mapping) else None
                if not isinstance(observed, list) or not isinstance(series, list):
                    continue
                for index in range(min(len(observed), len(series))):
                    difference = float(observed[index]) - float(series[index])
                    total += difference * difference
                    count += 1
            if count == 0:
                return None
            mse = total / count
            return mse if math.isfinite(mse) else None
        except Exception:
            return None

    # -- shared helpers ----------------------------------------------------- #

    @staticmethod
    def _metadata(plan: _Plan, *, phase: str, summary: Mapping[str, object] | None = None, error: str | None = None) -> dict[str, object]:
        metadata: dict[str, object] = {
            "kind": "discovery",
            "phase": phase,
            "states": list(plan.states),
            "config": dict(plan.config),
        }
        if summary is not None:
            metadata["summary"] = dict(summary)
        if error is not None:
            metadata["error"] = error
        return metadata

    def _emit(self, organization_id: str, kind: EventKind, run_id: str, payload: Mapping[str, object]) -> None:
        try:
            self._events.append(
                organization_id,
                int(_time.time() * 1000),
                kind,
                json.dumps(payload, separators=(",", ":"), allow_nan=False),
                run_id=run_id,
            )
        except Exception:
            pass


def _normalize_config(raw: object) -> dict[str, object]:
    """Normalise a discovery config: apply a ``recipe`` preset and aliases.

    ``recipe``/``preset`` selects a curated base config from
    :mod:`lawsynth.recipes`; explicit fields override it.  Friendly aliases
    (``degree`` -> ``polynomial_degree``) are expanded, and the result is
    restricted to the options the native engine accepts.
    """

    if raw is None:
        return {}
    if not isinstance(raw, Mapping):
        raise ValidationError("discovery config must be an object")
    data = dict(raw)

    recipe = data.pop("recipe", None)
    preset = data.pop("preset", None)
    if recipe is None:
        recipe = preset
    elif preset is not None:
        raise ValidationError("specify only one of 'recipe' or 'preset'")

    for alias, canonical in _CONFIG_ALIASES.items():
        if alias in data:
            if canonical in data:
                raise ValidationError(f"specify only one of {alias!r} or {canonical!r}")
            data[canonical] = data.pop(alias)

    unknown = set(data) - _ALLOWED_CONFIG
    if unknown:
        raise ValidationError("unknown discovery options", details={"fields": sorted(unknown)})

    base: dict[str, object] = {}
    if recipe is not None:
        base = _recipe_settings(recipe)
    merged = {**base, **data}
    unknown = set(merged) - _ALLOWED_CONFIG
    if unknown:
        raise ValidationError("unknown discovery options", details={"fields": sorted(unknown)})
    return merged


def _recipe_settings(name: object) -> dict[str, object]:
    if not isinstance(name, str) or not name.strip():
        raise ValidationError("recipe must be a non-empty string")
    try:
        from lawsynth import recipes
    except Exception as error:  # pragma: no cover - recipes ship with the SDK
        raise ValidationError("discovery recipes are unavailable in this install") from error
    try:
        recipe = recipes.get(name)
    except Exception as error:
        raise ValidationError(f"unknown discovery recipe {name!r}") from error
    return dict(recipe.settings)
