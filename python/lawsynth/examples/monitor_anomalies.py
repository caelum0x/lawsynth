#!/usr/bin/env python3
"""Model monitoring — is fresh data still in control, and where did it break?

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/monitor_anomalies.py

A world is discovered from clean observations, then used as a model of *normal
behaviour*. ``Study.monitor(new_data, threshold=...)`` simulates the world over
fresh data, standardizes the residuals with a robust median/MAD scale, and flags
any timestamp beyond ``threshold`` sigma. Clean data reports in-control; a single
injected shock is flagged at exactly the timestamp it was injected. Deterministic
and offline.
"""

from __future__ import annotations

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


def main() -> None:
    # 1. Discover a world from clean training observations.
    times, xs, vs = _oscillator(dt=0.02, steps=900)
    study = lawsynth.Study.from_columns(times, {"x": xs, "v": vs}, state=["x", "v"], name="oscillator")
    study.discover(threshold=0.05)
    print("Discovered model of normal behaviour:")
    for law in study.explain().laws:
        print(f"  {law.readable}")
    print()

    # 2. Monitor a fresh, clean run -> expect IN CONTROL.
    fresh = lawsynth.Dataset.from_columns(times, {"x": xs, "v": vs})
    clean_report = study.monitor(fresh, threshold=4.0)
    print("=== Clean fresh data ===")
    print(clean_report.to_text())
    print()

    # 3. Inject a single-sample shock into x at a known timestamp -> expect a FLAG.
    shock_index = 500
    shock_time = times[shock_index]
    shocked_x = list(xs)
    shocked_x[shock_index] += 0.8  # abrupt sensor spike
    shocked = lawsynth.Dataset.from_columns(times, {"x": shocked_x, "v": vs})
    shock_report = study.monitor(shocked, threshold=4.0)
    print(f"=== Shock injected at t={shock_time:.2f} (index {shock_index}) ===")
    print(shock_report.to_text())

    # 4. Verify the flag landed at the injected timestamp.
    assert clean_report.in_control, "clean data should be in control"
    assert not shock_report.in_control, "shocked data should be flagged"
    flagged = shock_report.flagged_times()
    assert shock_time in flagged, f"expected a flag at t={shock_time}, got {flagged}"
    print(f"\nverdict: clean -> {clean_report.verdict}; shocked -> {shock_report.verdict}")
    print(f"anomaly correctly flagged at injected time t={shock_time:.2f}.")


if __name__ == "__main__":
    main()
