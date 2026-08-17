"""LawSynth product features exposed over the ``/v1/worlds`` HTTP surface.

These are the transport analogs of the SDK/CLI product loop -- ``explain``,
``forecast``, ``report`` and ``compare`` -- backed by the same stored world
records and the same native engine the ``simulate`` action uses.  The module
owns only the product logic and route recognition; authentication, tenant
scoping, and response framing stay in :mod:`app`.

Capability boundaries are honest:

* ``explain``/``report``/``compare`` read the world's *declarative* structure
  (states, parameters, equations) and never touch the native runtime, so they
  work offline.
* ``forecast`` is simulate-backed: it calls the very same
  :func:`lawsynth_server.native.simulate_world` used by ``/simulate`` and
  therefore returns ``503 native_unavailable`` when the compiled engine is
  absent, exactly like the ``simulate`` path.
"""

from __future__ import annotations

from typing import Mapping, Sequence

from lawsynth.report import build_report
from lawsynth_server.errors import ValidationError
from lawsynth_server.native import simulate_world

from . import laws

SEGMENT = "worlds"

# Assumptions that hold for any stored world independent of a dataset.  Dataset
# dependent claims (fit quality, extrapolation validity) are deliberately
# omitted here because a stored world does not carry its observations.
_ASSUMPTIONS: tuple[str, ...] = (
    "Continuous-time dynamics: each law models a first derivative (dX/dt).",
    "Deterministic and offline -- identical inputs reproduce this world exactly.",
    "Parameters are constant unless overridden by a scheduled intervention.",
    "Only the declarative world structure is described here; goodness-of-fit "
    "against observations requires the originating dataset.",
)


# --------------------------------------------------------------------------- #
# Route recognition                                                            #
# --------------------------------------------------------------------------- #


def match(method: str, parts: Sequence[str]) -> str | None:
    """Return the product action a request addresses, or ``None``.

    Recognized shapes (all under the ``worlds`` segment):
    ``GET  worlds/{id}/explain``  -> ``"explain"``
    ``POST worlds/{id}/forecast`` -> ``"forecast"``
    ``GET  worlds/{id}/report``   -> ``"report"``
    ``POST worlds/compare``       -> ``"compare"``
    """

    if not parts or parts[0] != SEGMENT:
        return None
    if len(parts) == 3:
        action = parts[2]
        if method == "GET" and action == "explain":
            return "explain"
        if method == "POST" and action == "forecast":
            return "forecast"
        if method == "GET" and action == "report":
            return "report"
        return None
    if len(parts) == 2 and method == "POST" and parts[1] == "compare":
        return "compare"
    return None


def classify(method: str, parts: Sequence[str]) -> str | None:
    """Return a stable telemetry label for a product route, or ``None``."""

    action = match(method, parts)
    return f"worlds.{action}" if action is not None else None


# --------------------------------------------------------------------------- #
# Declarative world view                                                       #
# --------------------------------------------------------------------------- #


def _states(record: Mapping[str, object]) -> list[str]:
    value = record.get("states")
    return [str(name) for name in value] if isinstance(value, (list, tuple)) else []


def _controls(record: Mapping[str, object]) -> list[str]:
    value = record.get("controls")
    return [str(name) for name in value] if isinstance(value, (list, tuple)) else []


def _parameters(record: Mapping[str, object]) -> dict[str, float]:
    value = record.get("parameters")
    if not isinstance(value, Mapping):
        return {}
    result: dict[str, float] = {}
    for name, number in value.items():
        if isinstance(number, bool) or not isinstance(number, (int, float)):
            continue
        result[str(name)] = float(number)
    return result


def _equations(record: Mapping[str, object]) -> dict[str, str]:
    """Return the world's ``target -> expression`` laws in a uniform dict form."""

    value = record.get("equations")
    if isinstance(value, Mapping):
        return {str(target): str(expr) for target, expr in value.items()}
    if isinstance(value, (list, tuple)):
        states = _states(record)
        if len(states) == len(value):
            return {str(states[i]): str(expr) for i, expr in enumerate(value)}
        return {f"law_{i}": str(expr) for i, expr in enumerate(value)}
    return {}


# --------------------------------------------------------------------------- #
# 1. explain                                                                   #
# --------------------------------------------------------------------------- #


def explain(record: Mapping[str, object]) -> dict[str, object]:
    """Structured, plain-language explanation of a stored world."""

    states = _states(record)
    controls = _controls(record)
    parameters = _parameters(record)
    equations = _equations(record)
    read = laws.read_laws(equations)
    deps = laws.dependencies(read, states)
    terms_per_law = {str(law["target"]): len(law["terms"]) for law in read}  # type: ignore[arg-type]
    return {
        "id": record.get("id"),
        "name": record.get("name"),
        "variables": states,
        "controls": controls,
        "parameters": parameters,
        "laws": read,
        "dependencies": deps,
        "complexity": {
            "laws": len(read),
            "parameters": len(parameters),
            "controls": len(controls),
            "total_terms": laws.total_terms(read),
            "terms_per_law": terms_per_law,
        },
        "assumptions": list(_ASSUMPTIONS),
    }


# --------------------------------------------------------------------------- #
# 2. forecast (simulate-backed)                                                #
# --------------------------------------------------------------------------- #


def _number_map(value: object, field: str) -> dict[str, float]:
    if value is None:
        return {}
    if not isinstance(value, Mapping):
        raise ValidationError(f"{field} must be an object mapping identifiers to numbers")
    result: dict[str, float] = {}
    for name, number in value.items():
        if not isinstance(name, str) or not name.isidentifier():
            raise ValidationError(f"{field} keys must be identifiers")
        if isinstance(number, bool) or not isinstance(number, (int, float)):
            raise ValidationError(f"{field} values must be numbers")
        result[name] = float(number)
    return result


def _base_spec(body: Mapping[str, object]) -> dict[str, object]:
    """Validate the forecast horizon/step/start against the simulate rules."""

    horizon, step = body.get("horizon"), body.get("step")
    if isinstance(horizon, bool) or not isinstance(horizon, (int, float)) or horizon <= 0:
        raise ValidationError("forecast requires a positive horizon")
    if isinstance(step, bool) or not isinstance(step, (int, float)) or step <= 0 or step > horizon:
        raise ValidationError("forecast requires a positive step no larger than the horizon")
    start = body.get("start", 0.0)
    if isinstance(start, bool) or not isinstance(start, (int, float)):
        raise ValidationError("forecast start must be numeric")
    if float(start) >= float(horizon):
        raise ValidationError("forecast start must be before the horizon")
    if round(float(horizon) / float(step)) > 1_000_000:
        raise ValidationError("forecast exceeds the maximum step count")
    return {"horizon": float(horizon), "step": float(step), "start": float(start)}


def _parse_interventions(value: object, start: float, horizon: float) -> list[dict[str, object]]:
    if value is None:
        return []
    if not isinstance(value, (list, tuple)):
        raise ValidationError("interventions must be a list of scheduled changes")
    parsed: list[dict[str, object]] = []
    for entry in value:
        if not isinstance(entry, Mapping):
            raise ValidationError("each intervention must be an object")
        at = entry.get("at")
        if isinstance(at, bool) or not isinstance(at, (int, float)):
            raise ValidationError("each intervention requires a numeric 'at' time")
        if not (start < float(at) < horizon):
            raise ValidationError("intervention 'at' must fall strictly inside (start, horizon)")
        parsed.append(
            {
                "at": float(at),
                "parameters": _number_map(entry.get("parameters"), "intervention parameters"),
                "inputs": _number_map(entry.get("inputs"), "intervention inputs"),
            }
        )
    parsed.sort(key=lambda item: item["at"])
    return parsed


def _simulate_segment(
    record: Mapping[str, object],
    *,
    start: float,
    end: float,
    step: float,
    initial: Mapping[str, float],
    parameters: Mapping[str, float],
    inputs: Mapping[str, float],
) -> dict[str, object]:
    spec: dict[str, object] = {"horizon": end, "step": step, "start": start, "initial": dict(initial)}
    if parameters:
        spec["parameters"] = dict(parameters)
    if inputs:
        spec["inputs"] = dict(inputs)
    return simulate_world(record, spec)


def forecast(record: Mapping[str, object], body: object) -> dict[str, object]:
    """Forecast a world's trajectory, honouring scheduled interventions.

    A forecast is stitched from one or more real native simulate calls: each
    scheduled intervention starts a new segment whose initial state is the
    previous segment's final state and whose parameters/inputs fold in every
    intervention active so far.  With no interventions it is a single simulate
    call -- the same engine path as ``/v1/worlds/{id}/simulate``.
    """

    if not isinstance(body, Mapping):
        raise ValidationError("forecast body must be an object")
    spec = _base_spec(body)
    start, horizon, step = float(spec["start"]), float(spec["horizon"]), float(spec["step"])
    initial = _number_map(body.get("initial"), "initial")
    if not initial:
        raise ValidationError("forecast requires an 'initial' state with at least one value")
    base_parameters = _number_map(body.get("parameters"), "parameters")
    base_inputs = _number_map(body.get("inputs"), "inputs")
    interventions = _parse_interventions(body.get("interventions"), start, horizon)

    # Segment boundaries: [start, i0, i1, ..., horizon].
    boundaries = [start, *[item["at"] for item in interventions], horizon]
    parameters = dict(base_parameters)
    inputs = dict(base_inputs)
    time: list[float] = []
    values: dict[str, list[float]] = {}
    segment_initial: dict[str, float] = dict(initial)

    for index in range(len(boundaries) - 1):
        seg_start, seg_end = boundaries[index], boundaries[index + 1]
        if index > 0:
            change = interventions[index - 1]
            parameters.update(change["parameters"])  # type: ignore[arg-type]
            inputs.update(change["inputs"])  # type: ignore[arg-type]
        segment = _simulate_segment(
            record,
            start=seg_start,
            end=seg_end,
            step=step,
            initial=segment_initial,
            parameters=parameters,
            inputs=inputs,
        )
        seg_time = list(segment["time"])  # type: ignore[arg-type]
        seg_values = {name: list(series) for name, series in segment["values"].items()}  # type: ignore[union-attr]
        # Drop the duplicated boundary sample shared with the prior segment.
        offset = 1 if time and seg_time and seg_time[0] == time[-1] else 0
        time.extend(seg_time[offset:])
        for name, series in seg_values.items():
            values.setdefault(name, []).extend(series[offset:])
        # Carry the final state forward to seed the next segment.
        segment_initial = {name: series[-1] for name, series in seg_values.items() if series}
        if not segment_initial:
            segment_initial = dict(initial)

    return {
        "id": record.get("id"),
        "name": record.get("name"),
        "start": start,
        "horizon": horizon,
        "step": step,
        "interventions": interventions,
        "trajectory": {"time": time, "values": values},
    }


# --------------------------------------------------------------------------- #
# 3. report (self-contained HTML)                                             #
# --------------------------------------------------------------------------- #


def report_html(record: Mapping[str, object]) -> str:
    """Render a self-contained HTML report (equations + structure + inline SVG).

    Reuses the SDK's stdlib-only report generator.  No trajectory is drawn: a
    stored world carries no observations, so the report honestly shows the
    world's laws, dependency structure and assumptions rather than a fabricated
    simulation.
    """

    states = _states(record)
    parameters = _parameters(record)
    equations = _equations(record)
    read = laws.read_laws(equations)
    deps = laws.dependencies(read, states)
    name = str(record.get("name") or record.get("id") or "world")
    return build_report(
        title=f"LawSynth World — {name}",
        summary=[
            ("state variables", ", ".join(states) or "(none)"),
            ("parameters", ", ".join(sorted(parameters)) or "(none)"),
            ("laws discovered", str(len(read))),
        ],
        equations=equations,
        laws_readable=[str(law["readable"]) for law in read],
        fit={},
        trajectory_time=(),
        trajectory_values={},
        dependencies=deps,
        assumptions=_ASSUMPTIONS,
    )


# --------------------------------------------------------------------------- #
# 4. compare                                                                   #
# --------------------------------------------------------------------------- #


def _world_ref(body: Mapping[str, object], *keys: str) -> str:
    for key in keys:
        value = body.get(key)
        if isinstance(value, str) and value:
            return value
        if value is not None:
            raise ValidationError(f"'{key}' must be a world id string")
    raise ValidationError(f"compare requires two world ids ({' and '.join(keys[:1])}/...)")


def compare_refs(body: object) -> tuple[str, str]:
    """Extract the two world ids to compare from a request body."""

    if not isinstance(body, Mapping):
        raise ValidationError("compare body must be an object")
    left = _world_ref(body, "left", "a", "base")
    right = _world_ref(body, "right", "b", "target")
    return left, right


def compare(left: Mapping[str, object], right: Mapping[str, object]) -> dict[str, object]:
    """Structured diff of two stored worlds: variables, parameters, laws, size."""

    left_states, right_states = set(_states(left)), set(_states(right))
    left_controls, right_controls = set(_controls(left)), set(_controls(right))
    left_params, right_params = _parameters(left), _parameters(right)
    left_eq, right_eq = _equations(left), _equations(right)

    changed_params: dict[str, dict[str, float]] = {}
    unchanged_params: list[str] = []
    for name in sorted(set(left_params) & set(right_params)):
        lo, hi = left_params[name], right_params[name]
        if lo == hi:
            unchanged_params.append(name)
        else:
            changed_params[name] = {"left": lo, "right": hi, "delta": hi - lo}

    common_laws = sorted(set(left_eq) & set(right_eq))
    changed_laws = [
        {"target": target, "left": left_eq[target], "right": right_eq[target]}
        for target in common_laws
        if left_eq[target] != right_eq[target]
    ]
    unchanged_laws = [target for target in common_laws if left_eq[target] == right_eq[target]]

    left_terms = laws.total_terms(laws.read_laws(left_eq))
    right_terms = laws.total_terms(laws.read_laws(right_eq))

    return {
        "left": {"id": left.get("id"), "name": left.get("name")},
        "right": {"id": right.get("id"), "name": right.get("name")},
        "variables": _set_diff(left_states, right_states),
        "controls": _set_diff(left_controls, right_controls),
        "parameters": {
            "added": {name: right_params[name] for name in sorted(set(right_params) - set(left_params))},
            "removed": {name: left_params[name] for name in sorted(set(left_params) - set(right_params))},
            "changed": changed_params,
            "unchanged": unchanged_params,
        },
        "laws": {
            "added": sorted(set(right_eq) - set(left_eq)),
            "removed": sorted(set(left_eq) - set(right_eq)),
            "changed": changed_laws,
            "unchanged": unchanged_laws,
        },
        "complexity_delta": {
            "laws": len(right_eq) - len(left_eq),
            "parameters": len(right_params) - len(left_params),
            "controls": len(right_controls) - len(left_controls),
            "total_terms": right_terms - left_terms,
        },
    }


def _set_diff(left: set[str], right: set[str]) -> dict[str, list[str]]:
    return {
        "added": sorted(right - left),
        "removed": sorted(left - right),
        "common": sorted(left & right),
    }
