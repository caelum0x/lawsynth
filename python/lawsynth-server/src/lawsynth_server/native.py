"""Boundary between persisted service records and the native LawSynth engine.

The server deliberately persists only declarative data.  A request reconstructs
the native world from that data, so a service restart cannot leave an opaque
Python object in metadata and every simulation uses the same executable model
that the Python SDK exposes.
"""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from math import isfinite
from typing import Any

from .errors import NativeUnavailableError, ValidationError


def _native_module() -> Any:
    """Import the SDK only at the operation boundary.

    The server package remains usable for metadata and health endpoints where
    the optional compiled extension is intentionally absent.  Import failures
    are converted into an operationally useful 503 instead of an internal
    server error.
    """
    try:
        import lawsynth

        # Resolve the lazy SDK attribute now so a source-only installation is
        # reported before a run is recorded as successful.
        _ = lawsynth.World
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
    return lawsynth


def _number_map(value: object, field: str) -> dict[str, float]:
    if value is None:
        return {}
    if not isinstance(value, Mapping):
        raise ValidationError(f"{field} must be an object mapping identifiers to finite numbers")
    result: dict[str, float] = {}
    for name, number in value.items():
        if not isinstance(name, str) or not name.isidentifier():
            raise ValidationError(f"{field} keys must be identifiers")
        if isinstance(number, bool) or not isinstance(number, (int, float)) or not isfinite(float(number)):
            raise ValidationError(f"{field} values must be finite numbers")
        result[name] = float(number)
    return result


def _identifiers(value: object, field: str) -> list[str]:
    if not isinstance(value, Sequence) or isinstance(value, (str, bytes)) or not value:
        raise ValidationError(f"{field} must be a non-empty list of identifiers")
    identifiers = list(value)
    if any(not isinstance(name, str) or not name.isidentifier() for name in identifiers):
        raise ValidationError(f"{field} must contain identifiers")
    if len(set(identifiers)) != len(identifiers):
        raise ValidationError(f"{field} cannot contain duplicates")
    return identifiers


def _equations(value: object, states: Sequence[str]) -> dict[str, str]:
    if not isinstance(value, Mapping):
        raise ValidationError("world equations must map every state to an expression")
    equations = dict(value)
    if set(equations) != set(states) or any(
        not isinstance(target, str) or not isinstance(expression, str) or not expression.strip()
        for target, expression in equations.items()
    ):
        raise ValidationError("world equations must contain one non-empty expression for every state")
    return equations


def world_spec(values: Mapping[str, object]) -> dict[str, object]:
    """Normalize and validate the declarative subset accepted by native World."""
    states = _identifiers(values.get("states"), "states")
    controls_raw = values.get("controls", [])
    if not isinstance(controls_raw, Sequence) or isinstance(controls_raw, (str, bytes)):
        raise ValidationError("controls must be a list of identifiers")
    controls = list(controls_raw)
    if any(not isinstance(name, str) or not name.isidentifier() for name in controls):
        raise ValidationError("controls must contain identifiers")
    if len(set(controls)) != len(controls) or set(states) & set(controls):
        raise ValidationError("states and controls must be distinct")
    parameters = _number_map(values.get("parameters"), "parameters")
    if (set(states) | set(controls)) & set(parameters):
        raise ValidationError("states, controls, and parameters must use distinct identifiers")
    return {
        "states": states,
        "controls": controls,
        "parameters": parameters,
        "equations": _equations(values.get("equations"), states),
    }


def build_world(values: Mapping[str, object]) -> object:
    """Construct the real PyO3 World from a stored declarative world record."""
    spec = world_spec(values)
    lawsynth = _native_module()
    try:
        return lawsynth.World(spec["states"], spec["parameters"], spec["equations"], spec["controls"])
    except Exception as error:
        raise ValidationError(f"native world validation failed: {error}") from error


def discover_world(dataset: Mapping[str, object], states: object, config: object) -> tuple[object, dict[str, object]]:
    """Run native discovery on a stored numeric dataset and return its world spec."""
    time = dataset.get("time")
    columns = dataset.get("columns")
    selected_states = _identifiers(states, "states")
    if not isinstance(time, list) or not isinstance(columns, Mapping):
        raise ValidationError("dataset does not contain observations required for discovery")
    if any(state not in columns for state in selected_states):
        raise ValidationError("discovery states must be dataset columns")
    if config is None:
        config = {}
    if not isinstance(config, Mapping):
        raise ValidationError("discovery must be an object")
    allowed = {
        "polynomial_degree", "threshold", "solver", "include_trigonometric", "include_rational",
        "smoothing_radius", "derivative_method", "savgol_window", "tvreg_lambda",
        "tvreg_iterations", "symbolic_depth",
    }
    unknown = set(config) - allowed
    if unknown:
        raise ValidationError("unknown discovery options", details={"fields": sorted(unknown)})
    lawsynth = _native_module()
    try:
        world = lawsynth.discover(time, columns, state=selected_states, **dict(config))
        equations = world.equations()
    except Exception as error:
        raise ValidationError(f"native discovery failed: {error}") from error
    return world, {
        "states": selected_states,
        "controls": [],
        "parameters": {},
        "equations": dict(equations),
    }


def simulate_world(world_record: Mapping[str, object], simulation: Mapping[str, object]) -> dict[str, object]:
    """Execute a stored World and return JSON-safe trajectory values."""
    world = build_world(world_record)
    initial = _number_map(simulation.get("initial"), "initial")
    if not initial:
        raise ValidationError("simulation initial must provide at least one state value")
    parameters = _number_map(simulation.get("parameters"), "parameters")
    inputs = _number_map(simulation.get("inputs"), "inputs")
    try:
        trajectory = world.simulate(
            initial,
            start=float(simulation.get("start", 0.0)),
            end=float(simulation["horizon"]),
            step=float(simulation["step"]),
            parameters=parameters,
            inputs=inputs,
        )
    except Exception as error:
        raise ValidationError(f"native simulation failed: {error}") from error
    return {
        "time": list(trajectory.time),
        "values": {name: list(values) for name, values in trajectory.values.items()},
    }
