#!/usr/bin/env python3
"""Data preparation before discovery — clean noisy observations, recover the law.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/prepare_and_discover.py

A damped harmonic oscillator is sampled and then corrupted with deterministic
measurement noise. Discovery on the *raw* noisy series is thrown off — the
finite-difference derivatives it relies on amplify the noise. Running
``Study.prepare(smooth=...)`` first (a pure standard-library moving-average
smoother) recovers the governing laws with a materially better fit. Everything is
deterministic and offline: identical inputs reproduce this output exactly.
"""

from __future__ import annotations

import random

import lawsynth


def _oscillator(dt: float, steps: int, *, k: float = 1.0, c: float = 0.3):
    """Deterministic RK4 integration of x'' = -k·x - c·x' as a 2-D system."""
    x, v = 1.0, 0.0
    times, xs, vs = [], [], []
    for i in range(steps):
        times.append(i * dt)
        xs.append(x)
        vs.append(v)

        def deriv(x_: float, v_: float) -> tuple[float, float]:
            return v_, -k * x_ - c * v_

        k1x, k1v = deriv(x, v)
        k2x, k2v = deriv(x + 0.5 * dt * k1x, v + 0.5 * dt * k1v)
        k3x, k3v = deriv(x + 0.5 * dt * k2x, v + 0.5 * dt * k2v)
        k4x, k4v = deriv(x + dt * k3x, v + dt * k3v)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        v += dt / 6 * (k1v + 2 * k2v + 2 * k3v + k4v)
    return times, xs, vs


def _corrupt(values, *, sigma: float, seed: int):
    """Add deterministic Gaussian measurement noise (seeded — never the clock)."""
    rng = random.Random(seed)
    return [value + rng.gauss(0.0, sigma) for value in values]


def _fit_line(explanation) -> str:
    return ", ".join(
        f"{state}: R²={metrics['r_squared']:.4f} RMSE={metrics['rmse']:.4g}"
        for state, metrics in sorted(explanation.fit.items())
    )


def _truth_rmse(discovery, clean, times) -> float:
    """Honest metric: RMSE of the discovered law's trajectory vs. the TRUE clean series.

    Simulates the discovered world from the true initial condition over the clean
    grid, so noise in the training data cannot flatter the score.
    """
    span = times[-1] - times[0]
    trajectory = discovery.simulate(initial={"x": 1.0, "v": 0.0}, horizon=span, step=times[1] - times[0])
    total, count = 0.0, 0
    for state, reference in clean.items():
        simulated = trajectory.values[state]
        n = min(len(simulated), len(reference))
        total += sum((simulated[i] - reference[i]) ** 2 for i in range(n))
        count += n
    return (total / count) ** 0.5


def main() -> None:
    times, xs, vs = _oscillator(dt=0.02, steps=900)
    noisy_x = _corrupt(xs, sigma=0.03, seed=7)
    noisy_v = _corrupt(vs, sigma=0.03, seed=11)

    raw = lawsynth.Study.from_columns(
        times, {"x": noisy_x, "v": noisy_v}, state=["x", "v"], name="oscillator_noisy"
    )
    print("True system:  dx/dt = v      dv/dt = -1.0·x - 0.3·v")
    print(f"Observations: {len(times)} samples, additive noise σ=0.03\n")

    # --- Discover on the RAW noisy series -------------------------------- #
    raw_discovery = raw.discover(threshold=0.05)
    raw_explanation = raw_discovery.explain()
    print("RAW (no preparation):")
    for law in raw_explanation.laws:
        print(f"  {law.readable}")
    print(f"  fit -> {_fit_line(raw_explanation)}\n")

    # --- Prepare (moving-average smoothing) then discover ---------------- #
    prepared = raw.prepare(smooth=15)
    prep_discovery = prepared.discover(threshold=0.05)
    prep_explanation = prep_discovery.explain()
    print("PREPARED (Study.prepare(smooth=15) -> discover):")
    for law in prep_explanation.laws:
        print(f"  {law.readable}")
    print(f"  fit -> {_fit_line(prep_explanation)}\n")

    # --- Verdict: reproduce the TRUE clean dynamics ---------------------- #
    clean = {"x": xs, "v": vs}
    raw_rmse = _truth_rmse(raw_discovery, clean, times)
    prep_rmse = _truth_rmse(prep_discovery, clean, times)
    print("Fidelity to the TRUE (noise-free) trajectory — the honest metric:")
    print(f"  RMSE vs truth: raw={raw_rmse:.4g}  ->  prepared={prep_rmse:.4g}  "
          f"({100 * (1 - prep_rmse / raw_rmse):.0f}% lower error)")
    assert prep_rmse < raw_rmse, "preparation should improve fidelity to the true system"
    print("preparation recovered the system with a better fit.")


if __name__ == "__main__":
    main()
