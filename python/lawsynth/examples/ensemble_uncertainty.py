#!/usr/bin/env python3
"""Ensemble discovery — which discovered terms are robust, and which are noise?

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/ensemble_uncertainty.py

A single discovery gives one set of coefficients and no sense of how trustworthy
they are. ``Study.discover_ensemble(n=..., fraction=..., seed=...)`` re-discovers
the world on many deterministic bootstrap resamples and reports, per law term,
its selection frequency and coefficient spread — so the true terms (selected in
every resample, tight spread) are cleanly separated from spurious ones that only
survive on some samples. It also turns member disagreement into a forecast band.
Deterministic and offline: the same seed reproduces the ensemble exactly.
"""

from __future__ import annotations

import random

import lawsynth


def _lotka_volterra(dt: float, steps: int, *, alpha=1.1, beta=0.4, delta=0.1, gamma=0.4):
    """Deterministic RK4 predator/prey: dx/dt = αx - βxy, dy/dt = δxy - γy."""
    x, y = 10.0, 5.0
    times, xs, ys = [], [], []
    for i in range(steps):
        times.append(i * dt)
        xs.append(x)
        ys.append(y)

        def deriv(x_: float, y_: float) -> tuple[float, float]:
            return alpha * x_ - beta * x_ * y_, delta * x_ * y_ - gamma * y_

        k1x, k1y = deriv(x, y)
        k2x, k2y = deriv(x + 0.5 * dt * k1x, y + 0.5 * dt * k1y)
        k3x, k3y = deriv(x + 0.5 * dt * k2x, y + 0.5 * dt * k2y)
        k4x, k4y = deriv(x + dt * k3x, y + dt * k3y)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        y += dt / 6 * (k1y + 2 * k2y + 2 * k3y + k4y)
    return times, xs, ys


def _corrupt(values, *, sigma: float, seed: int):
    rng = random.Random(seed)
    return [value + rng.gauss(0.0, sigma) for value in values]


def main() -> None:
    times, xs, ys = _lotka_volterra(dt=0.02, steps=1000)
    noisy_x = _corrupt(xs, sigma=0.15, seed=3)
    noisy_y = _corrupt(ys, sigma=0.15, seed=5)

    study = lawsynth.Study.from_columns(
        times, {"x": noisy_x, "y": noisy_y}, state=["x", "y"], name="predator_prey"
    )
    print("True system:  dx/dt = 1.1·x - 0.4·x·y      dy/dt = 0.1·x·y - 0.4·y")
    print(f"Observations: {len(times)} samples, additive noise σ=0.15\n")

    ensemble = study.discover_ensemble(n=20, fraction=0.7, seed=0, threshold=0.05)
    print(ensemble.to_text())

    # Forecast band from the ensemble members (lower / median / upper).
    band = ensemble.forecast(horizon=8.0)
    print()
    print(f"Forecast band ({band.members} members) — final-state spread:")
    for state in band.states:
        lo, md, up = band.lower[state][-1], band.median[state][-1], band.upper[state][-1]
        print(f"  {state}(t=end): p10={lo:.3g}  median={md:.3g}  p90={up:.3g}  (band width {up - lo:.3g})")

    # Determinism check: a second ensemble with the same seed is identical.
    again = study.discover_ensemble(n=20, fraction=0.7, seed=0, threshold=0.05)
    assert again.to_dict()["terms"] == ensemble.to_dict()["terms"], "ensemble must be deterministic"
    print("\ndeterminism verified: identical seed -> identical ensemble.")


if __name__ == "__main__":
    main()
