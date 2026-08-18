"""Differentiable export of discovered worlds to SymPy, PyTorch and JAX.

A discovered ``.lsworld`` is a set of laws ``d(state)/dt = expression`` whose
coefficients are ordinary arithmetic strings (``((-3.99*x)+(-0.29*v))``). Symbolic
regression tools in the wider ecosystem expect two handles LawSynth historically
lacked:

* a **SymPy** expression per law — the gateway to ``simplify``, ``latex`` and
  ``lambdify`` (the whole computer-algebra ecosystem); and
* a **differentiable** ``derivatives(t, state)`` function for PyTorch or JAX whose
  model constants are *trainable parameters*, so a discovered world can be dropped
  in as a **Neural-ODE** layer and fine-tuned end to end.

The module imports cleanly with *none* of ``sympy``/``torch``/``jax`` installed:
the heavy dependency is required lazily, and its absence raises a clear, typed
:class:`MissingDependencyError` (a subclass of both :class:`LawSynthError` and the
built-in :class:`ImportError`, so it plays nicely with ``pytest.importorskip`` and
``except ImportError``). The core stays dependency-free and offline; you only pay
for a backend when you ask for it.

Faithfulness is verifiable: the exported dynamics are built from the very
expression strings the native engine integrates, so integrating them with the
same fixed-step RK4 reproduces ``world.simulate(...)`` to floating-point
round-off. :func:`numeric_derivatives` gives a dependency-free reference for that
check.
"""

from __future__ import annotations

import ast
import importlib
from dataclasses import dataclass
from typing import Callable, Mapping, Sequence

from .errors import LawSynthError
from .worldspec import WorldSpec

__all__ = [
    "ExportError",
    "MissingDependencyError",
    "to_sympy",
    "to_torch",
    "to_jax",
    "numeric_derivatives",
]


# --------------------------------------------------------------------------- #
# Errors                                                                       #
# --------------------------------------------------------------------------- #


class ExportError(LawSynthError):
    """Raised when a law cannot be exported (an unsupported expression form)."""


class MissingDependencyError(ExportError, ImportError):
    """Raised when an optional export backend (sympy/torch/jax) is not installed.

    Subclasses :class:`ImportError` as well as :class:`LawSynthError` so callers
    can catch it either way and ``pytest.importorskip`` semantics feel natural.
    """


def _require(module: str, feature: str):
    """Import an optional backend or raise a clear, typed error naming the fix."""
    try:
        return importlib.import_module(module)
    except ImportError as error:  # pragma: no cover - exercised only when absent
        raise MissingDependencyError(
            f"{feature} requires the optional dependency {module!r}, which is not "
            f"installed. Install it (e.g. `pip install {module}`) to enable this "
            f"export. LawSynth's core is dependency-free; {module} is only needed "
            f"for this feature."
        ) from error


# --------------------------------------------------------------------------- #
# Intermediate representation                                                  #
#                                                                              #
# Each law is parsed once into a small immutable node tree. Numeric literals    #
# and named parameters are hoisted into a single flat *constant vector* so that #
# a differentiable backend can expose them all as one trainable tensor/pytree.  #
# States resolve against the integrator's state vector; controls against the    #
# externally supplied inputs.                                                   #
# --------------------------------------------------------------------------- #

_FUNCTIONS = ("sin", "cos", "exp", "log")


@dataclass(frozen=True, slots=True)
class _ConstRef:
    index: int


@dataclass(frozen=True, slots=True)
class _StateRef:
    name: str


@dataclass(frozen=True, slots=True)
class _ControlRef:
    name: str


@dataclass(frozen=True, slots=True)
class _Bin:
    op: str  # one of + - * / **
    left: object
    right: object


@dataclass(frozen=True, slots=True)
class _Neg:
    operand: object


@dataclass(frozen=True, slots=True)
class _Call:
    func: str  # one of _FUNCTIONS
    arg: object


@dataclass(frozen=True, slots=True)
class _Program:
    """The full differentiable dynamics of a world, backend-agnostic."""

    states: tuple[str, ...]
    controls: tuple[str, ...]
    equations: tuple[tuple[str, object], ...]
    constants: tuple[float, ...]
    constant_names: tuple[str, ...]


class _Builder:
    """Parses law expressions into the IR, hoisting constants as it goes."""

    def __init__(self, spec: WorldSpec) -> None:
        self._states = frozenset(spec.states)
        self._values: list[float] = []
        self._names: list[str] = []
        self._param_index: dict[str, int] = {}
        # Named parameters occupy leading, stable constant slots.
        for name, value in spec.parameters:
            self._param_index[name] = len(self._values)
            self._values.append(float(value))
            self._names.append(name)
        # Controls are discovered from the expressions themselves: any identifier
        # that is neither a state nor a parameter is an external input. We compute
        # this here rather than trusting ``spec.controls`` because the SDK's
        # best-effort spec recovery can misclassify function names (``sin``) as
        # controls — but function names are handled as calls and never reach the
        # Name branch below, so this set stays clean.
        self._controls_seen: list[str] = []

    def _new_literal(self, value: float) -> _ConstRef:
        index = len(self._values)
        self._values.append(float(value))
        self._names.append(f"c{index}")
        return _ConstRef(index)

    def build(self, expression: str, target: str) -> object:
        try:
            tree = ast.parse(expression, mode="eval").body
        except SyntaxError as error:
            raise ExportError(f"law for {target!r} is not a parseable expression: {error}") from error
        return self._node(tree, target)

    def _node(self, node: ast.AST, target: str) -> object:
        if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)) and not isinstance(node.value, bool):
            return self._new_literal(float(node.value))
        if isinstance(node, ast.Name):
            if node.id in self._states:
                return _StateRef(node.id)
            if node.id in self._param_index:
                return _ConstRef(self._param_index[node.id])
            if node.id not in self._controls_seen:
                self._controls_seen.append(node.id)
            return _ControlRef(node.id)
        if isinstance(node, ast.UnaryOp):
            if isinstance(node.op, ast.USub):
                return _Neg(self._node(node.operand, target))
            if isinstance(node.op, ast.UAdd):
                return self._node(node.operand, target)
            raise ExportError(f"law for {target!r} uses an unsupported unary operator")
        if isinstance(node, ast.BinOp):
            op = _BINARY_OPS.get(type(node.op))
            if op is None:
                raise ExportError(
                    f"law for {target!r} uses an unsupported binary operator "
                    f"{type(node.op).__name__}"
                )
            return _Bin(op, self._node(node.left, target), self._node(node.right, target))
        if isinstance(node, ast.Call):
            if node.keywords or len(node.args) != 1 or not isinstance(node.func, ast.Name):
                raise ExportError(f"law for {target!r} uses an unsupported call form")
            if node.func.id not in _FUNCTIONS:
                raise ExportError(
                    f"law for {target!r} calls unsupported function {node.func.id!r}; "
                    f"supported: {', '.join(_FUNCTIONS)}"
                )
            return _Call(node.func.id, self._node(node.args[0], target))
        raise ExportError(
            f"law for {target!r} contains an unsupported construct: {type(node).__name__}"
        )

    def program(self, spec: WorldSpec) -> _Program:
        equations = tuple((target, self.build(expr, target)) for target, expr in spec.equations)
        return _Program(
            states=spec.states,
            controls=tuple(self._controls_seen),
            equations=equations,
            constants=tuple(self._values),
            constant_names=tuple(self._names),
        )


# The engine's printer emits ``^`` for exponentiation, which Python's parser reads
# as bitwise-xor (``BitXor``); the engine never means xor, so we map it to power.
_BINARY_OPS = {
    ast.Add: "+",
    ast.Sub: "-",
    ast.Mult: "*",
    ast.Div: "/",
    ast.Pow: "**",
    ast.BitXor: "**",
}


def _compile(world: object) -> _Program:
    spec = WorldSpec.from_world(world)
    return _Builder(spec).program(spec)


# --------------------------------------------------------------------------- #
# Backend-agnostic evaluation                                                  #
#                                                                              #
# Arithmetic (+ - * / ** and unary -) is expressed with Python operators, which #
# torch tensors, jax arrays and plain floats all overload identically. Only the  #
# elementwise transcendental functions differ per backend, so a backend is just  #
# a table of {sin, cos, exp, log}.                                              #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class _Backend:
    sin: Callable[[object], object]
    cos: Callable[[object], object]
    exp: Callable[[object], object]
    log: Callable[[object], object]

    def func(self, name: str) -> Callable[[object], object]:
        return getattr(self, name)


def _evaluate(node: object, backend: _Backend, state_env: Mapping[str, object], control_env: Mapping[str, object], constants) -> object:
    if isinstance(node, _ConstRef):
        return constants[node.index]
    if isinstance(node, _StateRef):
        return state_env[node.name]
    if isinstance(node, _ControlRef):
        return control_env[node.name]
    if isinstance(node, _Neg):
        return -_evaluate(node.operand, backend, state_env, control_env, constants)
    if isinstance(node, _Call):
        return backend.func(node.func)(_evaluate(node.arg, backend, state_env, control_env, constants))
    if isinstance(node, _Bin):
        left = _evaluate(node.left, backend, state_env, control_env, constants)
        right = _evaluate(node.right, backend, state_env, control_env, constants)
        if node.op == "+":
            return left + right
        if node.op == "-":
            return left - right
        if node.op == "*":
            return left * right
        if node.op == "/":
            return left / right
        return left ** right
    raise ExportError(f"unexpected IR node {node!r}")  # pragma: no cover - defensive


def _state_env(state, names: Sequence[str]) -> dict[str, object]:
    """Resolve a state vector or mapping into a ``name -> value`` environment."""
    if hasattr(state, "keys"):
        missing = [name for name in names if name not in state]
        if missing:
            raise ExportError(f"missing initial values for states {missing}")
        return {name: state[name] for name in names}
    values = list(state) if not hasattr(state, "shape") else state
    length = len(values) if not hasattr(values, "shape") else int(values.shape[0])
    if length != len(names):
        raise ExportError(f"state vector has length {length}, expected {len(names)} ({list(names)})")
    return {name: values[index] for index, name in enumerate(names)}


def _control_env(inputs, names: Sequence[str]) -> dict[str, object]:
    if not names:
        return {}
    if inputs is None:
        raise ExportError(
            f"this world has control inputs {list(names)}; pass inputs=... to derivatives()"
        )
    if hasattr(inputs, "keys"):
        missing = [name for name in names if name not in inputs]
        if missing:
            raise ExportError(f"missing control inputs {missing}")
        return {name: inputs[name] for name in names}
    values = list(inputs)
    if len(values) != len(names):
        raise ExportError(f"inputs vector has length {len(values)}, expected {len(names)}")
    return {name: values[index] for index, name in enumerate(names)}


# --------------------------------------------------------------------------- #
# Dependency-free reference evaluator                                          #
# --------------------------------------------------------------------------- #


def numeric_derivatives(world: object, state, *, t: float = 0.0, inputs=None) -> dict[str, float]:
    """Evaluate ``d(state)/dt`` at a point using only the standard library.

    ``state`` is a mapping ``{name: value}`` or a positional sequence ordered like
    the world's states. Returns ``{state_name: derivative}``. This mirrors exactly
    what the native engine integrates, so it is the reference the torch/jax
    exports are checked against — and it needs no optional dependencies.
    """
    import math

    program = _compile(world)
    backend = _Backend(math.sin, math.cos, math.exp, math.log)
    state_env = {name: float(value) for name, value in _state_env(state, program.states).items()}
    control_env = _control_env(inputs, program.controls)
    constants = program.constants
    return {
        target: float(_evaluate(node, backend, state_env, control_env, constants))
        for target, node in program.equations
    }


# --------------------------------------------------------------------------- #
# SymPy export                                                                 #
# --------------------------------------------------------------------------- #


def to_sympy(world: object) -> dict[str, object]:
    """Convert each law of ``world`` to a SymPy expression.

    Returns ``{state: expr}`` where ``expr`` is the right-hand side of
    ``d(state)/dt`` as a :class:`sympy.Expr`. States, controls and named
    parameters become real :class:`sympy.Symbol` s; numeric coefficients become
    exact SymPy numbers. The result unlocks the whole SymPy ecosystem::

        laws = lawsynth.to_sympy(world)
        sympy.simplify(laws["v"])
        sympy.latex(laws["v"])
        sympy.lambdify(symbols, laws["v"])

    Raises :class:`MissingDependencyError` if SymPy is not installed.
    """
    sympy = _require("sympy", "World.to_sympy()")
    spec = WorldSpec.from_world(world)
    symbols = {
        name: sympy.Symbol(name, real=True)
        for name in (*spec.states, *spec.controls, *(k for k, _ in spec.parameters))
    }
    functions = {"sin": sympy.sin, "cos": sympy.cos, "exp": sympy.exp, "log": sympy.log}

    def convert(node: ast.AST, target: str):
        if isinstance(node, ast.Constant) and isinstance(node.value, (int, float)) and not isinstance(node.value, bool):
            value = node.value
            if isinstance(value, int):
                return sympy.Integer(value)
            return sympy.Float(value)
        if isinstance(node, ast.Name):
            symbol = symbols.get(node.id)
            if symbol is None:
                # A free identifier (e.g. a genuine control input) becomes a symbol.
                symbol = sympy.Symbol(node.id, real=True)
                symbols[node.id] = symbol
            return symbol
        if isinstance(node, ast.UnaryOp):
            if isinstance(node.op, ast.USub):
                return -convert(node.operand, target)
            if isinstance(node.op, ast.UAdd):
                return convert(node.operand, target)
            raise ExportError(f"law for {target!r} uses an unsupported unary operator")
        if isinstance(node, ast.BinOp):
            left = convert(node.left, target)
            right = convert(node.right, target)
            op = type(node.op)
            if op is ast.Add:
                return left + right
            if op is ast.Sub:
                return left - right
            if op is ast.Mult:
                return left * right
            if op is ast.Div:
                return left / right
            if op in (ast.Pow, ast.BitXor):
                return left ** right
            raise ExportError(f"law for {target!r} uses an unsupported binary operator {op.__name__}")
        if isinstance(node, ast.Call):
            if node.keywords or len(node.args) != 1 or not isinstance(node.func, ast.Name):
                raise ExportError(f"law for {target!r} uses an unsupported call form")
            handler = functions.get(node.func.id)
            if handler is None:
                raise ExportError(f"law for {target!r} calls unsupported function {node.func.id!r}")
            return handler(convert(node.args[0], target))
        raise ExportError(f"law for {target!r} contains an unsupported construct: {type(node).__name__}")

    result: dict[str, object] = {}
    for target, expression in spec.equations:
        try:
            tree = ast.parse(expression, mode="eval").body
        except SyntaxError as error:
            raise ExportError(f"law for {target!r} is not a parseable expression: {error}") from error
        result[target] = convert(tree, target)
    return result


# --------------------------------------------------------------------------- #
# PyTorch export                                                               #
# --------------------------------------------------------------------------- #


def to_torch(world: object, *, initial=None, trainable_initial: bool = True, dtype=None):
    """Build a differentiable :class:`torch.nn.Module` for the world's dynamics.

    The returned module exposes ``derivatives(t, state)`` (also ``forward``)
    returning ``dstate`` as a 1-D tensor ordered like the world's states. Every
    model constant is a single trainable ``constants`` :class:`torch.nn.Parameter`
    (``requires_grad=True``), so the world can be embedded as a Neural-ODE layer
    and fine-tuned by gradient descent. When ``initial`` is provided it is exposed
    as ``initial`` — an optional trainable parameter (``trainable_initial``) or a
    buffer otherwise.

    ``state`` may be a 1-D tensor or a ``{name: value}`` mapping. Raises
    :class:`MissingDependencyError` if PyTorch is not installed.
    """
    torch = _require("torch", "World.to_torch()")
    program = _compile(world)
    tensor_dtype = dtype if dtype is not None else torch.float64
    backend = _Backend(torch.sin, torch.cos, torch.exp, torch.log)

    class TorchDynamics(torch.nn.Module):
        """Differentiable RHS of a discovered world; constants are trainable."""

        def __init__(self) -> None:
            super().__init__()
            self.state_names = program.states
            self.control_names = program.controls
            self.constant_names = program.constant_names
            self._program = program
            self._backend = backend
            self.constants = torch.nn.Parameter(
                torch.tensor(list(program.constants), dtype=tensor_dtype)
            )
            self.initial = None
            if initial is not None:
                env = _state_env(initial, program.states)
                init_tensor = torch.tensor(
                    [float(env[name]) for name in program.states], dtype=tensor_dtype
                )
                if trainable_initial:
                    self.initial = torch.nn.Parameter(init_tensor)
                else:
                    self.register_buffer("initial", init_tensor)

        def derivatives(self, t, state, inputs=None):
            state_env = _state_env(state, self._program.states)
            control_env = _control_env(inputs, self._program.controls)
            outputs = [
                _evaluate(node, self._backend, state_env, control_env, self.constants)
                for _, node in self._program.equations
            ]
            return torch.stack(outputs)

        def forward(self, t, state, inputs=None):
            return self.derivatives(t, state, inputs)

    return TorchDynamics()


# --------------------------------------------------------------------------- #
# JAX export                                                                   #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class JaxDynamics:
    """A differentiable JAX dynamics function with a trainable parameter pytree.

    ``params`` is a dict pytree holding ``constants`` (and ``initial`` when
    requested); it is what you differentiate through with ``jax.grad``.
    ``derivatives(params, t, state)`` is a pure function returning ``dstate``.
    """

    state_names: tuple[str, ...]
    control_names: tuple[str, ...]
    constant_names: tuple[str, ...]
    params: dict
    derivatives: Callable
    initial: object = None


def to_jax(world: object, *, initial=None, trainable_initial: bool = True, dtype=None) -> JaxDynamics:
    """Build a differentiable JAX dynamics function for the world.

    Returns a :class:`JaxDynamics` whose ``params`` pytree carries the model
    constants (and, when ``initial`` is given and ``trainable_initial``, the
    initial state) as trainable arrays, and whose pure ``derivatives(params, t,
    state)`` returns ``dstate``. Suitable as a Neural-ODE vector field for
    ``jax.grad`` / ``diffrax``.

    For bit-faithful agreement with the float64 engine, enable x64 first::

        import jax; jax.config.update("jax_enable_x64", True)

    Raises :class:`MissingDependencyError` if JAX is not installed.
    """
    _require("jax", "World.to_jax()")
    jnp = _require("jax.numpy", "World.to_jax()")
    program = _compile(world)
    array_dtype = dtype if dtype is not None else jnp.float64
    backend = _Backend(jnp.sin, jnp.cos, jnp.exp, jnp.log)

    params: dict[str, object] = {"constants": jnp.asarray(program.constants, dtype=array_dtype)}
    initial_array = None
    if initial is not None:
        env = _state_env(initial, program.states)
        initial_array = jnp.asarray([float(env[name]) for name in program.states], dtype=array_dtype)
        if trainable_initial:
            params["initial"] = initial_array

    def derivatives(params, t, state, inputs=None):
        state_env = _state_env(state, program.states)
        control_env = _control_env(inputs, program.controls)
        constants = params["constants"]
        outputs = [
            _evaluate(node, backend, state_env, control_env, constants)
            for _, node in program.equations
        ]
        return jnp.stack(outputs)

    return JaxDynamics(
        state_names=program.states,
        control_names=program.controls,
        constant_names=program.constant_names,
        params=params,
        derivatives=derivatives,
        initial=initial_array,
    )


# --------------------------------------------------------------------------- #
# Attach convenience methods to the native World (best-effort, lazy)           #
# --------------------------------------------------------------------------- #


def _install() -> None:
    try:
        from ._native import World
    except Exception:  # pragma: no cover - native optional at import time
        return
    if not hasattr(World, "to_sympy"):
        World.to_sympy = lambda self: to_sympy(self)  # type: ignore[attr-defined]
    if not hasattr(World, "to_torch"):
        World.to_torch = lambda self, **kwargs: to_torch(self, **kwargs)  # type: ignore[attr-defined]
    if not hasattr(World, "to_jax"):
        World.to_jax = lambda self, **kwargs: to_jax(self, **kwargs)  # type: ignore[attr-defined]


_install()
