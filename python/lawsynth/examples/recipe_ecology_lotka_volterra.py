#!/usr/bin/env python3
"""Ecology recipe demo — recover Lotka–Volterra predator–prey from data.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/recipe_ecology_lotka_volterra.py

The script integrates the canonical Lotka–Volterra system with a deterministic
RK4 stepper (no external data, no randomness), then hands the observations to
``Study.discover(recipe="ecology")``. The ecology recipe uses a quadratic
feature library — exactly what the α·x·y interaction terms need — so discovery
recovers the true coefficients and the forward simulation tracks the data.

True system (α=1.1, β=0.4, δ=0.1, γ=0.4)::

    dx/dt =  α·x − β·x·y      (prey: growth minus predation)
    dy/dt = −γ·y + δ·x·y      (predator: decay plus predation gain)
"""

from __future__ import annotations

from typing import Callable, Sequence

import lawsynth


def integrate(
    deriv: Callable[[float, Sequence[float]], list[float]],
    y0: Sequence[float],
    *,
    dt: float,
    steps: int,
    sample: int = 1,
) -> tuple[list[float], list[list[float]]]:
    """Deterministic RK4 integration; returns (times, per-state value columns)."""
    y = list(y0)
    t = 0.0
    times: list[float] = []
    columns: list[list[float]] = [[] for _ in y0]
    for i in range(steps):
        if i % sample == 0:
            times.append(t)
            for j, value in enumerate(y):
                columns[j].append(value)
        k1 = deriv(t, y)
        k2 = deriv(t + dt / 2, [y[j] + dt / 2 * k1[j] for j in range(len(y))])
        k3 = deriv(t + dt / 2, [y[j] + dt / 2 * k2[j] for j in range(len(y))])
        k4 = deriv(t + dt, [y[j] + dt * k3[j] for j in range(len(y))])
        y = [y[j] + dt / 6 * (k1[j] + 2 * k2[j] + 2 * k3[j] + k4[j]) for j in range(len(y))]
        t += dt
    return times, columns


def main() -> None:
    alpha, beta, delta, gamma = 1.1, 0.4, 0.1, 0.4

    def lotka_volterra(_t: float, state: Sequence[float]) -> list[float]:
        x, y = state
        return [alpha * x - beta * x * y, delta * x * y - gamma * y]

    times, (prey, predator) = integrate(
        lotka_volterra, [10.0, 5.0], dt=0.005, steps=6000, sample=15
    )

    recipe = lawsynth.recipes.get("ecology")
    print("=" * 70)
    print(recipe.describe())
    print("=" * 70, "\n")

    study = lawsynth.Study.from_columns(
        times, {"x": prey, "y": predator}, state=["x", "y"], name="lotka_volterra"
    )

    # Discover using the ecology recipe (quadratic interaction library).
    discovery = study.discover(recipe="ecology")

    print("Recovered laws (compare to the true system in the docstring):")
    print(f"  true   dx/dt =  {alpha}·x - {beta}·x·y")
    print(f"  true   dy/dt = -{gamma}·y + {delta}·x·y")
    explanation = discovery.explain()
    for law in explanation.laws:
        print(f"  found  {law.readable}")
    print()

    print("Fit quality (forward simulation vs. observations):")
    for state, metrics in sorted(explanation.fit.items()):
        print(f"  {state}: R² = {metrics['r_squared']:.5f}, RMSE = {metrics['rmse']:.4g}")
    print()

    # Use it: forecast a what-if with a larger starting predator population.
    forecast = study.forecast({"y": 8.0}, horizon=20.0)
    print("What-if — start with more predators (y: 5 -> 8):")
    print(f"  final divergence from baseline: "
          f"{ {k: round(v, 3) for k, v in forecast.divergence.items()} }")


if __name__ == "__main__":
    main()
