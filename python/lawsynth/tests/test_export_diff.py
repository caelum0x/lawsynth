"""Tests for SymPy + differentiable torch/jax export (``lawsynth.export``).

The module must import with none of sympy/torch/jax installed; dependency-free
behaviour (IR compilation, ``numeric_derivatives``, the clear missing-dependency
error) is tested unconditionally, while the backend builders use
``pytest.importorskip`` so they skip cleanly when a dependency is absent.
"""

from __future__ import annotations

import math

import pytest

import lawsynth
from lawsynth import export
from lawsynth.equation import Equation
from lawsynth.errors import LawSynthError
from lawsynth.variable import Variable
from lawsynth.world import build_world
from lawsynth.worldspec import WorldSpec


def _native_available() -> bool:
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name == "lawsynth._native":
            return False
        raise
    return True


native = pytest.mark.skipif(not _native_available(), reason="native extension not built")


def _rk4(deriv, y0, *, dt, steps):
    y = list(y0)
    t = 0.0
    times, columns = [], [[] for _ in y0]
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
    """Deterministically discover a damped linear oscillator."""
    k, c = 4.0, 0.3

    def spring(_t, s):
        x, v = s
        return [v, -k * x - c * v]

    times, (x, v) = _rk4(spring, [1.0, 0.0], dt=0.01, steps=2000)
    study = lawsynth.Study.from_columns(times, {"x": x, "v": v}, state=["x", "v"], name="osc")
    return study.discover(recipe="mechanics")


def _trig_world():
    """A directly-built native world exercising sin, division and power."""
    states = [Variable("x", "state"), Variable("v", "state")]
    equations = [Equation("x", "(v/2.0)"), Equation("v", "((-3.0*sin(x)) + (x^2))")]
    return build_world(states, {}, equations, [])


# --------------------------------------------------------------------------- #
# Import & error surface (no optional dependency required)                     #
# --------------------------------------------------------------------------- #


def test_module_imports_without_optional_dependencies():
    # Importing the module and the public names must not require sympy/torch/jax.
    assert callable(lawsynth.to_sympy)
    assert callable(lawsynth.to_torch)
    assert callable(lawsynth.to_jax)
    assert callable(lawsynth.numeric_derivatives)
    assert issubclass(export.MissingDependencyError, LawSynthError)
    assert issubclass(export.MissingDependencyError, ImportError)


def test_missing_dependency_raises_clear_typed_error():
    with pytest.raises(export.MissingDependencyError) as info:
        export._require("lawsynth_absent_backend_xyz", "World.to_torch()")
    message = str(info.value)
    assert "lawsynth_absent_backend_xyz" in message
    assert "pip install" in message
    assert "World.to_torch()" in message
    # It is catchable both as a LawSynth error and as a plain ImportError.
    assert isinstance(info.value, LawSynthError)
    assert isinstance(info.value, ImportError)


@native
def test_backend_absence_paths_raise_on_real_worlds():
    """When a backend is genuinely absent, the exporter must say so clearly."""
    world = _discover_oscillator().world
    for name, call in (
        ("sympy", lambda: lawsynth.to_sympy(world)),
        ("torch", lambda: lawsynth.to_torch(world)),
        ("jax", lambda: lawsynth.to_jax(world)),
    ):
        if _module_absent(name):
            with pytest.raises(export.MissingDependencyError) as info:
                call()
            assert name in str(info.value)


def _module_absent(name: str) -> bool:
    try:
        __import__(name)
    except ImportError:
        return True
    return False


class _StubWorld:
    """A minimal world exposing ``equations()`` — enough to drive the exporter.

    Lets us feed the export layer expressions the native engine would never emit,
    so its own guard rails (unsupported function / operator) are covered without a
    built native extension.
    """

    def __init__(self, equations):
        self._equations = dict(equations)

    def equations(self):
        return dict(self._equations)


def test_unsupported_function_raises_export_error():
    world = _StubWorld({"x": "tan(x)"})
    with pytest.raises(export.ExportError) as info:
        lawsynth.numeric_derivatives(world, {"x": 0.1})
    assert "tan" in str(info.value)


def test_unsupported_operator_raises_export_error():
    world = _StubWorld({"x": "x % 2"})
    with pytest.raises(export.ExportError):
        lawsynth.numeric_derivatives(world, {"x": 0.1})


def test_to_sympy_rejects_unsupported_function():
    pytest.importorskip("sympy")
    world = _StubWorld({"x": "tan(x)"})
    with pytest.raises(export.ExportError):
        lawsynth.to_sympy(world)


# --------------------------------------------------------------------------- #
# Dependency-free numeric reference & IR                                       #
# --------------------------------------------------------------------------- #


@native
def test_numeric_derivatives_match_discovered_engine_dynamics():
    world = _discover_oscillator().world
    point = lawsynth.numeric_derivatives(world, {"x": 1.0, "v": 0.0})
    # dx/dt = a*v with a ~ 1 and v = 0 -> ~0; dv/dt = -k*x with k ~ 4, x = 1 -> ~ -4.
    assert abs(point["x"]) < 1e-6
    assert -4.05 < point["v"] < -3.95


@native
def test_exported_dynamics_reproduce_native_simulate_to_machine_precision():
    world = _discover_oscillator().world
    initial = {"x": 1.0, "v": 0.0}

    def exported(_t, s):
        d = lawsynth.numeric_derivatives(world, {"x": s[0], "v": s[1]})
        return [d["x"], d["v"]]

    _, mine = _rk4(exported, [initial["x"], initial["v"]], dt=0.01, steps=100)
    native_traj = world.simulate(dict(initial), start=0.0, end=1.0, step=0.01)
    deviation = max(
        abs(native_traj.values[name][j] - mine[i][j])
        for i, name in enumerate(["x", "v"])
        for j in range(len(mine[i]))
    )
    assert deviation < 1e-9


@native
def test_numeric_derivatives_evaluate_sin_div_and_power():
    world = _trig_world()
    d = lawsynth.numeric_derivatives(world, {"x": 0.5, "v": 2.0})
    assert d["x"] == pytest.approx(1.0)  # v / 2
    assert d["v"] == pytest.approx(-3.0 * math.sin(0.5) + 0.5 ** 2)


@native
def test_named_parameters_become_leading_named_constants():
    spec = WorldSpec.create(
        states=["x", "v"],
        parameters={"k": 3.0, "c": 0.5},
        equations={"x": "v", "v": "((-k*x)-(c*v))"},
    )
    program = export._compile(spec.realize())
    assert program.constant_names[:2] == ("k", "c")
    assert program.constants[:2] == (3.0, 0.5)


@native
def test_numeric_derivatives_accept_positional_state_vector():
    world = _trig_world()
    program = export._compile(world)
    mapping = lawsynth.numeric_derivatives(world, {"x": 0.5, "v": 2.0})
    positional = lawsynth.numeric_derivatives(
        world, [ {"x": 0.5, "v": 2.0}[name] for name in program.states ]
    )
    assert positional == mapping


# --------------------------------------------------------------------------- #
# SymPy export                                                                 #
# --------------------------------------------------------------------------- #


@native
def test_to_sympy_maps_polynomial_law():
    sympy = pytest.importorskip("sympy")
    world = _discover_oscillator().world
    laws = lawsynth.to_sympy(world)
    assert set(laws) == {"x", "v"}
    x, v = sympy.Symbol("x", real=True), sympy.Symbol("v", real=True)
    # d(v)/dt is a linear combination of x and v; check coefficients numerically.
    dv = laws["v"]
    coeff_x = float(dv.coeff(x))
    coeff_v = float(dv.coeff(v))
    assert -4.05 < coeff_x < -3.95
    assert -0.35 < coeff_v < -0.25
    # SymPy ecosystem handles the result: latex + lambdify round-trip.
    assert isinstance(sympy.latex(dv), str)
    fn = sympy.lambdify((x, v), dv)
    assert fn(1.0, 0.0) == pytest.approx(float(dv.subs({x: 1.0, v: 0.0})))


@native
def test_to_sympy_maps_sin_div_power():
    sympy = pytest.importorskip("sympy")
    laws = lawsynth.to_sympy(_trig_world())
    x = sympy.Symbol("x", real=True)
    v = sympy.Symbol("v", real=True)
    assert sympy.simplify(laws["x"] - v / 2) == 0
    assert sympy.simplify(laws["v"] - (-3 * sympy.sin(x) + x ** 2)) == 0


# --------------------------------------------------------------------------- #
# PyTorch export                                                               #
# --------------------------------------------------------------------------- #


@native
def test_to_torch_builds_differentiable_module_with_trainable_constants():
    torch = pytest.importorskip("torch")
    world = _discover_oscillator().world
    initial = {"x": 1.0, "v": 0.0}
    module = lawsynth.to_torch(world, initial=initial)

    # Constants are a trainable parameter.
    assert isinstance(module.constants, torch.nn.Parameter)
    assert module.constants.requires_grad
    # Initial conditions are exposed as an optional trainable parameter.
    assert isinstance(module.initial, torch.nn.Parameter)
    assert module.initial.requires_grad

    state = torch.tensor([initial[name] for name in module.state_names], dtype=torch.float64)
    dstate = module.derivatives(0.0, state)
    reference = lawsynth.numeric_derivatives(world, initial)
    for i, name in enumerate(module.state_names):
        assert float(dstate[i]) == pytest.approx(reference[name], abs=1e-10)

    # A gradient flows back into the constants (the Neural-ODE fine-tuning path).
    loss = (module.derivatives(0.0, state) ** 2).sum()
    loss.backward()
    assert module.constants.grad is not None
    assert module.constants.grad.shape == module.constants.shape


@native
def test_to_torch_initial_can_be_a_non_trainable_buffer():
    torch = pytest.importorskip("torch")
    world = _discover_oscillator().world
    module = lawsynth.to_torch(world, initial={"x": 1.0, "v": 0.0}, trainable_initial=False)
    assert not isinstance(module.initial, torch.nn.Parameter)
    assert isinstance(module.initial, torch.Tensor)


# --------------------------------------------------------------------------- #
# JAX export                                                                   #
# --------------------------------------------------------------------------- #


@native
def test_to_jax_builds_differentiable_function_with_trainable_pytree():
    jax = pytest.importorskip("jax")
    jax.config.update("jax_enable_x64", True)
    jnp = pytest.importorskip("jax.numpy")
    world = _discover_oscillator().world
    initial = {"x": 1.0, "v": 0.0}
    dyn = lawsynth.to_jax(world, initial=initial)

    assert "constants" in dyn.params
    assert "initial" in dyn.params  # trainable initial by default
    state = jnp.asarray([initial[name] for name in dyn.state_names], dtype=jnp.float64)
    dstate = dyn.derivatives(dyn.params, 0.0, state)
    reference = lawsynth.numeric_derivatives(world, initial)
    for i, name in enumerate(dyn.state_names):
        assert float(dstate[i]) == pytest.approx(reference[name], abs=1e-8)

    def loss(params):
        return jnp.sum(dyn.derivatives(params, 0.0, state) ** 2)

    grads = jax.grad(loss)(dyn.params)
    assert grads["constants"].shape == dyn.params["constants"].shape
