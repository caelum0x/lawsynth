#!/usr/bin/env python3
"""Generate a deterministic *forced* (controlled) dataset for `lawsynth control`.

The system is a driven damped harmonic oscillator with one exogenous control
input `u(t)`:

    dx/dt = v
    dv/dt = -x - 0.3*v + u,   with   u(t) = sin(0.7*t)

The trajectory is integrated with classical fourth-order Runge-Kutta at a fixed
step. There is no random number generator and no wall-clock read anywhere in
this file, so the emitted `forced-oscillator.csv` is byte-for-byte reproducible
on any machine with a standard Python 3 interpreter (only the `math` module is
used). Re-running this script overwrites the CSV with identical bytes.

Usage:
    python3 gen_forced_oscillator.py            # writes forced-oscillator.csv
"""

from __future__ import annotations

import math
from pathlib import Path

DT = 0.02
STEPS = 600
DAMPING = 0.3


def control(t: float) -> float:
    """The exogenous, measured control signal u(t)."""
    return math.sin(0.7 * t)


def field(state: tuple[float, float], u: float) -> tuple[float, float]:
    """Right-hand side dx/dt = v, dv/dt = -x - 0.3*v + u."""
    x, v = state
    return (v, -x - DAMPING * v + u)


def rk4_step(state: tuple[float, float], t: float) -> tuple[float, float]:
    """One fixed-step RK4 update. The control is sampled inside the step."""
    x, v = state
    k1 = field((x, v), control(t))
    k2 = field((x + 0.5 * DT * k1[0], v + 0.5 * DT * k1[1]), control(t + 0.5 * DT))
    k3 = field((x + 0.5 * DT * k2[0], v + 0.5 * DT * k2[1]), control(t + 0.5 * DT))
    k4 = field((x + DT * k3[0], v + DT * k3[1]), control(t + DT))
    return (
        x + DT * (k1[0] + 2 * k2[0] + 2 * k3[0] + k4[0]) / 6.0,
        v + DT * (k1[1] + 2 * k2[1] + 2 * k3[1] + k4[1]) / 6.0,
    )


def main() -> None:
    state = (1.0, 0.0)
    lines = ["time,x,v,u"]
    t = 0.0
    for _ in range(STEPS + 1):
        x, v = state
        lines.append(f"{t:.12e},{x:.12e},{v:.12e},{control(t):.12e}")
        state = rk4_step(state, t)
        t += DT
    out = Path(__file__).with_name("forced-oscillator.csv")
    out.write_text("\n".join(lines) + "\n")
    print(f"wrote {out.name} ({STEPS + 1} rows, step {DT}, columns time,x,v,u)")


if __name__ == "__main__":
    main()
