#!/usr/bin/env python3
"""Log a LawSynth discovery run to MLflow / Weights & Biases (with a dep-free anchor).

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/track_run.py

The example discovers a damped linear oscillator, builds the dependency-free
:class:`lawsynth.RunRecord` — a deterministic snapshot of the run's params,
metrics and ``.lsworld`` artifact — prints it as JSON, and then shows the honest
"backend absent -> install to enable" path for both trackers.

``mlflow``/``wandb`` are optional. When one is absent the example prints an honest
install hint and moves on; the RunRecord anchor is built and shown regardless,
because it needs neither backend and performs no network I/O. Everything is
deterministic and offline.
"""

from __future__ import annotations

from typing import Sequence

import lawsynth


def _rk4(deriv, y0: Sequence[float], *, dt: float, steps: int):
    """Deterministic fixed-step RK4 — the same scheme the engine integrates."""
    y = list(y0)
    t = 0.0
    times: list[float] = []
    columns = [[] for _ in y0]
    for _ in range(steps):
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


def _discover_oscillator():
    """Discover a damped linear oscillator: dx/dt=v, dv/dt=-k*x - c*v."""
    k, c = 4.0, 0.3

    def spring(_t, s):
        x, v = s
        return [v, -k * x - c * v]

    times, (x, v) = _rk4(spring, [1.0, 0.0], dt=0.01, steps=2000)
    study = lawsynth.Study.from_columns(
        times, {"x": x, "v": v}, state=["x", "v"], name="damped_oscillator"
    )
    return study.discover(recipe="mechanics")


def _try_import(name: str):
    try:
        return __import__(name)
    except ImportError:
        return None


def main() -> None:
    print("=" * 72)
    print("LawSynth experiment-tracking interop — RunRecord + MLflow / W&B")
    print("=" * 72)

    discovery = _discover_oscillator()
    print("\nDiscovered laws (native expression strings):")
    for target, expression in sorted(discovery.equations.items()):
        print(f"  d{target}/dt = {expression}")

    # ---- the dependency-free anchor -------------------------------------- #
    # Build a deterministic snapshot of params + metrics + the .lsworld artifact.
    # This needs neither mlflow nor wandb and hits no network.
    record = lawsynth.run_record(discovery)
    print("\nRunRecord (deterministic, dependency-free) as JSON:")
    print(record.to_json())

    print("\nMetrics that genuinely exist for this run (nothing fabricated):")
    for key in sorted(record.metrics):
        print(f"  {key:18s} = {record.metrics[key]:.6g}")
    print("\nParams (the actual discovery config):")
    for key in sorted(record.params):
        print(f"  {key:22s} = {record.params[key]}")

    # Determinism is checkable on the spot: rebuild and compare bytes.
    again = lawsynth.run_record(discovery)
    print(f"\nRebuilt record is byte-identical: {again.to_json() == record.to_json()}")

    exercised: list[str] = []
    absent: list[str] = []

    # ---- (a) MLflow ------------------------------------------------------- #
    print("\n" + "-" * 72)
    print("(a) MLflow")
    mlflow = _try_import("mlflow")
    if mlflow is None:
        absent.append("mlflow")
        print("  mlflow is not installed -> install it to enable: `pip install mlflow`")
        print("  With mlflow: run_id = lawsynth.log_to_mlflow(discovery) logs the params,")
        print("  metrics and the .lsworld artifact to your tracking server / local store.")
    else:
        exercised.append("mlflow")
        import tempfile

        with tempfile.TemporaryDirectory() as tmp:
            uri = (__import__("pathlib").Path(tmp) / "mlruns").as_uri()
            run_id = lawsynth.log_to_mlflow(discovery, tracking_uri=uri, run_name="oscillator")
            print(f"  logged MLflow run: {run_id} (local store at {uri})")

    # ---- (b) Weights & Biases -------------------------------------------- #
    print("\n" + "-" * 72)
    print("(b) Weights & Biases")
    wandb = _try_import("wandb")
    if wandb is None:
        absent.append("wandb")
        print("  wandb is not installed -> install it to enable: `pip install wandb`")
        print("  With wandb: run_id = lawsynth.log_to_wandb(discovery, project='lawsynth')")
        print("  logs the config, summary metrics and the .lsworld artifact to your project.")
    else:
        exercised.append("wandb")
        print("  wandb is installed. Logging requires a project + credentials; call")
        print("  lawsynth.log_to_wandb(discovery, project='your-project') to push the run.")

    # ---- summary ---------------------------------------------------------- #
    print("\n" + "=" * 72)
    print(f"Backends exercised: {exercised or ['(none — both optional deps absent)']}")
    print(f"Backends absent   : {absent or ['(none)']}")
    print("The RunRecord anchor was built and shown regardless; it stays offline & "
          "dependency-free.")


if __name__ == "__main__":
    main()
