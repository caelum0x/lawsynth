#!/usr/bin/env python3
"""End-to-end LawSynth Study demo on a small synthetic dataset.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/study_quickstart.py

It generates a deterministic damped-oscillator series, discovers the governing
laws, prints a plain-language explanation, runs a what-if forecast, and writes a
self-contained HTML report you can open in any browser.
"""

from __future__ import annotations

import csv
import math
import tempfile
from pathlib import Path

import lawsynth


def _write_synthetic_csv(path: Path) -> None:
    """A damped harmonic oscillator: x'' = -k·x - c·x', written as a 2-D system.

    dx/dt = v
    dv/dt = -k·x - c·v         (k = 1.0 spring, c = 0.3 damping)
    """
    k, c = 1.0, 0.3
    dt, steps = 0.02, 900
    x, v = 1.0, 0.0
    rows = []
    for i in range(steps):
        t = i * dt
        rows.append((t, x, v))
        # Deterministic RK4 integration of the true system.
        def deriv(x_: float, v_: float) -> tuple[float, float]:
            return v_, -k * x_ - c * v_

        k1x, k1v = deriv(x, v)
        k2x, k2v = deriv(x + 0.5 * dt * k1x, v + 0.5 * dt * k1v)
        k3x, k3v = deriv(x + 0.5 * dt * k2x, v + 0.5 * dt * k2v)
        k4x, k4v = deriv(x + dt * k3x, v + dt * k3v)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        v += dt / 6 * (k1v + 2 * k2v + 2 * k3v + k4v)

    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["time", "x", "v"])
        writer.writerows((f"{t:.6f}", f"{xv:.10f}", f"{vv:.10f}") for t, xv, vv in rows)


def main() -> None:
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth_quickstart_"))
    csv_path = workdir / "oscillator.csv"
    _write_synthetic_csv(csv_path)
    print(f"synthetic observations written to {csv_path}")

    # 1. Observe -> Study.
    study = lawsynth.Study.from_csv(csv_path, time="time", state=["x", "v"], name="damped_oscillator")
    print(study, "\n")

    # 2. Discover the laws.
    discovery = study.discover(threshold=0.05)
    print("discovered equations:")
    for target, expression in discovery.equations.items():
        print(f"  d{target}/dt = {expression}")
    print()

    # 3. Understand — plain-language explanation.
    explanation = study.explain()
    print("=" * 68)
    print(explanation.to_text())
    print("=" * 68, "\n")

    # 4. Use — forecast forward and run a what-if.
    trajectory = study.simulate(horizon=12.0)
    print(f"forecast: {len(trajectory.time)} points; "
          f"final state x={trajectory.values['x'][-1]:.4f}, v={trajectory.values['v'][-1]:.4f}")

    forecast = study.forecast({"x": 2.0}, horizon=12.0)
    print("what-if (release from x=2.0 instead of x=1.0): "
          f"final divergence {dict((k, round(v, 4)) for k, v in forecast.divergence.items())}\n")

    # 5. Share — self-contained HTML report + portable world bundle.
    report_path = study.report(workdir / "report.html")
    world_path = study.save(workdir / "damped_oscillator.lsworld")
    print(f"HTML report : {report_path}  ({report_path.stat().st_size} bytes)")
    print(f"world bundle: {world_path}  ({world_path.stat().st_size} bytes)")

    # Round-trip the bundle to prove persistence works.
    reloaded = lawsynth.Study.load(world_path, dataset=study.dataset, state=["x", "v"])
    assert reloaded.world.equations() == study.world.equations()
    print("bundle round-trip verified.")


if __name__ == "__main__":
    main()
