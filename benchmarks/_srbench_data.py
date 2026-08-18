"""Deterministic data generators for the SRBench-style credibility suite.

Every generator here is pure Python, offline, and byte-reproducible.  Two kinds
of process are produced:

* **ODE trajectories** — classic low-dimensional nonlinear systems from
  Strogatz's *Nonlinear Dynamics and Chaos* (and a handful of inherently
  dynamical physics laws from the Feynman lectures).  These are integrated with
  a fixed-step RK4 scheme so the resulting CSV is identical on every run and on
  every platform.  The LawSynth dynamics engine is *designed* for these.

* **Static algebraic samples** — a subset of the public Feynman equations
  (``y = f(x1, ..., xn)``).  We generate our *own* deterministic data (a seeded
  standard-library RNG over documented feature ranges); we do **not** vendor
  AI-Feynman's ``FeynmanEquations`` data files or ``units.xlsx``.  These are
  honest capability boundaries: LawSynth discovers governing dynamics, not
  static multivariate regression, so they are generated (for reproducibility)
  but reported as boundaries rather than scored for recovery.

Nothing in this module invokes LawSynth: it only *produces reference data*.
"""

from __future__ import annotations

import csv
import math
import random
from collections.abc import Callable, Sequence
from pathlib import Path
from typing import Any

# --------------------------------------------------------------------------- #
# Deterministic fixed-step RK4 integration for the ODE families.
# --------------------------------------------------------------------------- #

Derivative = Callable[[Sequence[float], dict[str, float]], list[float]]


def _rk4(
    rhs: Derivative,
    initial: Sequence[float],
    step: float,
    samples: int,
    params: dict[str, float],
) -> list[tuple[float, ...]]:
    """Integrate ``rhs`` with classic RK4 and return one row per sample."""
    if samples < 3:
        raise ValueError("an ODE benchmark needs at least three samples")
    if not math.isfinite(step) or step <= 0:
        raise ValueError("step must be finite and positive")
    state = list(initial)
    rows: list[tuple[float, ...]] = [tuple(state)]
    for _ in range(samples - 1):
        k1 = rhs(state, params)
        k2 = rhs([s + 0.5 * step * d for s, d in zip(state, k1, strict=True)], params)
        k3 = rhs([s + 0.5 * step * d for s, d in zip(state, k2, strict=True)], params)
        k4 = rhs([s + step * d for s, d in zip(state, k3, strict=True)], params)
        state = [
            s + step * (a + 2.0 * b + 2.0 * c + d) / 6.0
            for s, a, b, c, d in zip(state, k1, k2, k3, k4, strict=True)
        ]
        rows.append(tuple(state))
    return rows


# Right-hand sides.  Each takes the state in declared order and a parameter map.
def _logistic(s: Sequence[float], p: dict[str, float]) -> list[float]:
    return [p["r"] * s[0] * (1.0 - s[0] / p["K"])]


def _saddle_node(s: Sequence[float], p: dict[str, float]) -> list[float]:
    return [p["mu"] - s[0] * s[0]]


def _bistable(s: Sequence[float], p: dict[str, float]) -> list[float]:
    return [p["r"] * s[0] - s[0] ** 3]


def _damped_oscillator(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, v = s
    return [v, -p["k"] * x - p["c"] * v]


def _van_der_pol(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, y = s
    return [y, p["mu"] * (1.0 - x * x) * y - x]


def _duffing(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, y = s
    return [y, -p["delta"] * y - p["alpha"] * x - p["beta"] * x ** 3]


def _lotka_volterra(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, y = s
    return [p["a"] * x - p["b"] * x * y, -p["c"] * y + p["d"] * x * y]


def _brusselator(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, y = s
    return [p["a"] - (p["b"] + 1.0) * x + x * x * y, p["b"] * x - x * x * y]


def _fitzhugh_nagumo(s: Sequence[float], p: dict[str, float]) -> list[float]:
    v, w = s
    return [v - v ** 3 / 3.0 - w + p["I"], p["eps"] * (v + p["a"] - p["b"] * w)]


def _lorenz(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, y, z = s
    return [p["sigma"] * (y - x), x * (p["rho"] - z) - y, x * y - p["beta"] * z]


def _rossler(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, y, z = s
    return [-y - z, x + p["a"] * y, p["b"] + z * (x - p["c"])]


def _pendulum(s: Sequence[float], p: dict[str, float]) -> list[float]:
    theta, omega = s
    return [omega, -p["b"] * omega - p["c"] * math.sin(theta)]


def _newton_cooling(s: Sequence[float], p: dict[str, float]) -> list[float]:
    return [-p["k"] * (s[0] - p["T_env"])]


def _free_fall_drag(s: Sequence[float], p: dict[str, float]) -> list[float]:
    return [p["g"] - p["c"] * s[0]]


def _rc_discharge(s: Sequence[float], p: dict[str, float]) -> list[float]:
    return [-s[0] / (p["R"] * p["C"])]


def _radioactive_decay(s: Sequence[float], p: dict[str, float]) -> list[float]:
    return [-s[0] / p["tau"]]


def _harmonic(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, v = s
    return [v, -p["omega2"] * x]


ODE_SYSTEMS: dict[str, Derivative] = {
    "logistic": _logistic,
    "saddle_node": _saddle_node,
    "bistable": _bistable,
    "damped_oscillator": _damped_oscillator,
    "van_der_pol": _van_der_pol,
    "duffing": _duffing,
    "lotka_volterra": _lotka_volterra,
    "brusselator": _brusselator,
    "fitzhugh_nagumo": _fitzhugh_nagumo,
    "lorenz": _lorenz,
    "rossler": _rossler,
    "pendulum": _pendulum,
    "newton_cooling": _newton_cooling,
    "free_fall_drag": _free_fall_drag,
    "rc_discharge": _rc_discharge,
    "radioactive_decay": _radioactive_decay,
    "harmonic": _harmonic,
    # Blackbox processes (no published closed-form ground truth advertised).
    "blackbox_coupled_decay": _damped_oscillator,  # reused RHS, opaque to scorer
}


def _blackbox_driven(s: Sequence[float], p: dict[str, float]) -> list[float]:
    """A mildly nonlinear coupled process presented without ground truth."""
    a, b = s
    return [-p["k1"] * a + p["c"] * a * b, p["k2"] * a - p["k3"] * b]


def _blackbox_three(s: Sequence[float], p: dict[str, float]) -> list[float]:
    x, y, z = s
    return [
        -p["a"] * x + p["b"] * y,
        p["c"] * x - p["d"] * y - p["e"] * y * z,
        p["f"] * y * z - p["g"] * z,
    ]


ODE_SYSTEMS["blackbox_driven"] = _blackbox_driven
ODE_SYSTEMS["blackbox_three"] = _blackbox_three


def integrate_ode(config: dict[str, Any]) -> tuple[list[str], list[float], list[tuple[float, ...]]]:
    """Return ``(state_names, times, rows)`` for an ODE benchmark config."""
    system = config["system"]
    kind = str(system["kind"])
    rhs = ODE_SYSTEMS.get(kind)
    if rhs is None:
        raise ValueError(f"unknown ODE system '{kind}'")
    states = [str(name) for name in system["states"]]
    step = float(system["step"])
    samples = int(system["samples"])
    params = {str(k): float(v) for k, v in system.get("parameters", {}).items()}
    initial_map = {str(k): float(v) for k, v in system["initial"].items()}
    initial = [initial_map[name] for name in states]
    times = [index * step for index in range(samples)]
    rows = _rk4(rhs, initial, step, samples, params)
    return states, times, rows


# --------------------------------------------------------------------------- #
# Static Feynman equations (public formulas; our own deterministic data).
# --------------------------------------------------------------------------- #

FeatureRange = tuple[float, float]
StaticFormula = Callable[[dict[str, float]], float]


def _static(features: dict[str, FeatureRange], formula: StaticFormula) -> dict[str, Any]:
    return {"features": features, "formula": formula}


# Each entry documents its canonical Feynman equation number in the case TOML.
STATIC_FEYNMAN: dict[str, dict[str, Any]] = {
    # I.6.20a : f = exp(-theta^2 / 2)
    "gaussian_std": _static(
        {"theta": (-2.0, 2.0)},
        lambda v: math.exp(-v["theta"] ** 2 / 2.0),
    ),
    # I.6.20 : f = exp(-(theta/sigma)^2 / 2)
    "gaussian_scaled": _static(
        {"theta": (-2.0, 2.0), "sigma": (0.5, 2.0)},
        lambda v: math.exp(-((v["theta"] / v["sigma"]) ** 2) / 2.0),
    ),
    # I.9.18 (reduced) : F = G m1 m2 / r^2
    "gravitation": _static(
        {"G": (1.0, 2.0), "m1": (1.0, 4.0), "m2": (1.0, 4.0), "r": (1.0, 3.0)},
        lambda v: v["G"] * v["m1"] * v["m2"] / v["r"] ** 2,
    ),
    # I.12.1 : F = mu Nn
    "friction": _static(
        {"mu": (0.1, 1.0), "Nn": (1.0, 10.0)},
        lambda v: v["mu"] * v["Nn"],
    ),
    # I.12.2 : F = q1 q2 / (4 pi eps r^2)
    "coulomb": _static(
        {"q1": (1.0, 3.0), "q2": (1.0, 3.0), "eps": (1.0, 2.0), "r": (1.0, 3.0)},
        lambda v: v["q1"] * v["q2"] / (4.0 * math.pi * v["eps"] * v["r"] ** 2),
    ),
    # I.13.12 : U = G m1 m2 (1/r2 - 1/r1)
    "grav_potential": _static(
        {"G": (1.0, 2.0), "m1": (1.0, 4.0), "m2": (1.0, 4.0), "r1": (1.0, 3.0), "r2": (1.0, 3.0)},
        lambda v: v["G"] * v["m1"] * v["m2"] * (1.0 / v["r2"] - 1.0 / v["r1"]),
    ),
    # I.14.3 : U = m g z
    "potential_energy": _static(
        {"m": (1.0, 5.0), "g": (9.0, 10.0), "z": (0.0, 10.0)},
        lambda v: v["m"] * v["g"] * v["z"],
    ),
    # I.14.4 : U = k x^2 / 2
    "spring_energy": _static(
        {"k": (1.0, 5.0), "x": (0.0, 3.0)},
        lambda v: v["k"] * v["x"] ** 2 / 2.0,
    ),
    # I.16.6 : relativistic velocity addition v = (u + w) / (1 + u w / c^2)
    "velocity_addition": _static(
        {"u": (0.0, 1.0), "w": (0.0, 1.0), "c": (3.0, 4.0)},
        lambda v: (v["u"] + v["w"]) / (1.0 + v["u"] * v["w"] / v["c"] ** 2),
    ),
    # I.18.4 : center of mass r = (m1 r1 + m2 r2) / (m1 + m2)
    "center_of_mass": _static(
        {"m1": (1.0, 5.0), "m2": (1.0, 5.0), "r1": (0.0, 5.0), "r2": (0.0, 5.0)},
        lambda v: (v["m1"] * v["r1"] + v["m2"] * v["r2"]) / (v["m1"] + v["m2"]),
    ),
    # I.25.13 : V = q / C
    "capacitor_voltage": _static(
        {"q": (1.0, 5.0), "C": (1.0, 5.0)},
        lambda v: v["q"] / v["C"],
    ),
    # I.27.6 : focal 1 / (1/d1 + n/d2)
    "lens_focal": _static(
        {"d1": (1.0, 5.0), "d2": (1.0, 5.0), "n": (1.0, 2.0)},
        lambda v: 1.0 / (1.0 / v["d1"] + v["n"] / v["d2"]),
    ),
    # I.29.4 : k = omega / c
    "wavenumber": _static(
        {"omega": (1.0, 10.0), "c": (3.0, 4.0)},
        lambda v: v["omega"] / v["c"],
    ),
    # I.34.8 : omega = q v B / p
    "cyclotron": _static(
        {"q": (1.0, 3.0), "v": (1.0, 5.0), "B": (1.0, 3.0), "p": (1.0, 5.0)},
        lambda v: v["q"] * v["v"] * v["B"] / v["p"],
    ),
    # I.39.22 : P = n kb T / V
    "ideal_gas": _static(
        {"n": (1.0, 5.0), "kb": (1.0, 2.0), "T": (1.0, 5.0), "V": (1.0, 5.0)},
        lambda v: v["n"] * v["kb"] * v["T"] / v["V"],
    ),
    # II.3.24 : flux FE = P / (4 pi r^2)
    "radiated_flux": _static(
        {"P": (1.0, 10.0), "r": (1.0, 5.0)},
        lambda v: v["P"] / (4.0 * math.pi * v["r"] ** 2),
    ),
}


def _static_seed(kind: str) -> int:
    return sum((index + 1) * ord(char) for index, char in enumerate(kind))


def sample_static(config: dict[str, Any]) -> tuple[list[str], list[list[float]]]:
    """Deterministically sample a static Feynman equation.

    Returns ``(column_names, rows)`` where the final column is the target.
    Uniform sampling over documented feature ranges with a fixed per-equation
    seed keeps the emitted CSV byte-identical across runs and machines.
    """
    system = config["system"]
    kind = str(system["kind"])
    spec = STATIC_FEYNMAN.get(kind)
    if spec is None:
        raise ValueError(f"unknown static Feynman equation '{kind}'")
    samples = int(system["samples"])
    target = str(system.get("target", "y"))
    feature_names = list(spec["features"])
    formula: StaticFormula = spec["formula"]
    rng = random.Random(_static_seed(kind))
    rows: list[list[float]] = []
    for _ in range(samples):
        values = {
            name: rng.uniform(low, high)
            for name, (low, high) in spec["features"].items()
        }
        row = [values[name] for name in feature_names]
        row.append(formula(values))
        rows.append(row)
    return [*feature_names, target], rows


# --------------------------------------------------------------------------- #
# CSV materialisation (shared byte-stable formatting).
# --------------------------------------------------------------------------- #


def _format(value: float) -> str:
    return f"{value:.12g}"


def write_dataset(config: dict[str, Any], workdir: Path) -> Path:
    """Materialise a benchmark's deterministic dataset as a CSV file."""
    family = str(config["family"])
    workdir.mkdir(parents=True, exist_ok=True)
    output = workdir / "observations.csv"
    if family == "feynman" and config["capability"]["status"] != "supported":
        columns, rows = sample_static(config)
        with output.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.writer(handle, lineterminator="\n")
            writer.writerow(columns)
            for row in rows:
                writer.writerow([_format(value) for value in row])
        return output
    states, times, rows = integrate_ode(config)
    with output.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow(["time", *states])
        for time, row in zip(times, rows, strict=True):
            writer.writerow([_format(time), *(_format(value) for value in row)])
    return output


def ground_truth_trajectory(config: dict[str, Any]) -> dict[str, list[float]]:
    """Return the generated trajectory keyed by state name (for R^2 scoring)."""
    states, _times, rows = integrate_ode(config)
    columns: dict[str, list[float]] = {name: [] for name in states}
    for row in rows:
        for name, value in zip(states, row, strict=True):
            columns[name].append(value)
    return columns
