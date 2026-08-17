#!/usr/bin/env python3
"""Compose and edit executable worlds — build coupled systems, then tune them.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/compose_and_edit.py

This demonstrates model *composition* and *editing*, both implemented by
manipulating a world's declarative structure (states / parameters / controls /
laws) and rebuilding a validated native world — the inputs are never mutated.

* :func:`lawsynth.compose` unions two worlds into one coupled system, namespacing
  colliding identifiers.
* :meth:`World.rename`, :meth:`World.set_parameter`, :meth:`World.drop_law` and
  :meth:`World.scale_law` are targeted, immutable edits returning new worlds.

We build two harmonic oscillators with distinct frequencies, compose them into a
single 4-state system that simulates, then rename a parameter, retune it, and
confirm the dynamics actually changed while the world stays valid and runnable.
"""

from __future__ import annotations

import lawsynth
from lawsynth import WorldSpec


def oscillator(omega_squared: float, *, param: str) -> object:
    """A harmonic oscillator dx/dt = v, dv/dt = -omega^2 * x as a native world.

    Building it through :class:`WorldSpec` records the parameter *value* so the
    world round-trips through composition and editing (a raw native world does
    not expose its parameters).
    """
    return WorldSpec.create(
        states=["x", "v"],
        parameters={param: omega_squared},
        equations={"x": "v", "v": f"((-1.0*{param})*x)"},
    ).realize()


def _period_estimate(times, values) -> float:
    """Rough oscillation period from the first upward zero-crossing spacing."""
    crossings = [
        times[i]
        for i in range(1, len(values))
        if values[i - 1] < 0.0 <= values[i]
    ]
    return (crossings[1] - crossings[0]) if len(crossings) >= 2 else float("nan")


def main() -> None:
    print("=" * 72)
    print("Compose: two oscillators -> one coupled 4-state system")
    print("=" * 72)

    slow = oscillator(1.0, param="w2")   # omega = 1.0
    fast = oscillator(9.0, param="w2")   # omega = 3.0

    # Both worlds share identifiers (x, v, w2); prefixes namespace them apart.
    system = lawsynth.compose(slow, fast, prefix_a="s_", prefix_b="f_")
    print("Composed laws:")
    for target, expression in system.equations().items():
        print(f"  d{target}/dt = {expression}")

    trajectory = system.simulate(
        {"s_x": 1.0, "s_v": 0.0, "f_x": 1.0, "f_v": 0.0},
        start=0.0, end=12.0, step=0.001,
    )
    slow_period = _period_estimate(trajectory.time, trajectory.values["s_x"])
    fast_period = _period_estimate(trajectory.time, trajectory.values["f_x"])
    print(f"\nSimulated {len(trajectory.time)} steps to t=12.")
    print(f"  slow sub-system period ~= {slow_period:.3f}  (theory 2*pi/1 = {6.283185:.3f})")
    print(f"  fast sub-system period ~= {fast_period:.3f}  (theory 2*pi/3 = {2.094395:.3f})")
    print(f"  final state: {{{', '.join(f'{k}={v[-1]:+.3f}' for k, v in sorted(trajectory.values.items()))}}}")
    print()

    print("=" * 72)
    print("Edit: rename + set_parameter (immutable, still simulates)")
    print("=" * 72)

    # A damped spring: dpos/dt = vel, dvel/dt = -k*pos - c*vel.
    spring = WorldSpec.create(
        states=["pos", "vel"],
        parameters={"k": 2.0, "c": 0.2},
        equations={"pos": "vel", "vel": "((-1.0*k)*pos) + ((-1.0*c)*vel)"},
    ).realize()
    print("Original spring laws:", spring.equations())

    # rename the stiffness parameter, then stiffen it four-fold — each returns a
    # NEW world; the original is untouched.
    stiffer = spring.rename("k", "stiffness").set_parameter("stiffness", 8.0)
    print("Edited spring laws: ", stiffer.equations())

    ic = {"pos": 1.0, "vel": 0.0}
    base = spring.simulate(ic, start=0.0, end=16.0, step=0.001)
    edited = stiffer.simulate(ic, start=0.0, end=16.0, step=0.001)
    base_period = _period_estimate(base.time, base.values["pos"])
    edited_period = _period_estimate(edited.time, edited.values["pos"])
    print(f"\n  original (k=2)      period ~= {base_period:.3f}")
    print(f"  stiffened (k=8)     period ~= {edited_period:.3f}   (stiffer -> faster)")
    assert edited_period < base_period, "stiffening must shorten the period"
    assert spring.equations() != stiffer.equations(), "edit must produce a distinct world"
    print("  original world unchanged by the edit:", spring.equations()["vel"])
    print()

    print("=" * 72)
    print("Edit: scale_law and drop_law")
    print("=" * 72)
    doubled = spring.scale_law("vel", 2.0)
    print("scale_law(vel, 2.0):", doubled.equations()["vel"])

    # A 3-state world: a coupled oscillator (x, v) plus a decoupled decay z.
    mixed = WorldSpec.create(
        states=["x", "v", "z"],
        parameters={"w2": 4.0, "r": 0.5},
        equations={"x": "v", "v": "((-1.0*w2)*x)", "z": "((-1.0*r)*z)"},
    ).realize()
    # Dropping z removes an unreferenced law, so the remainder simulates directly.
    pruned = mixed.drop_law("z")
    print("drop_law(z) leaves states:", list(pruned.equations().keys()))
    remaining = pruned.simulate({"x": 1.0, "v": 0.0}, start=0.0, end=1.0, step=0.01)
    print(f"  pruned world simulates; final x = {remaining.values['x'][-1]:+.4f}")


if __name__ == "__main__":
    main()
