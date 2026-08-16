"""Executable contracts for benchmark families not yet exposed by LawSynth.

These contracts deliberately test the public boundary instead of inventing a
causal/regime/uncertainty result.  They use deterministic, family-specific
datasets so that a future implementation has a reproducible target dataset.
"""
from __future__ import annotations

import importlib
import json
import math
import random
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class CapabilityBoundary(Exception):
    """A requested benchmark requires an intentionally unavailable engine."""

    family: str
    name: str
    feature: str

    def __str__(self) -> str:
        return f"{self.family}/{self.name} requires unsupported capability: {self.feature}"


FEATURES = {
    "causal": "causal identification and effect estimation",
    "regime": "regime segmentation and switching-system inference",
    "uncertainty": "calibrated uncertainty estimation and coverage scoring",
}

# These are deliberately public-operation names, rather than internal data
# containers such as ``Regime`` or ``Interval``.  Their absence is checked on
# the installed SDK surface below.
REQUIRED_PUBLIC_OPERATIONS = {
    "causal": ("identify_causal_effect", "estimate_interventional_effect"),
    "regime": ("segment_regimes", "fit_switching_dynamics"),
    "uncertainty": ("calibrate_prediction_intervals", "estimate_parameter_coverage"),
}


def _seed(family: str, name: str) -> int:
    return sum((index + 1) * ord(char) for index, char in enumerate(f"{family}:{name}"))


def generate_series(family: str, name: str, count: int = 128) -> dict[str, Any]:
    """Generate deterministic structured observations for one contract.

    Each family gets a distinct data-generating process.  The generated truth
    is retained only for exercising the contract, never handed to LawSynth as
    an inferred answer.
    """
    rng = random.Random(_seed(family, name))
    rows: list[dict[str, float | int]] = []
    if family == "causal":
        previous = 0.0
        for step in range(count):
            latent = math.sin(step * 0.13) + rng.gauss(0.0, 0.12)
            treatment = 0.8 * latent + 0.25 * previous + rng.gauss(0.0, 0.08)
            if name == "interventional":
                treatment = 1.0 if step % 2 else -1.0
            if name == "lagged":
                outcome = 0.7 * previous + 0.4 * treatment + 0.5 * latent
            elif name == "nonlinear":
                outcome = treatment * treatment + 0.5 * latent + rng.gauss(0.0, 0.04)
            else:
                outcome = 1.2 * treatment + 0.7 * latent + rng.gauss(0.0, 0.04)
            rows.append({"time": step, "latent": latent, "treatment": treatment, "outcome": outcome})
            previous = treatment
        truth = {"estimand": "average_treatment_effect", "known_only_to_generator": True}
    elif family == "regime":
        state = 0
        for step in range(count):
            if name == "change-point":
                state = int(step >= count // 2)
            elif name == "event-driven":
                state = int(step in {32, 33, 76, 77})
            elif name == "markov-switching" and rng.random() < 0.13:
                state = 1 - state
            elif name == "hmm" and rng.random() < 0.08:
                state = 1 - state
            mean = (-1.0 if state == 0 else 1.5) + 0.1 * math.sin(step / 4)
            rows.append({"time": step, "observation": mean + rng.gauss(0.0, 0.12), "hidden_state": state})
        truth = {"labels_are_oracle_only": True, "state_count": 2}
    elif family == "uncertainty":
        for step in range(count):
            centre = math.sin(step * 0.11)
            scale = 0.1 + 0.05 * (step % 5)
            observation = centre + rng.gauss(0.0, scale)
            rows.append({"time": step, "estimate": centre, "observation": observation, "nominal_scale": scale})
        truth = {"target": name, "intervals_not_produced_by_engine": True}
    else:
        raise ValueError(f"unknown benchmark family: {family}")
    return {"family": family, "name": name, "seed": _seed(family, name), "rows": rows, "truth": truth}


def execute(directory: Path) -> dict[str, Any]:
    spec = load_spec(directory)
    data = generate_series(spec["family"], spec["name"], int(spec["samples"]))
    audit = audit_public_api(spec["family"])
    if not audit["boundary"]:
        raise AssertionError(
            "a public operation appeared; replace this boundary benchmark with "
            "a recovery/accuracy benchmark for the implemented feature"
        )
    error = CapabilityBoundary(spec["family"], spec["name"], FEATURES[spec["family"]])
    return {
        "status": "capability_boundary",
        "benchmark": f"{spec['family']}/{spec['name']}",
        "rows": len(data["rows"]),
        "seed": data["seed"],
        "error": str(error),
        "feature": error.feature,
        "missing_public_operations": audit["missing"],
    }


def audit_public_api(family: str) -> dict[str, Any]:
    """Probe the SDK's actual public surface for a benchmark operation.

    This is intentionally an integration check, not a mock exception: it
    imports the package source and examines the exported callable API that a
    user can invoke.  Once any operation below is implemented, the benchmark
    fails and must be replaced by an accuracy or calibration evaluation.
    """
    sdk_source = Path(__file__).resolve().parents[1] / "python" / "lawsynth" / "src"
    if str(sdk_source) not in sys.path:
        sys.path.insert(0, str(sdk_source))
    lawsynth = importlib.import_module("lawsynth")
    missing = [name for name in REQUIRED_PUBLIC_OPERATIONS[family] if not callable(getattr(lawsynth, name, None))]
    return {"boundary": len(missing) == len(REQUIRED_PUBLIC_OPERATIONS[family]), "missing": missing}


def load_spec(directory: Path) -> dict[str, Any]:
    import tomllib

    raw = tomllib.loads((directory / "benchmark.toml").read_text(encoding="utf-8"))
    required = {"family", "name", "samples", "status"}
    if set(raw) != required:
        raise ValueError(f"invalid benchmark keys in {directory}: {sorted(raw)}")
    if raw["status"] != "capability_boundary" or raw["family"] not in FEATURES:
        raise ValueError("benchmark must describe a known unavailable capability")
    if not isinstance(raw["samples"], int) or raw["samples"] < 16:
        raise ValueError("samples must be an integer >= 16")
    return raw


def write_generated(directory: Path) -> Path:
    spec = load_spec(directory)
    payload = generate_series(spec["family"], spec["name"], int(spec["samples"]))
    output = Path(tempfile.gettempdir()) / f"lawsynth-{spec['family']}-{spec['name']}.json"
    output.write_text(json.dumps(payload, sort_keys=True), encoding="utf-8")
    return output


def score(directory: Path) -> dict[str, Any]:
    result = execute(directory)
    expected = json.loads((directory / "expected.json").read_text(encoding="utf-8"))
    if result["status"] != expected["status"] or result["feature"] != expected["feature"]:
        raise AssertionError(f"contract mismatch: {result!r}")
    if result["rows"] < 16:
        raise AssertionError("contract did not generate sufficient observations")
    return result


def main(directory: Path, mode: str) -> int:
    if mode == "generate":
        print(write_generated(directory))
    elif mode == "run":
        print(json.dumps(execute(directory), sort_keys=True))
    elif mode == "score":
        print(json.dumps(score(directory), sort_keys=True))
    else:
        raise ValueError(f"unknown mode: {mode}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(Path(sys.argv[1]), sys.argv[2]))
