"""Deterministic fixture generators.

Given a small declarative spec, these functions produce fixture data structures
with no randomness beyond a fixed, spec-derived seed. The generators mirror the
deterministic observation processes used by the scientific benchmarks
(``benchmarks/_common.py``) and the World IR bundle shape from
``specs/world-ir``.
"""

from __future__ import annotations

import math
import random
from typing import Any

from checksum import seed_from


def _times(samples: int, step: float) -> list[float]:
    if samples < 3:
        raise ValueError("an observation fixture needs at least three samples")
    if not math.isfinite(step) or step <= 0:
        raise ValueError("step must be finite and positive")
    return [index * step for index in range(samples)]


def _series(kind: str, times: list[float], params: dict[str, float]) -> dict[str, list[float]]:
    if kind == "exponential_decay":
        rate = float(params.get("rate", 1.0))
        return {"x": [math.exp(-rate * t) for t in times]}
    if kind == "harmonic":
        omega = float(params.get("omega", 1.0))
        return {
            "x": [math.cos(omega * t) for t in times],
            "v": [-omega * math.sin(omega * t) for t in times],
        }
    if kind == "logistic_map":
        rate = float(params.get("rate", 3.7))
        value = float(params.get("initial", 0.5))
        values = [value]
        for _ in times[1:]:
            value = rate * value * (1.0 - value)
            values.append(value)
        return {"x": values}
    raise ValueError(f"unknown observation kind {kind!r}")


def observation_fixture(spec: dict[str, Any]) -> dict[str, Any]:
    """Build a regularly sampled observation dataset fixture."""
    samples = int(spec["samples"])
    step = float(spec["step"])
    times = _times(samples, step)
    channels = _series(str(spec["kind"]), times, dict(spec.get("parameters", {})))

    noise = spec.get("noise")
    if noise is not None:
        generator = random.Random(seed_from(str(spec.get("name", "fixture")), str(spec["kind"])))
        scale = float(noise)
        channels = {
            name: [value + generator.gauss(0.0, scale) for value in series]
            for name, series in channels.items()
        }

    columns = list(channels)
    rows = [
        {"time": times[index], **{name: channels[name][index] for name in columns}}
        for index in range(samples)
    ]
    return {
        "time_column": "time",
        "columns": columns,
        "sample_count": samples,
        "step": step,
        "rows": rows,
    }


def world_bundle_fixture(spec: dict[str, Any]) -> dict[str, Any]:
    """Build a validated-shape World IR bundle payload with lexical ordering."""
    variables = sorted(spec.get("variables", []), key=lambda entry: entry["id"])
    parameters = sorted(spec.get("parameters", []), key=lambda entry: entry["id"])
    laws = sorted(spec.get("laws", []), key=lambda entry: entry["target"])
    return {
        "spec_version": str(spec.get("spec_version", "0.1")),
        "kind": str(spec.get("kind", "continuous")),
        "variables": variables,
        "parameters": parameters,
        "laws": laws,
    }


_BUILDERS = {
    "observation": observation_fixture,
    "world_bundle": world_bundle_fixture,
}


def build_fixture(spec: dict[str, Any]) -> dict[str, Any]:
    """Dispatch a spec to the builder named by its ``type`` field."""
    fixture_type = spec.get("type")
    try:
        builder = _BUILDERS[str(fixture_type)]
    except KeyError as error:
        known = ", ".join(sorted(_BUILDERS))
        raise ValueError(f"unknown fixture type {fixture_type!r}; known: {known}") from error
    return builder(spec)
