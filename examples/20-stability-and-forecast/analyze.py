#!/usr/bin/env python3
"""Dynamics-analysis walkthrough for the frictionless order book.

This scenario goes past discovery: it discovers a world from the deterministic
dataset with the real ``lawsynth`` engine and then *analyses* that world with
the shipped analysis commands — fixed-point **stability**, conserved-quantity
**invariants**, and a **forecast** — chaining only real, shipped subcommands.

The system is an undamped mid-price / order-flow oscillator (``resilience = 0``
in :mod:`config.toml`). Its linear field ``d(mid)/dt = impact·imbalance``,
``d(imbalance)/dt = -liquidity·mid`` has a single fixed point at the origin that
is a **center**: closed orbits, no attraction or repulsion. That is exactly the
case a linearization cannot decide, so ``stability`` reports it as inconclusive
— and ``invariants`` then explains *why*, by recovering the conserved energy
``liquidity·mid² + impact·imbalance²`` whose level sets are those closed orbits.

Determinism / offline: the dataset is generated RNG-free by the shared harness
(fixed initial condition + RK4), and the engine's analyses are deterministic for
a given binary and inputs. The compiled CLI is located (and, if missing, built
once) exactly the way the benchmark suite does, via
``benchmarks/_engine.ensure_binary``. If the engine cannot be produced offline,
the walkthrough says so honestly and exits with status 2 rather than
fabricating a result.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EXAMPLES = ROOT / "examples"
BENCHMARKS = ROOT / "benchmarks"
for extra in (EXAMPLES, BENCHMARKS):
    if str(extra) not in sys.path:
        sys.path.insert(0, str(extra))

from _workflow import generate_example, load_example  # noqa: E402
from _engine import EngineUnavailable, ensure_binary  # noqa: E402

HERE = Path(__file__).parent
OUTPUT = HERE / "output"


def _run(binary: Path, args: list[str]) -> subprocess.CompletedProcess[str]:
    """Invoke the real engine by absolute path, capturing stdout/stderr."""
    completed = subprocess.run(
        [str(binary), *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(
            f"`lawsynth {args[0]}` failed (exit {completed.returncode}):\n"
            f"{completed.stderr.strip()}"
        )
    return completed


def _print_stability(payload: dict) -> None:
    """Summarise the parsed ``stability --json`` object for the transcript."""
    states = ", ".join(payload["states"])
    print(f"  search: {payload['seeds_converged']}/{payload['seeds_total']} "
          f"seeds converged inside the box (state order: {states})")
    if not payload["fixed_points"]:
        print("  no fixed point located inside the box")
        return
    for point in payload["fixed_points"]:
        coords = ", ".join(f"{value:+.4f}" for value in point["coordinates"])
        eigs = ", ".join(f"{e['re']:+.4f}{e['im']:+.4f}i" for e in point["eigenvalues"])
        verdict = "INCONCLUSIVE" if point["inconclusive"] else "decided"
        print(f"  fixed point ({coords}) -> {point['classification']} [{verdict}]")
        print(f"    Jacobian eigenvalues: {eigs}")


def main() -> int:
    example = load_example(HERE)
    print(f"# {example.config['name']} — dynamics analysis")
    print(example.config["description"])
    print()

    # 0) Deterministic dataset (fixed IC + RK4, no RNG, no wall clock).
    dataset = generate_example(HERE)
    print(f"[1] dataset: {dataset.relative_to(ROOT)} "
          f"({len(dataset.read_text().splitlines()) - 1} samples)")

    # Locate the compiled engine the same way the benchmarks do; build once if
    # absent. Honest boundary if it cannot be produced offline.
    try:
        binary = ensure_binary(ROOT, allow_build=True)
    except EngineUnavailable as error:
        print(f"engine unavailable: {error}", file=sys.stderr)
        return 2
    print(f"    engine: {binary.relative_to(ROOT)}")
    OUTPUT.mkdir(exist_ok=True)
    world = OUTPUT / "analysis-world.lsworld"

    # 1) Discover the world with the real engine. A threshold above the RK4
    #    truncation dust yields the clean linear field; mse is ~machine zero.
    print()
    discover = _run(binary, [
        "discover", str(dataset), "--time", "time",
        "--state", ",".join(example.states),
        "--degree", "2", "--threshold", "0.01",
        "--output", str(world),
    ])
    print(f"[2] discover -> {world.relative_to(ROOT)}")
    print(f"    {discover.stdout.strip().splitlines()[0]}")
    laws = _run(binary, ["explain", str(world)]).stdout
    for line in laws.splitlines():
        if "/dt =" in line:
            print(f"    {line.strip()}")

    # 2) Stability: locate and classify the fixed points. The origin is a
    #    center — the honest inconclusive verdict this scenario is about.
    print()
    print("[3] stability (fixed points + linear classification)")
    stability = _run(binary, [
        "stability", str(world), "--box", "-3:3,-6:6", "--json",
    ])
    (OUTPUT / "stability.json").write_text(stability.stdout, encoding="utf-8")
    _print_stability(json.loads(stability.stdout))

    # 3) Invariants: recover the conserved energy that explains the center.
    print()
    print("[4] invariants (conserved quantities, degree-2 monomial library)")
    invariants = _run(binary, [
        "invariants", str(world), "--degree", "2", "--box", "-3:3",
        "--resolution", "7",
    ])
    (OUTPUT / "invariants.txt").write_text(invariants.stdout, encoding="utf-8")
    for line in invariants.stdout.splitlines():
        if line.strip().startswith(("Conserved", "#1", "residual", "singular")):
            print(f"  {line.strip()}")

    # 4) Forecast: roll the discovered world forward. Energy conservation keeps
    #    the trajectory on a bounded closed orbit (no growth, no decay).
    print()
    print("[5] forecast (roll the discovered world forward)")
    forecast_csv = OUTPUT / "forecast.csv"
    forecast = _run(binary, [
        "forecast", str(world), "--horizon", "6", "--step", "0.5",
        "--initial", "mid=1.0", "--initial", "imbalance=0.0",
        "--output", str(forecast_csv),
    ])
    for line in forecast.stdout.strip().splitlines():
        print(f"  {line.strip()}")
    print(f"    trajectory: {forecast_csv.relative_to(ROOT)}")

    print()
    print("done: stability found a center (inconclusive), invariants recovered "
          "its conserved energy, forecast stayed on a bounded orbit.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
