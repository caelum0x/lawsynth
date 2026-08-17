#!/usr/bin/env python3
"""Symbolic simplification of a discovered world — honest and verified.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/simplify_world.py

LawSynth discovers laws as arithmetic expression strings with inline
coefficients, e.g. ``((1.04*x)+(-0.39*(x*y)))``. :func:`lawsynth.simplify`
rewrites each law into a canonical, minimal form using a pure-``ast`` algebraic
simplifier (constant folding, identity collapse, like-term combination, sign
normalisation) and returns a *new equivalent world*.

The guarantee is honest: the simplified world is mathematically equal to the
original, which we prove by simulating both and reporting the maximum trajectory
deviation (at the level of floating-point round-off). A freshly discovered world
is already sparse, so simplifying it mostly canonicalises; the folding power
becomes obvious once a law has been *edited* (here, rescaled) into a redundant
form — simplify collapses it with the trajectory unchanged.
"""

from __future__ import annotations

from typing import Callable, Sequence

import lawsynth


def integrate(
    deriv: Callable[[float, Sequence[float]], list[float]],
    y0: Sequence[float],
    *,
    dt: float,
    steps: int,
    sample: int = 1,
) -> tuple[list[float], list[list[float]]]:
    """Deterministic RK4 integration; returns (times, per-state value columns)."""
    y = list(y0)
    t = 0.0
    times: list[float] = []
    columns: list[list[float]] = [[] for _ in y0]
    for i in range(steps):
        if i % sample == 0:
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


def _report(title: str, simplified: "lawsynth.SimplifiedWorld", initial: dict[str, float]) -> None:
    print("=" * 72)
    print(title)
    print("=" * 72)
    for law in simplified.laws:
        note = "" if law.changed else "   (already minimal)"
        print(f"  d{law.target}/dt   {law.before_nodes} -> {law.after_nodes} AST nodes{note}")
        print(f"      before:  {law.before}")
        print(f"      after:   {law.after}")
    print(
        f"  total complexity: {simplified.before_nodes} -> {simplified.after_nodes} nodes "
        f"({simplified.reduced:+d}, {simplified.reduction_ratio:.0%} smaller)"
    )
    deviation = simplified.verify(initial, end=8.0, step=0.01)
    print(f"  equivalence: max trajectory deviation over t in [0, 8] = {deviation:.3e}")
    assert deviation < 1e-9, "simplification must be mathematically equivalent"
    print()


def main() -> None:
    # A Lotka–Volterra predator–prey system, observed and then discovered.
    alpha, beta, delta, gamma = 1.1, 0.4, 0.1, 0.4

    def lotka_volterra(_t: float, state: Sequence[float]) -> list[float]:
        x, y = state
        return [alpha * x - beta * x * y, delta * x * y - gamma * y]

    times, (prey, predator) = integrate(
        lotka_volterra, [1.5, 1.0], dt=0.01, steps=4000, sample=4
    )

    study = lawsynth.Study.from_columns(
        times, {"x": prey, "y": predator}, state=["x", "y"], name="lotka_volterra"
    )
    discovery = study.discover(recipe="ecology")

    print("Discovered laws (native inline-coefficient form):")
    for target, expression in discovery.equations.items():
        print(f"  d{target}/dt = {expression}")
    print()

    initial = {"x": 1.5, "y": 1.0}

    # 1) Simplify the freshly discovered world (already sparse -> canonicalise).
    _report("Simplify the discovered world", lawsynth.simplify(discovery.world), initial)

    # 2) Edit the world into a redundant form, then simplify — the folding case.
    #    scale_law is a purely structural edit (wrap the RHS in a factor); simplify
    #    then folds the nested constants back down. Trajectory is unchanged.
    rescaled = discovery.world.scale_law("x", 3.0).scale_law("y", 3.0)
    print("After scale_law(x, 3.0) and scale_law(y, 3.0) — redundant nested form:")
    for target, expression in rescaled.equations().items():
        print(f"  d{target}/dt = {expression}")
    print()
    _report("Simplify the rescaled world (constant folding + distribution)",
            lawsynth.simplify(rescaled), initial)

    # The Study/DiscoveryResult façades expose the same operation.
    via_study = study.simplify()
    print(f"Study.simplify() -> {via_study!r}; its .world simulates and is equivalent "
          f"(max deviation {via_study.verify(initial, end=4.0):.3e}).")


if __name__ == "__main__":
    main()
