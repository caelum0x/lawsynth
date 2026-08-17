"""Composition and targeted editing of executable worlds.

Every operation manipulates the declarative :class:`~lawsynth.worldspec.WorldSpec`
behind a world and rebuilds a fresh, validated native world through the standard
construction path — the inputs are never mutated. This module deliberately
avoids the name ``compose`` for anything but the public callable so it never
shadows ``lawsynth.compose``.

* :func:`compose` unions two worlds into one coupled system, namespacing any
  colliding identifiers.
* :func:`rename`, :func:`set_parameter`, :func:`drop_law` and :func:`scale_law`
  are targeted, immutable edits, each returning a new valid world.
"""

from __future__ import annotations

from .errors import ValidationError
from .worldspec import WorldSpec, rename_identifiers

__all__ = ["compose", "rename", "set_parameter", "drop_law", "scale_law"]


def _prefixed(spec: WorldSpec, prefix: str) -> tuple[WorldSpec, dict[str, str]]:
    """Apply ``prefix`` to every identifier of ``spec`` (identity if empty)."""
    if not prefix:
        return spec, {}
    mapping = {name: f"{prefix}{name}" for name in spec.identifiers}
    return _apply_mapping(spec, mapping), mapping


def _apply_mapping(spec: WorldSpec, mapping: dict[str, str]) -> WorldSpec:
    states = tuple(mapping.get(s, s) for s in spec.states)
    parameters = tuple((mapping.get(k, k), v) for k, v in spec.parameters)
    controls = tuple(mapping.get(c, c) for c in spec.controls)
    equations = tuple(
        (mapping.get(target, target), rename_identifiers(expression, mapping))
        for target, expression in spec.equations
    )
    return WorldSpec(states, parameters, controls, equations)


def compose(world_a: object, world_b: object, *, prefix_a: str = "", prefix_b: str = "") -> object:
    """Combine two worlds into one coupled system.

    The result is the union of both worlds' states, parameters, controls and
    laws. Optional ``prefix_a`` / ``prefix_b`` namespace *all* of a world's
    identifiers. Any identifiers that still collide after prefixing are
    disambiguated by prefixing the second world's copy with ``b_``; an
    unresolvable collision raises :class:`ValidationError`. The returned native
    world is valid and simulates.
    """
    spec_a = WorldSpec.from_world(world_a)
    spec_b = WorldSpec.from_world(world_b)
    spec_a, _ = _prefixed(spec_a, prefix_a)
    spec_b, _ = _prefixed(spec_b, prefix_b)

    collisions = spec_a.identifiers & spec_b.identifiers
    if collisions:
        fallback = {name: f"b_{name}" for name in collisions}
        if set(fallback.values()) & (spec_a.identifiers | spec_b.identifiers):
            raise ValidationError(
                f"cannot auto-namespace colliding identifiers {sorted(collisions)}; "
                "pass prefix_a/prefix_b to disambiguate"
            )
        spec_b = _apply_mapping(spec_b, fallback)

    merged_params: dict[str, float] = {}
    for key, value in (*spec_a.parameters, *spec_b.parameters):
        merged_params[key] = value

    merged = WorldSpec(
        states=spec_a.states + spec_b.states,
        parameters=tuple(merged_params.items()),
        controls=spec_a.controls + spec_b.controls,
        equations=spec_a.equations + spec_b.equations,
    )
    return merged.validate().realize()


def rename(world: object, old: str, new: str) -> object:
    """Return a new world with identifier ``old`` renamed to ``new`` everywhere."""
    return WorldSpec.from_world(world).rename(old, new).realize()


def set_parameter(world: object, name: str, value: float) -> object:
    """Return a new world with parameter ``name`` set to ``value``."""
    return WorldSpec.from_world(world).set_parameter(name, value).realize()


def drop_law(world: object, target: str) -> object:
    """Return a new world with the law for state ``target`` removed.

    If ``target`` is still referenced by other laws it is demoted to a control
    input so the remaining system stays well-formed.
    """
    return WorldSpec.from_world(world).drop_law(target).realize()


def scale_law(world: object, target: str, factor: float) -> object:
    """Return a new world with the law for ``target`` multiplied by ``factor``."""
    return WorldSpec.from_world(world).scale_law(target, factor).realize()


def _install() -> None:
    """Attach the editing methods to the native world type (best-effort, lazy)."""
    try:
        from ._native import World
    except Exception:  # pragma: no cover - native optional at import time
        return
    methods = {
        "rename": lambda self, old, new: rename(self, old, new),
        "set_parameter": lambda self, name, value: set_parameter(self, name, value),
        "drop_law": lambda self, target: drop_law(self, target),
        "scale_law": lambda self, target, factor: scale_law(self, target, factor),
    }
    for name, function in methods.items():
        if not hasattr(World, name):
            setattr(World, name, function)


_install()
