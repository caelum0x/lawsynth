#!/usr/bin/env python3
"""Analyse a discovered world with the CLI-backed engine analyses.

Run it (from the repository root)::

    cargo build -p lawsynth-cli            # build the CLI once (produces target/debug/lawsynth)
    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/analyze_world.py

This example is a presentation layer over the ``lawsynth`` CLI — the Rust engine
is the single source of truth for every number below. It shows the three analyses
:mod:`lawsynth.analysis` exposes:

* ``domains()`` / ``domain_run(name)`` — list the curated presets and run a
  preset's clean round-trip recovery;
* ``stability(world, box=...)`` — locate and classify the fixed points of a
  discovered world (needs a ``.lsworld`` bundle, which we discover here);
* ``discover_controlled(csv, states=..., controls=...)`` — fit a forced model
  ``dx/dt = f(x, u)`` and validate it in-sample.

Everything is deterministic and offline. When the CLI binary is not built the
example degrades honestly: it prints the concrete fix and moves on, rather than
fabricating a result.
"""

from __future__ import annotations

import math
import subprocess
import tempfile
from pathlib import Path

import lawsynth
from lawsynth import analysis


def _write_csv(path: Path, header: list[str], rows: list[list[float]]) -> None:
    import csv

    with path.open("w", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(header)
        writer.writerows(rows)


def _discover_stable_node(binary: Path, workdir: Path) -> Path:
    """Discover a stable node dx/dt=-x, dy/dt=-2y and return its .lsworld path."""
    rows = [[i * 0.02, math.exp(-i * 0.02), math.exp(-2 * i * 0.02)] for i in range(400)]
    csv_path = workdir / "node.csv"
    world = workdir / "node.lsworld"
    _write_csv(csv_path, ["time", "x", "y"], rows)
    completed = subprocess.run(
        [str(binary), "discover", str(csv_path), "--time", "time",
         "--state", "x,y", "--output", str(world), "--degree", "1"],
        capture_output=True, text=True, check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"discover failed: {completed.stderr.strip()}")
    return world


def _forced_dataset(workdir: Path) -> Path:
    """A forced linear system dx/dt = -x + u with u = cos(t) (RK4-integrated)."""
    x, t, dt = 1.0, 0.0, 0.02
    rows: list[list[float]] = []
    for _ in range(400):
        rows.append([t, x, math.cos(t)])
        k1 = -x + math.cos(t)
        k2 = -(x + dt / 2 * k1) + math.cos(t + dt / 2)
        k3 = -(x + dt / 2 * k2) + math.cos(t + dt / 2)
        k4 = -(x + dt * k3) + math.cos(t + dt)
        x = x + dt / 6 * (k1 + 2 * k2 + 2 * k3 + k4)
        t += dt
    csv_path = workdir / "forced.csv"
    _write_csv(csv_path, ["time", "x", "u"], rows)
    return csv_path


def main() -> None:
    print("=" * 72)
    print("LawSynth engine analyses via the CLI — stability / control / domains")
    print("=" * 72)

    # The whole client short-circuits to a MissingBinaryError when the CLI is not
    # built. Probe once up front so we can degrade honestly.
    try:
        binary = analysis._locate_binary()
    except analysis.MissingBinaryError as error:
        print(f"\nCLI not available: {error}")
        print("Build it with `cargo build -p lawsynth-cli`, then re-run this example.")
        return
    print(f"\nUsing CLI binary: {binary}")

    # ---- (1) domains ------------------------------------------------------ #
    print("\n" + "-" * 72)
    print("(1) Curated domain presets")
    names = lawsynth.domains()
    print(f"  presets: {', '.join(names)}")
    if names:
        run = lawsynth.domain_run(names[0])
        verdict = "recovered" if run["recovered"] else "incomplete"
        print(f"  round-trip '{names[0]}': {verdict} (clean synthetic data)")
        for entry in run["recovery"]:
            print(f"    {entry['state']}: RHS RMSE = {entry['rhs_rmse']:.2e}")

    with tempfile.TemporaryDirectory(prefix="lawsynth-analyze-") as tmp:
        workdir = Path(tmp)

        # ---- (2) stability ------------------------------------------------ #
        print("\n" + "-" * 72)
        print("(2) Fixed-point / linear-stability analysis")
        world = _discover_stable_node(binary, workdir)
        report = lawsynth.stability(world, box=[(-1.0, 1.0), (-1.0, 1.0)])
        print(f"  states: {', '.join(report.states)}")
        print(f"  seeds:  {report.seeds_converged}/{report.seeds_total} converged")
        for number, point in enumerate(report.fixed_points, start=1):
            coords = point.at(report.states)
            eigen = ", ".join(f"{e.re:.3g}{e.im:+.3g}i" for e in point.eigenvalues)
            print(f"  #{number} at {coords}: {point.classification}")
            print(f"      eigenvalues: {eigen}")

        # ---- (3) controlled (SINDYc) discovery ---------------------------- #
        print("\n" + "-" * 72)
        print("(3) Controlled discovery of a forced system dx/dt = f(x, u)")
        dataset = _forced_dataset(workdir)
        model = lawsynth.discover_controlled(
            dataset, states=["x"], controls=["u"], degree=1, validate=True
        )
        for equation in model.equations:
            print(f"  d/dt {equation.state} = {equation.expression()}")
        if model.validation is not None:
            print(f"  in-sample R2 = {model.validation.aggregate_r_squared:.6f}, "
                  f"RMSE = {model.validation.aggregate_rmse:.2e}")
            print("  note: in-sample (same data fitted); open-loop error grows with horizon.")

    print("\n" + "=" * 72)
    print("All numbers are computed by the Rust engine; Python only parses/presents.")


if __name__ == "__main__":
    main()
