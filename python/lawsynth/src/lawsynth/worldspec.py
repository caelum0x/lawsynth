"""Declarative, immutable structure behind a native executable World.

The native ``World`` is opaque: it exposes ``equations()``, ``simulate()``,
``save()``/``load()`` and nothing else — parameter *values* and the
state/control split are not recoverable from an arbitrary native instance. To
support principled composition and editing we therefore carry a small
declarative :class:`WorldSpec` (states, parameters, controls, laws) alongside
every world the SDK constructs, and reconstruct native worlds from it through
the existing :func:`lawsynth.world.build_world` path.

Worlds the SDK builds are recorded in a process-local registry keyed by object
identity (the world object is held, so the id stays valid). For worlds we did
*not* build — a freshly discovered world, or a raw native construction — we
recover a best-effort spec from ``equations()``: the equation targets are the
states and any remaining free identifier is treated as a control. Discovered
worlds inline all coefficients, so they have no free identifiers and round-trip
exactly.
"""

from __future__ import annotations

import ast
from dataclasses import dataclass

from .equation import Equation
from .errors import ValidationError
from .variable import Variable
from .world import build_world

__all__ = ["WorldSpec", "spec_of", "rename_identifiers", "free_identifiers"]


# Registry: id(world) -> (world, spec). Holding the world keeps the id valid and
# lets edits/composition recover the exact parameter values we constructed with.
_REGISTRY: dict[int, tuple[object, "WorldSpec"]] = {}


def free_identifiers(expression: str, bound: frozenset[str]) -> frozenset[str]:
    """Identifiers referenced in ``expression`` that are not in ``bound``."""
    try:
        tree = ast.parse(expression, mode="eval")
    except SyntaxError:
        return frozenset()
    names = {node.id for node in ast.walk(tree) if isinstance(node, ast.Name)}
    return frozenset(names - bound)


def rename_identifiers(expression: str, mapping: dict[str, str]) -> str:
    """Rewrite every free identifier in ``expression`` according to ``mapping``."""
    if not mapping:
        return expression
    try:
        tree = ast.parse(expression, mode="eval")
    except SyntaxError:
        return expression

    class _Renamer(ast.NodeTransformer):
        def visit_Name(self, node: ast.Name) -> ast.Name:
            if node.id in mapping:
                return ast.copy_location(ast.Name(id=mapping[node.id], ctx=node.ctx), node)
            return node

    return ast.unparse(_Renamer().visit(tree))


@dataclass(frozen=True, slots=True)
class WorldSpec:
    """An immutable, declarative description of a World's structure.

    ``parameters`` and ``equations`` are ordered tuples of pairs so the whole
    spec is hashable and deterministic. Every mutating helper returns a *new*
    spec; nothing is edited in place.
    """

    states: tuple[str, ...]
    parameters: tuple[tuple[str, float], ...]
    controls: tuple[str, ...]
    equations: tuple[tuple[str, str], ...]

    # -- construction ------------------------------------------------------- #

    @classmethod
    def create(
        cls,
        *,
        states=None,
        parameters=None,
        controls=(),
        equations,
    ) -> "WorldSpec":
        """Validate and build a spec from mappings/sequences."""
        equation_items = tuple(equations.items()) if isinstance(equations, dict) else tuple(equations)
        state_names = tuple(states) if states is not None else tuple(target for target, _ in equation_items)
        param_items = tuple(dict(parameters).items()) if parameters else ()
        control_names = tuple(controls)
        spec = cls(state_names, tuple((k, float(v)) for k, v in param_items), control_names, equation_items)
        spec.validate()
        return spec

    @classmethod
    def from_world(cls, world: object) -> "WorldSpec":
        """Recover the spec for ``world`` from the registry or from its equations."""
        recorded = _REGISTRY.get(id(world))
        if recorded is not None and recorded[0] is world:
            return recorded[1]
        equations = tuple(dict(world.equations()).items())
        states = tuple(target for target, _ in equations)
        bound = frozenset(states)
        free: set[str] = set()
        for _, expression in equations:
            free |= free_identifiers(expression, bound)
        # No parameter values are recoverable from a native world, so every free
        # identifier is treated as a control input. Discovered worlds inline all
        # coefficients and hit this path with an empty free set.
        return cls(states, (), tuple(sorted(free)), equations)

    # -- accessors ---------------------------------------------------------- #

    @property
    def parameter_map(self) -> dict[str, float]:
        return dict(self.parameters)

    @property
    def equation_map(self) -> dict[str, str]:
        return dict(self.equations)

    @property
    def identifiers(self) -> frozenset[str]:
        """Every declared name: states, parameters and controls."""
        return frozenset(self.states) | frozenset(k for k, _ in self.parameters) | frozenset(self.controls)

    # -- validation & realisation ------------------------------------------ #

    def validate(self) -> "WorldSpec":
        targets = tuple(target for target, _ in self.equations)
        if set(targets) != set(self.states):
            raise ValidationError("exactly one equation is required for each state")
        if len(targets) != len(set(targets)):
            raise ValidationError("duplicate equation targets")
        declared = self.identifiers
        counts: dict[str, int] = {}
        for name in (*self.states, *(k for k, _ in self.parameters), *self.controls):
            counts[name] = counts.get(name, 0) + 1
        clashes = sorted(name for name, count in counts.items() if count > 1)
        if clashes:
            raise ValidationError(f"identifiers must occupy one namespace; clashes: {clashes}")
        bound = declared
        for target, expression in self.equations:
            unknown = free_identifiers(expression, bound)
            if unknown:
                raise ValidationError(
                    f"law for {target!r} references undeclared identifiers {sorted(unknown)}"
                )
        return self

    def realize(self) -> object:
        """Build a validated native World and register it for later recovery."""
        self.validate()
        states = [Variable(name, "state") for name in self.states]
        controls = [Variable(name, "control") for name in self.controls]
        equations = [Equation(target, expression) for target, expression in self.equations]
        world = build_world(states, self.parameter_map, equations, controls)
        _REGISTRY[id(world)] = (world, self)
        return world

    # -- immutable transforms ---------------------------------------------- #

    def with_equations(self, equations: dict[str, str]) -> "WorldSpec":
        ordered = tuple((target, equations[target]) for target, _ in self.equations)
        return WorldSpec(self.states, self.parameters, self.controls, ordered)

    def rename(self, old: str, new: str) -> "WorldSpec":
        if old not in self.identifiers:
            raise ValidationError(f"{old!r} is not a declared identifier")
        if not new.isidentifier():
            raise ValidationError("new name must be a valid identifier")
        if new in self.identifiers:
            raise ValidationError(f"{new!r} is already declared")
        mapping = {old: new}
        states = tuple(new if s == old else s for s in self.states)
        parameters = tuple((new if k == old else k, v) for k, v in self.parameters)
        controls = tuple(new if c == old else c for c in self.controls)
        equations = tuple(
            ((new if target == old else target), rename_identifiers(expression, mapping))
            for target, expression in self.equations
        )
        return WorldSpec(states, parameters, controls, equations).validate()

    def set_parameter(self, name: str, value: float) -> "WorldSpec":
        if name not in self.parameter_map:
            available = sorted(k for k, _ in self.parameters)
            raise ValidationError(f"{name!r} is not a parameter; declared parameters: {available}")
        parameters = tuple((k, float(value) if k == name else v) for k, v in self.parameters)
        return WorldSpec(self.states, parameters, self.controls, self.equations)

    def drop_law(self, target: str) -> "WorldSpec":
        if target not in self.states:
            raise ValidationError(f"{target!r} is not a state with a law to drop")
        remaining = tuple((t, e) for t, e in self.equations if t != target)
        states = tuple(s for s in self.states if s != target)
        bound = frozenset(states) | frozenset(k for k, _ in self.parameters) | frozenset(self.controls)
        # If the dropped state is still referenced, demote it to a control input
        # so the remaining system stays well-formed.
        still_used = any(target in free_identifiers(expr, bound) for _, expr in remaining)
        controls = self.controls + (target,) if still_used else self.controls
        return WorldSpec(states, self.parameters, controls, remaining).validate()

    def scale_law(self, target: str, factor: float) -> "WorldSpec":
        if target not in self.equation_map:
            raise ValidationError(f"{target!r} has no law to scale")
        # A purely structural edit: wrap the RHS in the factor. Call
        # :func:`lawsynth.simplify` afterwards to fold it into canonical form.
        scaled = f"({float(factor)})*({self.equation_map[target]})"
        equations = tuple((t, scaled if t == target else e) for t, e in self.equations)
        return WorldSpec(self.states, self.parameters, self.controls, equations).validate()


def spec_of(world: object) -> WorldSpec:
    """Public accessor: the declarative spec backing ``world``."""
    return WorldSpec.from_world(world)
