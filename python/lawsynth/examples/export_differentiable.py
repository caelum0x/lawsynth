#!/usr/bin/env python3
"""Export a discovered world to SymPy and to a differentiable torch/jax layer.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 \
        python/lawsynth/examples/export_differentiable.py

The example discovers a damped linear oscillator, then demonstrates the three
export handles the symbolic-regression ecosystem expects:

* ``to_sympy(world)`` — a SymPy expression per law (``simplify`` / ``latex`` /
  ``lambdify`` from there);
* ``to_torch(world)`` — a ``torch.nn.Module`` whose ``derivatives(t, state)`` is
  differentiable and whose model constants are trainable ``nn.Parameter`` s, so
  the world drops in as a Neural-ODE layer;
* ``to_jax(world)`` — the same as a pure JAX function over a trainable pytree.

``sympy``/``torch``/``jax`` are all optional. When one is absent the example
prints an honest "install X to enable" line and moves on — the discovery and the
dependency-free faithfulness check (``numeric_derivatives`` vs the engine) still
run. Everything is deterministic and offline.
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
    print("LawSynth differentiable export — SymPy + torch/jax (trainable constants)")
    print("=" * 72)

    discovery = _discover_oscillator()
    world = discovery.world
    print("\nDiscovered laws (native expression strings):")
    for target, expression in sorted(discovery.equations.items()):
        print(f"  d{target}/dt = {expression}")

    # Dependency-free faithfulness anchor: the exported dynamics *are* the engine
    # dynamics. Integrating them with RK4 must reproduce world.simulate(...).
    initial = {"x": 1.0, "v": 0.0}
    point = lawsynth.numeric_derivatives(world, initial)
    print("\nnumeric_derivatives (stdlib only) at x=1, v=0:")
    print(f"  {point}")

    def exported(t, s):
        d = lawsynth.numeric_derivatives(world, {"x": s[0], "v": s[1]})
        return [d["x"], d["v"]]

    _, mine = _rk4(exported, [initial["x"], initial["v"]], dt=0.01, steps=100)
    native = world.simulate(dict(initial), start=0.0, end=1.0, step=0.01)
    deviation = max(
        abs(native.values[name][j] - mine[i][j])
        for i, name in enumerate(["x", "v"])
        for j in range(len(mine[i]))
    )
    print(f"  max |exported RK4 - native simulate| over t in [0,1]: {deviation:.2e}"
          "  (machine precision -> faithful)")

    exercised: list[str] = []
    absent: list[str] = []

    # ---- (a) SymPy -------------------------------------------------------- #
    print("\n" + "-" * 72)
    print("(a) SymPy export")
    sympy = _try_import("sympy")
    if sympy is None:
        absent.append("sympy")
        print("  sympy is not installed -> install it to enable: `pip install sympy`")
        print("  With sympy: lawsynth.to_sympy(world) -> {state: sympy.Expr}, then")
        print("  sympy.simplify(...) / sympy.latex(...) / sympy.lambdify(...).")
    else:
        exercised.append("sympy")
        laws = lawsynth.to_sympy(world)
        for target, expr in sorted(laws.items()):
            print(f"  d{target}/dt = {expr}")
            print(f"           simplified: {sympy.simplify(expr)}")
            print(f"           latex     : {sympy.latex(expr)}")

    # ---- (b) PyTorch ------------------------------------------------------ #
    print("\n" + "-" * 72)
    print("(b) PyTorch export (Neural-ODE layer, trainable constants)")
    torch = _try_import("torch")
    if torch is None:
        absent.append("torch")
        print("  torch is not installed -> install it to enable: `pip install torch`")
        print("  With torch: module = lawsynth.to_torch(world); module.constants is an")
        print("  nn.Parameter (requires_grad) and module.derivatives(t, state) is")
        print("  differentiable for embedding as a Neural-ODE and fine-tuning.")
    else:
        exercised.append("torch")
        module = lawsynth.to_torch(world, initial=initial)
        state = torch.tensor([initial["x"], initial["v"]], dtype=torch.float64)
        dstate = module.derivatives(0.0, state)
        print(f"  state_names   : {module.state_names}")
        print(f"  constant_names: {module.constant_names}")
        print(f"  constants     : {module.constants.detach().tolist()}")
        print(f"  constants is nn.Parameter: {isinstance(module.constants, torch.nn.Parameter)}"
              f", requires_grad={module.constants.requires_grad}")
        print(f"  initial is nn.Parameter  : {isinstance(module.initial, torch.nn.Parameter)}"
              f", requires_grad={module.initial.requires_grad}")
        print(f"  derivatives(0, [1,0]) = {dstate.detach().tolist()}")
        ref = lawsynth.numeric_derivatives(world, initial)
        match = max(abs(dstate.detach().tolist()[i] - ref[n]) for i, n in enumerate(module.state_names))
        print(f"  max |torch - engine| at the point: {match:.2e}")
        # A real gradient flows back to the trainable constants.
        loss = (module.derivatives(0.0, state) ** 2).sum()
        loss.backward()
        print(f"  d(loss)/d(constants) = {module.constants.grad.tolist()}  (autograd works)")

    # ---- (c) JAX ---------------------------------------------------------- #
    print("\n" + "-" * 72)
    print("(c) JAX export (pure function over a trainable pytree)")
    jax = _try_import("jax")
    if jax is None:
        absent.append("jax")
        print("  jax is not installed -> install it to enable: `pip install jax`")
        print("  With jax: dyn = lawsynth.to_jax(world); dyn.params['constants'] is a")
        print("  trainable array and dyn.derivatives(dyn.params, t, state) is grad-able.")
    else:
        exercised.append("jax")
        jax.config.update("jax_enable_x64", True)
        jnp = __import__("jax.numpy", fromlist=["numpy"])
        dyn = lawsynth.to_jax(world, initial=initial)
        state = jnp.asarray([initial["x"], initial["v"]], dtype=jnp.float64)
        dstate = dyn.derivatives(dyn.params, 0.0, state)
        print(f"  params keys   : {sorted(dyn.params)}")
        print(f"  constants     : {list(dyn.params['constants'])}")
        print(f"  derivatives(params, 0, [1,0]) = {list(dstate)}")

        def loss(params):
            return jnp.sum(dyn.derivatives(params, 0.0, state) ** 2)

        grads = jax.grad(loss)(dyn.params)
        print(f"  d(loss)/d(constants) = {list(grads['constants'])}  (jax.grad works)")

    # ---- summary ---------------------------------------------------------- #
    print("\n" + "=" * 72)
    print(f"Backends exercised: {exercised or ['(none — all optional deps absent)']}")
    print(f"Backends absent   : {absent or ['(none)']}")
    print("The module imported and ran cleanly regardless; core stays offline & "
          "dependency-free.")


if __name__ == "__main__":
    main()
