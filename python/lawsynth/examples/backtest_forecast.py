#!/usr/bin/env python3
"""Rigorous rolling-origin backtesting of a discovered LawSynth world.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/backtest_forecast.py

Discovery tells you how well a world *fits*; a backtest tells you how well it
*forecasts*. This demo generates a deterministic damped oscillator, discovers its
laws, then walks a forecast origin across the observed series — simulating forward
from each origin and scoring the prediction against what actually happened. It
prints per-state out-of-sample accuracy (RMSE / MAE / R²), the skill-vs-horizon
decay curve, an overall verdict, and writes a self-contained HTML report with an
inline skill-vs-horizon SVG chart. Everything is deterministic and offline.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import lawsynth


def _damped_oscillator(dt: float = 0.02, steps: int = 1200) -> tuple[list[float], dict[str, list[float]]]:
    """A damped harmonic oscillator x'' = -k·x - c·x' as a 2-D first-order system."""
    k, c = 1.0, 0.3
    x, v = 1.0, 0.0
    time: list[float] = []
    columns: dict[str, list[float]] = {"x": [], "v": []}
    for i in range(steps):
        time.append(i * dt)
        columns["x"].append(x)
        columns["v"].append(v)

        def deriv(x_: float, v_: float) -> tuple[float, float]:
            return v_, -k * x_ - c * v_

        k1x, k1v = deriv(x, v)
        k2x, k2v = deriv(x + 0.5 * dt * k1x, v + 0.5 * dt * k1v)
        k3x, k3v = deriv(x + 0.5 * dt * k2x, v + 0.5 * dt * k2v)
        k4x, k4v = deriv(x + dt * k3x, v + dt * k3v)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        v += dt / 6 * (k1v + 2 * k2v + 2 * k3v + k4v)
    return time, columns


def main() -> None:
    time, columns = _damped_oscillator()
    study = lawsynth.Study.from_columns(time, columns, state=["x", "v"], name="damped_oscillator")

    # 1. Discover the governing laws.
    discovery = study.discover(threshold=0.05)
    print("discovered equations:")
    for target, expression in discovery.equations.items():
        print(f"  d{target}/dt = {expression}")
    print()

    # 2. Backtest — rolling-origin walk-forward forecast evaluation.
    #    5 origins evenly spaced across the series, each forecasting 40 steps ahead.
    result = study.backtest(origins=5, horizon=40)
    print("=" * 72)
    print(result.to_text())
    print("=" * 72, "\n")

    # 3. Inspect the skill-vs-horizon decay directly (mean |error| per lead).
    print("skill-vs-horizon (mean |error| across states), sampled every 8 leads:")
    for h, err in zip(result.leads, result.skill_combined):
        if h % 8 == 1 or h == result.horizon:
            print(f"  lead h={h:>3}: mean|error| = {err:.3e}")
    print(f"\noverall verdict: {result.verdict} "
          f"(mean R² = {result.mean_r_squared:.4f}; error grows {result.decay:.1f}x over the horizon)\n")

    # 4. Determinism check — identical inputs reproduce an identical backtest.
    again = study.backtest(origins=5, horizon=40)
    assert again.to_dict() == result.to_dict()
    print("determinism: re-running the backtest reproduced identical scores.")

    # 5. Share — a self-contained HTML report with the skill-vs-horizon SVG.
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth_backtest_"))
    html = workdir / "backtest.html"
    html.write_text(result._repr_html_(), encoding="utf-8")
    print(f"HTML report (skill-vs-horizon chart + tables): {html}  "
          f"({html.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
