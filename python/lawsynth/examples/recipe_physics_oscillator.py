#!/usr/bin/env python3
"""Physics/mechanics recipe demo — recover a Van der Pol oscillator.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/recipe_physics_oscillator.py

The Van der Pol oscillator has a nonlinear damping term ``μ·x²·v`` — a *cubic*
feature. The ``general`` default library stops at quadratic and cannot see it,
so a default discovery mis-fits badly. The ``mechanics`` recipe raises the
polynomial library to degree 3, and discovery recovers the true law exactly.
This example runs both to make the recipe's value concrete.

True system (μ=1.0), written as a 2-D first-order system::

    dx/dt = v
    dv/dt = μ·v − μ·x²·v − x        (= μ·(1 − x²)·v − x)
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


def _min_r2(explanation: lawsynth.Explanation) -> float:
    return min(metrics["r_squared"] for metrics in explanation.fit.values())


def main() -> None:
    mu = 1.0

    def van_der_pol(_t: float, state: Sequence[float]) -> list[float]:
        x, v = state
        return [v, mu * (1.0 - x * x) * v - x]

    times, (position, velocity) = integrate(
        van_der_pol, [2.0, 0.0], dt=0.01, steps=4000, sample=5
    )

    recipe = lawsynth.recipes.get("mechanics")
    print("=" * 70)
    print(recipe.describe())
    print("=" * 70, "\n")

    study = lawsynth.Study.from_columns(
        times, {"x": position, "v": velocity}, state=["x", "v"], name="van_der_pol"
    )

    # Baseline: the general default library (degree 2) misses the cubic term.
    default_fit = study.discover(recipe="general").explain()
    default_v_law = next(law for law in default_fit.laws if law.target == "v")
    print(f"general recipe (quadratic library): min R² = {_min_r2(default_fit):.4f}  "
          "<- cannot see the cubic damping term")
    print(f"    {default_v_law.readable}\n")

    # Mechanics recipe: degree-3 library recovers the true nonlinear law.
    discovery = study.discover(recipe="mechanics")
    explanation = discovery.explain()

    print("Recovered laws with the mechanics recipe:")
    print("  true   dv/dt = -x + v - x·x·v")
    for law in explanation.laws:
        print(f"  found  {law.readable}")
    print()

    print("Fit quality (forward simulation vs. observations):")
    for state, metrics in sorted(explanation.fit.items()):
        print(f"  {state}: R² = {metrics['r_squared']:.5f}, RMSE = {metrics['rmse']:.4g}")
    print()

    # Forecast the limit cycle forward beyond the observed window.
    trajectory = study.simulate(horizon=30.0)
    print(f"Forecast: simulated {len(trajectory.time)} points to t=30; "
          f"final (x, v) = ({trajectory.values['x'][-1]:.3f}, "
          f"{trajectory.values['v'][-1]:.3f})")


if __name__ == "__main__":
    main()
