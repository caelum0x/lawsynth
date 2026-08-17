#!/usr/bin/env python3
"""Epidemiology recipe demo — recover an SIR epidemic from prevalence data.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/recipe_epidemiology_sir.py

The SIR model's transmission term ``β·S·I`` is bilinear with a small
coefficient. The ``general`` default threshold (0.05) can prune it away; the
``epidemiology`` recipe uses a finer threshold (0.01) so the term survives and
discovery recovers β and γ. The recovered R (removed) fraction is 1 − S − I, so
tracking S and I fully determines the epidemic.

True system (β=0.6, γ=0.15), populations normalised to fractions of 1::

    dS/dt = −β·S·I                (susceptibles infected)
    dI/dt =  β·S·I − γ·I          (new infections minus recoveries)
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
    beta, gamma = 0.6, 0.15

    def sir(_t: float, state: Sequence[float]) -> list[float]:
        s, i = state
        return [-beta * s * i, beta * s * i - gamma * i]

    times, (susceptible, infected) = integrate(
        sir, [0.99, 0.01], dt=0.05, steps=4000, sample=10
    )

    recipe = lawsynth.recipes.get("epidemiology")
    print("=" * 70)
    print(recipe.describe())
    print("=" * 70, "\n")

    study = lawsynth.Study.from_columns(
        times, {"S": susceptible, "I": infected}, state=["S", "I"], name="sir_epidemic"
    )

    # Discover with the epidemiology recipe (fine threshold keeps β·S·I).
    discovery = study.discover(recipe="epidemiology")
    explanation = discovery.explain()

    print("Recovered laws (compare to the true system in the docstring):")
    print(f"  true   dS/dt = -{beta}·S·I")
    print(f"  true   dI/dt =  {beta}·S·I - {gamma}·I")
    for law in explanation.laws:
        print(f"  found  {law.readable}")
    print()

    print("Fit quality (forward simulation vs. observations):")
    for state, metrics in sorted(explanation.fit.items()):
        print(f"  {state}: R² = {metrics['r_squared']:.5f}, RMSE = {metrics['rmse']:.4g}")
    print()

    # Scenario board: compare outbreak sizes for lower starting susceptibility
    # (e.g. the effect of prior vaccination reducing S).
    study.add_scenario("vaccinated_30pct", interventions={"S": 0.69})
    study.add_scenario("vaccinated_50pct", interventions={"S": 0.49})
    comparison = study.compare_scenarios(horizon=200.0)
    print("Scenario board — final state after t=200 (lower S0 = smaller epidemic):")
    print(comparison.table())


if __name__ == "__main__":
    main()
