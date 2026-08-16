#!/usr/bin/env python3
"""Shared, dependency-free execution layer for the LawSynth examples.

The examples deliberately keep their data generation and baseline discovery
auditable: trajectories are generated from the stated equations, derivative
estimates use finite differences, and sparse-ish model selection is performed
with a small ridge least-squares solver.  When the optional native LawSynth
extension is installed, the discovery command also runs it and records that
fact; the JSON baseline remains portable and reproducible without a compiler.
"""
from __future__ import annotations

import csv
import json
import math
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Callable


class UnsupportedCapability(RuntimeError):
    """Raised when an example requires an intentionally unavailable subsystem."""


@dataclass(frozen=True)
class Example:
    root: Path
    config: dict

    @property
    def states(self) -> list[str]:
        return list(self.config["states"])

    @property
    def output(self) -> Path:
        path = self.root / "output"
        path.mkdir(exist_ok=True)
        return path


def load_example(root: Path) -> Example:
    with (root / "config.toml").open("rb") as handle:
        return Example(root, tomllib.load(handle))


def _parameters(cfg: dict) -> dict[str, float]:
    return {key: float(value) for key, value in cfg.get("parameters", {}).items()}


def rhs(kind: str, t: float, y: list[float], p: dict[str, float], history: Callable[[float], list[float]] | None = None) -> list[float]:
    """Evaluate the equations advertised by an example configuration."""
    if kind in {"quickstart", "customer_growth"}:
        x = y[0]
        return [p["r"] * x * (1.0 - x / p["capacity"])]
    if kind == "lorenz":
        x, yv, z = y
        return [p["sigma"] * (yv - x), x * (p["rho"] - z) - yv, x * yv - p["beta"] * z]
    if kind == "lotka_volterra":
        prey, predator = y
        return [p["alpha"] * prey - p["beta"] * prey * predator, p["delta"] * prey * predator - p["gamma"] * predator]
    if kind == "damped_pendulum":
        theta, omega = y
        return [omega, -p["gravity"] * math.sin(theta) - p["damping"] * omega]
    if kind == "sir":
        susceptible, infected, recovered = y
        infections = p["beta"] * susceptible * infected / p["population"]
        return [-infections, infections - p["gamma"] * infected, p["gamma"] * infected]
    if kind == "regime_switching":
        rate = p["rate_before"] if t < p["switch_time"] else p["rate_after"]
        return [rate * y[0]]
    if kind == "delayed_feedback":
        delayed = history(t - p["delay"])[0] if history else y[0]
        return [-p["decay"] * y[0] + p["feedback"] * delayed]
    if kind == "stochastic_volatility":
        price, variance = y
        return [p["drift"] * price, p["mean_reversion"] * (p["long_variance"] - variance)]
    if kind == "supply_demand":
        demand, supply, price = y
        return [p["demand_rate"] * (p["target_price"] - price), p["supply_rate"] * (price - p["cost"]), p["price_rate"] * (demand - supply)]
    if kind == "inventory_control":
        inventory, backlog = y
        order = max(0.0, p["target_inventory"] - inventory)
        sales = min(inventory + order, p["demand"] + backlog)
        return [order - sales, p["demand"] - sales]
    if kind == "energy_load":
        load = y[0]
        target = p["base_load"] + p["amplitude"] * math.sin(2.0 * math.pi * t / p["period"])
        return [p["relaxation"] * (target - load)]
    if kind == "macro_dynamics":
        output, inflation = y
        return [p["growth"] * output - p["sensitivity"] * inflation, p["inflation_pressure"] * output - p["inflation_decay"] * inflation]
    if kind == "market_microstructure":
        mid, imbalance = y
        return [p["impact"] * imbalance, -p["resilience"] * imbalance - p["liquidity"] * mid]
    if kind == "synthetic_control":
        treated, donor = y
        treatment = p["effect"] if t >= p["treatment_time"] else 0.0
        return [p["coupling"] * (donor - treated) + treatment, p["donor_growth"] * donor]
    if kind == "user_constraints":
        x, z = y
        return [p["positive_rate"] * x, -p["decay"] * z]
    if kind in {"custom_operator", "custom_stage"}:
        # The generated observations remain a real, standard model.  The
        # requested extension is assessed separately and never emulated.
        return [-p["decay"] * y[0]]
    if kind == "bundle_interchange":
        x = y[0]
        return [-p["decay"] * x]
    if kind == "server_api":
        return [-p["decay"] * y[0]]
    raise ValueError(f"unknown example model {kind!r}")


def _rk4_step(kind: str, t: float, state: list[float], h: float, p: dict[str, float], history: Callable[[float], list[float]] | None) -> list[float]:
    def add(base: list[float], scale: float, delta: list[float]) -> list[float]:
        return [value + scale * derivative for value, derivative in zip(base, delta)]
    k1 = rhs(kind, t, state, p, history)
    k2 = rhs(kind, t + h / 2.0, add(state, h / 2.0, k1), p, history)
    k3 = rhs(kind, t + h / 2.0, add(state, h / 2.0, k2), p, history)
    k4 = rhs(kind, t + h, add(state, h, k3), p, history)
    return [value + h * (a + 2.0 * b + 2.0 * c + d) / 6.0 for value, a, b, c, d in zip(state, k1, k2, k3, k4)]


def integrate(cfg: dict) -> tuple[list[float], dict[str, list[float]]]:
    """Integrate one configured world deterministically using RK4/Euler DDE."""
    kind, p = str(cfg["kind"]), _parameters(cfg)
    start, stop, step = (float(cfg[key]) for key in ("start", "stop", "step"))
    count = round((stop - start) / step)
    if count < 2 or not math.isclose(start + count * step, stop, abs_tol=1e-9):
        raise ValueError("stop-start must be an integral number of steps and contain at least three rows")
    times = [start + index * step for index in range(count + 1)]
    rows: list[list[float]] = [[float(value) for value in cfg["initial"]]]

    def history(query: float) -> list[float]:
        if query <= start:
            return rows[0]
        index = min(len(rows) - 1, max(0, int(round((query - start) / step))))
        return rows[index]

    for index, time in enumerate(times[:-1]):
        state = rows[-1]
        if kind == "delayed_feedback":
            derivative = rhs(kind, time, state, p, history)
            next_state = [value + step * slope for value, slope in zip(state, derivative)]
        elif kind == "inventory_control":
            derivative = rhs(kind, time, state, p, history)
            next_state = [max(0.0, value + step * slope) for value, slope in zip(state, derivative)]
        elif kind == "stochastic_volatility":
            # Deterministic xorshift innovations make this SDE example exactly
            # reproducible without pretending the volatility is deterministic.
            seed = (int(p["seed"]) + 1103515245 * (index + 1)) & 0x7FFF_FFFF
            noise = (seed / 0x7FFF_FFFF) * 2.0 - 1.0
            drift = rhs(kind, time, state, p, history)
            next_state = [
                max(1e-12, state[0] + step * drift[0] + state[0] * math.sqrt(max(state[1], 0.0)) * math.sqrt(step) * noise),
                max(1e-12, state[1] + step * drift[1] + p["vol_of_vol"] * math.sqrt(max(state[1], 0.0)) * math.sqrt(step) * noise),
            ]
        else:
            next_state = _rk4_step(kind, time, state, step, p, history)
        if any(not math.isfinite(value) for value in next_state):
            raise ValueError(f"non-finite state at t={time}")
        rows.append(next_state)
    columns = {name: [row[index] for row in rows] for index, name in enumerate(cfg["states"])}
    return times, columns


def _write_csv(path: Path, time: list[float], columns: dict[str, list[float]]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        names = list(columns)
        writer.writerow(["time", *names])
        writer.writerows([f"{time[index]:.12g}", *(f"{columns[name][index]:.12g}" for name in names)] for index in range(len(time)))


def _read_csv(path: Path) -> tuple[list[float], dict[str, list[float]]]:
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    if not rows or "time" not in rows[0]:
        raise ValueError(f"{path} is not a LawSynth example CSV")
    names = [name for name in rows[0] if name != "time"]
    return [float(row["time"]) for row in rows], {name: [float(row[name]) for row in rows] for name in names}


def generate_example(root: Path) -> Path:
    example = load_example(root)
    time, columns = integrate(example.config)
    path = example.output / "observations.csv"
    _write_csv(path, time, columns)
    return path


def _features(columns: dict[str, list[float]]) -> tuple[list[str], list[list[float]]]:
    names = list(columns)
    labels = ["1", *names]
    rows = [[1.0, *(columns[name][index] for name in names)] for index in range(len(next(iter(columns.values()))))]
    for left, a in enumerate(names):
        for b in names[left:]:
            labels.append(f"{a}*{b}")
            for index, row in enumerate(rows):
                row.append(columns[a][index] * columns[b][index])
    return labels, rows


def _solve(matrix: list[list[float]], target: list[float], ridge: float = 1e-8) -> list[float]:
    """Solve normal equations with pivoting; no third-party numerical stack."""
    width = len(matrix[0])
    normal = [[ridge if i == j else 0.0 for j in range(width)] for i in range(width)]
    rhs_vector = [0.0] * width
    for row, value in zip(matrix, target):
        for i, left in enumerate(row):
            rhs_vector[i] += left * value
            for j, right in enumerate(row):
                normal[i][j] += left * right
    for pivot in range(width):
        chosen = max(range(pivot, width), key=lambda index: abs(normal[index][pivot]))
        if abs(normal[chosen][pivot]) < 1e-14:
            raise ValueError("singular discovery design matrix")
        normal[pivot], normal[chosen] = normal[chosen], normal[pivot]
        rhs_vector[pivot], rhs_vector[chosen] = rhs_vector[chosen], rhs_vector[pivot]
        scale = normal[pivot][pivot]
        normal[pivot] = [value / scale for value in normal[pivot]]
        rhs_vector[pivot] /= scale
        for row_index in range(width):
            if row_index == pivot:
                continue
            factor = normal[row_index][pivot]
            normal[row_index] = [value - factor * base for value, base in zip(normal[row_index], normal[pivot])]
            rhs_vector[row_index] -= factor * rhs_vector[pivot]
    return rhs_vector


def _derivative(time: list[float], values: list[float]) -> list[float]:
    result = []
    for index in range(1, len(values) - 1):
        result.append((values[index + 1] - values[index - 1]) / (time[index + 1] - time[index - 1]))
    return result


def discover_example(root: Path) -> Path:
    example = load_example(root)
    cfg = example.config
    if cfg.get("capability") == "unsupported":
        raise UnsupportedCapability(str(cfg["boundary"]))
    source = generate_example(root)
    time, columns = _read_csv(source)
    labels, features = _features(columns)
    equations: dict[str, dict[str, float]] = {}
    mse: dict[str, float] = {}
    design = features[1:-1]
    threshold = float(cfg.get("threshold", 1e-4))
    for state in example.states:
        derivative = _derivative(time, columns[state])
        coefficients = _solve(design, derivative)
        kept = {label: value for label, value in zip(labels, coefficients) if abs(value) >= threshold}
        predictions = [sum(weight * value for weight, value in zip(coefficients, row)) for row in design]
        mse[state] = sum((actual - predicted) ** 2 for actual, predicted in zip(derivative, predictions)) / len(derivative)
        equations[state] = kept
    native = "not-installed"
    try:
        import lawsynth as ls  # type: ignore
        ls.discover(time, columns, state=example.states, polynomial_degree=2, threshold=threshold)
        native = "executed"
    except ImportError:
        pass
    except Exception as error:  # Record true native validation/fitting errors.
        native = f"failed: {type(error).__name__}: {error}"
    payload = {
        "schema_version": 1,
        "method": "finite-difference polynomial library ridge regression",
        "native_engine": native,
        "states": example.states,
        "equations": equations,
        "derivative_mse": mse,
        "source": str(source.relative_to(root)),
    }
    path = example.output / "discovery.json"
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path


def simulate_example(root: Path) -> Path:
    example = load_example(root)
    if example.config.get("capability") == "unsupported":
        raise UnsupportedCapability(str(example.config["boundary"]))
    time, columns = integrate(example.config)
    path = example.output / "simulation.csv"
    _write_csv(path, time, columns)
    return path


def verify_example(root: Path) -> None:
    example = load_example(root)
    expected = json.loads((root / "expected" / "metrics.json").read_text(encoding="utf-8"))
    if expected["status"] == "unsupported":
        try:
            discover_example(root)
        except UnsupportedCapability as error:
            if str(error) != expected["boundary"]:
                raise AssertionError("unsupported boundary changed") from error
            return
        raise AssertionError("unsupported example unexpectedly succeeded")
    dataset = generate_example(root)
    time, columns = _read_csv(dataset)
    if len(time) != expected["samples"] or list(columns) != example.states:
        raise AssertionError("generated dataset does not match its data contract")
    artifact = json.loads(discover_example(root).read_text(encoding="utf-8"))
    if not artifact["equations"] or any(not math.isfinite(value) for value in artifact["derivative_mse"].values()):
        raise AssertionError("discovery did not produce finite equations")
    simulation = simulate_example(root)
    if simulation.stat().st_size <= 40:
        raise AssertionError("simulation output is empty")


def cli(action: str, root: Path) -> int:
    try:
        result = {"generate": generate_example, "discover": discover_example, "simulate": simulate_example}[action](root)
    except UnsupportedCapability as error:
        print(f"unsupported: {error}", file=sys.stderr)
        return 2
    print(result)
    return 0
