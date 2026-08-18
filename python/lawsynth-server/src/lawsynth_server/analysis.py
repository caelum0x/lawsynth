"""Stability analysis over a stored world, backed by the LawSynth engine.

This resource mirrors :mod:`lawsynth_server.simulations` + :mod:`lawsynth_server.native`
end to end. The server persists only declarative world data, so a request first
reconstructs the native ``World`` from that record — exactly as a simulation does
(:func:`lawsynth_server.native.build_world`) — and then reaches the engine's
stability analysis. Fixed-point location is **not** re-implemented here: the
native ``World`` has no stability method, so the world is saved to a temporary
``.lsworld`` bundle and analysed through the CLI-backed
``lawsynth.analysis.stability`` (the compiled engine is the single source of truth
for the numerics, mirroring the SDK's own ``Study.stability`` convenience method).

The engine's honest outputs are surfaced verbatim:

* an empty ``fixed_points`` (no equilibrium located inside the search box) is a
  valid result with the seed accounting, **not** an error;
* a non-hyperbolic (center / marginal) point keeps its ``inconclusive`` flag;
* a malformed request (missing box, non-numeric interval, or a box whose
  dimension does not match the world's state count) is a clear ``422``;
* the optional engine's absence (native extension or built CLI) becomes a typed
  ``503`` and an engine failure a typed ``500`` — never a raw traceback.
"""

from __future__ import annotations

import tempfile
from collections.abc import Mapping, Sequence
from math import isfinite
from pathlib import Path
from typing import Any

from .errors import NativeUnavailableError, ServerError, ValidationError
from .native import build_world, world_spec

# The optional stability knobs map one-to-one onto ``lawsynth.analysis.stability``
# keyword arguments; each is validated as a positive, finite number of the noted
# kind so a bad value is a 422 before any engine work is attempted.
_INT_KNOBS = ("grid", "max_iterations")
_FLOAT_KNOBS = ("tolerance", "dedup", "marginal_band", "divergence")


def _interval(value: object, index: int) -> tuple[float, float]:
    """Validate one ``LOW:HIGH`` search interval into a ``(low, high)`` pair."""
    if isinstance(value, str):
        parts = value.split(":")
        if len(parts) != 2:
            raise ValidationError(f"box interval {index} must be formatted as LOW:HIGH")
        raw_low: object
        raw_high: object
        raw_low, raw_high = parts
    elif isinstance(value, Sequence) and not isinstance(value, (str, bytes)) and len(value) == 2:
        raw_low, raw_high = value[0], value[1]
    else:
        raise ValidationError(f"box interval {index} must be a [low, high] pair or a LOW:HIGH string")
    try:
        low, high = float(raw_low), float(raw_high)  # type: ignore[arg-type]
    except (TypeError, ValueError) as error:
        raise ValidationError(f"box interval {index} bounds must be finite numbers") from error
    if isinstance(raw_low, bool) or isinstance(raw_high, bool) or not isfinite(low) or not isfinite(high):
        raise ValidationError(f"box interval {index} bounds must be finite numbers")
    if low > high:
        raise ValidationError(f"box interval {index} has lower bound {low} above upper bound {high}")
    return low, high


def _parse_box(value: object) -> list[tuple[float, float]]:
    """Normalize the search box into a non-empty list of ``(low, high)`` intervals.

    Accepts either the raw ``"LOW:HIGH,LOW:HIGH"`` string the engine's CLI speaks
    (one interval per state, in state order) or a list of ``[low, high]`` pairs.
    """
    if isinstance(value, str):
        text = value.strip()
        if not text:
            raise ValidationError("box must not be empty")
        entries: Sequence[object] = [part.strip() for part in text.split(",")]
    elif isinstance(value, Sequence) and not isinstance(value, (str, bytes)):
        entries = list(value)
    else:
        raise ValidationError("box must be a list of [low, high] intervals or a LOW:HIGH string")
    if not entries:
        raise ValidationError("box must contain at least one LOW:HIGH interval per state")
    return [_interval(entry, index) for index, entry in enumerate(entries)]


def _knobs(spec: Mapping[str, object]) -> dict[str, float | int]:
    """Validate the optional stability knobs, returning only the ones supplied."""
    result: dict[str, float | int] = {}
    for field in _INT_KNOBS:
        if field not in spec:
            continue
        value = spec[field]
        if isinstance(value, bool) or not isinstance(value, int) or value <= 0:
            raise ValidationError(f"{field} must be a positive integer")
        result[field] = value
    for field in _FLOAT_KNOBS:
        if field not in spec:
            continue
        value = spec[field]
        if isinstance(value, bool) or not isinstance(value, (int, float)) or not isfinite(float(value)) or value <= 0:
            raise ValidationError(f"{field} must be a positive finite number")
        result[field] = float(value)
    return result


def validate_stability_request(spec: object) -> dict[str, object]:
    """Validate a stability request body and normalize it for the engine.

    Only the request shape is checked here (mirroring
    :func:`lawsynth_server.simulations.validate_simulation_spec`); the box's
    dimension is checked against the world's state count in
    :func:`analyze_stability`, where the world record is available.
    """
    if not isinstance(spec, Mapping):
        raise ValidationError("stability request must be an object")
    if "box" not in spec:
        raise ValidationError("stability requires a search box (one LOW:HIGH interval per state)")
    unknown = set(spec) - ({"box"} | set(_INT_KNOBS) | set(_FLOAT_KNOBS))
    if unknown:
        raise ValidationError("unknown stability options", details={"fields": sorted(unknown)})
    return {"box": _parse_box(spec["box"]), **_knobs(spec)}


def _analysis_module() -> Any:
    """Import the CLI-backed analysis SDK at the operation boundary.

    Mirrors :func:`lawsynth_server.native._native_module`: an import failure or a
    source-only install (where the lazy ``World`` attribute cannot resolve) is
    reported as an operationally useful ``503`` rather than an internal error.
    """
    try:
        import lawsynth

        _ = lawsynth.World
        from lawsynth import analysis
    except (ImportError, AttributeError) as error:
        raise NativeUnavailableError(
            "the LawSynth native runtime is unavailable; install the built lawsynth package"
        ) from error
    except Exception as error:
        if error.__class__.__name__ == "NativeError":
            raise NativeUnavailableError(
                "the LawSynth native runtime is unavailable; install the built lawsynth package"
            ) from error
        raise
    return analysis


def _report_to_json(report: Any, states: Sequence[str]) -> dict[str, object]:
    """Render a native :class:`StabilityReport` into a JSON-safe response body."""
    return {
        "world": str(report.world),
        "states": list(report.states),
        "seeds_total": int(report.seeds_total),
        "seeds_converged": int(report.seeds_converged),
        "fixed_points": [
            {
                "coordinates": list(point.coordinates),
                "state_values": point.at(report.states),
                "classification": str(point.classification),
                "inconclusive": bool(point.inconclusive),
                "eigenvalues": [{"re": float(eig.re), "im": float(eig.im)} for eig in point.eigenvalues],
            }
            for point in report.fixed_points
        ],
    }


def analyze_stability(world_record: Mapping[str, object], request: Mapping[str, object]) -> dict[str, object]:
    """Locate and classify the fixed points of a stored world via the engine.

    Rebuilds the native ``World`` from ``world_record`` (as a simulation does),
    checks the search box against the world's state count, saves the world to a
    temporary ``.lsworld`` bundle, and runs the CLI-backed
    ``lawsynth.analysis.stability``. Engine absence and failures are mapped onto
    the server's typed errors; the engine's fixed points are returned verbatim.
    """
    spec = world_spec(world_record)
    states = spec["states"]
    box = request["box"]
    if not isinstance(box, list):  # pragma: no cover - validate_stability_request guarantees this
        raise ValidationError("box must be a list of intervals")
    if len(box) != len(states):
        raise ValidationError(
            "box dimension must match the world's state count: "
            f"the world has {len(states)} states ({', '.join(states)}) but the box has {len(box)} intervals"
        )
    knobs = {field: request[field] for field in (*_INT_KNOBS, *_FLOAT_KNOBS) if field in request}

    analysis = _analysis_module()
    world = build_world(world_record)
    with tempfile.TemporaryDirectory(prefix="lawsynth-stability-") as directory:
        bundle = Path(directory) / "world.lsworld"
        try:
            world.save(str(bundle))
        except Exception as error:  # pragma: no cover - native save is exercised via the live path
            raise ServerError(f"failed to persist the world for analysis: {error}") from error
        try:
            report = analysis.stability(bundle, box=box, **knobs)
        except analysis.MissingBinaryError as error:
            raise NativeUnavailableError(
                "the LawSynth native runtime is unavailable; install the built lawsynth package"
            ) from error
        except analysis.CliError as error:
            raise ServerError(f"stability analysis failed: {error.stderr or 'engine error'}") from error
        except analysis.AnalysisError as error:
            raise ServerError(f"stability analysis produced an unreadable result: {error}") from error
    return _report_to_json(report, states)
