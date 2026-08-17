"""Honest, equivalence-checked symbolic simplification of executable worlds.

:func:`simplify_world` rewrites every law of a world into a canonical, minimal
form via :mod:`lawsynth.algebra`, rebuilds a *new* native world from the
simplified laws, and returns a :class:`SimplifiedWorld` report carrying the
before/after expression and AST node count for each law. The result is
guaranteed mathematically equivalent: :meth:`SimplifiedWorld.verify` simulates
the original and simplified worlds on a shared grid and returns the maximum
trajectory deviation (expected at the level of floating-point round-off).

The module deliberately avoids the name ``simplify`` so it never shadows the
public ``lawsynth.simplify`` callable.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Mapping

from .algebra import node_count, simplify_expression
from .errors import NativeError, ValidationError
from .worldspec import WorldSpec

__all__ = ["LawSimplification", "SimplifiedWorld", "simplify_world"]


@dataclass(frozen=True, slots=True)
class LawSimplification:
    """Before/after record for a single law."""

    target: str
    before: str
    after: str
    before_nodes: int
    after_nodes: int

    @property
    def reduced(self) -> int:
        return self.before_nodes - self.after_nodes

    @property
    def changed(self) -> bool:
        return self.before != self.after


class SimplifiedWorld:
    """The result of simplifying a world: the new world plus a per-law report."""

    __slots__ = ("_original", "_world", "_laws")

    def __init__(self, original: object, world: object, laws: tuple[LawSimplification, ...]) -> None:
        self._original = original
        self._world = world
        self._laws = laws

    @property
    def original(self) -> object:
        """The original (unsimplified) native world."""
        return self._original

    @property
    def world(self) -> object:
        """The new, simplified, equivalent native world."""
        return self._world

    @property
    def laws(self) -> tuple[LawSimplification, ...]:
        return self._laws

    @property
    def before_nodes(self) -> int:
        return sum(law.before_nodes for law in self._laws)

    @property
    def after_nodes(self) -> int:
        return sum(law.after_nodes for law in self._laws)

    @property
    def reduced(self) -> int:
        """Total AST-node reduction across all laws (>= 0)."""
        return self.before_nodes - self.after_nodes

    @property
    def reduction_ratio(self) -> float:
        return self.reduced / self.before_nodes if self.before_nodes else 0.0

    def verify(
        self,
        initial: Mapping[str, float],
        *,
        start: float = 0.0,
        end: float = 1.0,
        step: float = 0.01,
    ) -> float:
        """Simulate original vs. simplified from ``initial``; return max deviation.

        A value at the level of machine epsilon confirms the simplification is
        mathematically faithful rather than merely plausible.
        """
        try:
            before = self._original.simulate(dict(initial), start=start, end=end, step=step)
            after = self._world.simulate(dict(initial), start=start, end=end, step=step)
        except Exception as error:  # native raises plain exceptions
            raise NativeError(f"equivalence simulation failed: {error}") from error
        deviation = 0.0
        for name, column in before.values.items():
            other = after.values.get(name, ())
            for a, b in zip(column, other):
                deviation = max(deviation, abs(a - b))
        return deviation

    # -- rendering ---------------------------------------------------------- #

    def to_text(self) -> str:
        lines = [
            "Simplified world",
            f"  total complexity: {self.before_nodes} -> {self.after_nodes} nodes "
            f"({self.reduced:+d}, {self.reduction_ratio:.0%} smaller)",
            "",
        ]
        for law in self._laws:
            marker = "" if law.changed else "  (already minimal)"
            lines.append(f"  d{law.target}/dt: {law.before_nodes} -> {law.after_nodes} nodes{marker}")
            lines.append(f"    before: {law.before}")
            lines.append(f"    after:  {law.after}")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return f"SimplifiedWorld(laws={len(self._laws)}, reduced={self.reduced})"


def simplify_world(world: object) -> SimplifiedWorld:
    """Simplify every law of ``world`` and return an equivalence-checked report."""
    spec = WorldSpec.from_world(world)
    if not spec.equations:
        raise ValidationError("world has no laws to simplify")
    laws: list[LawSimplification] = []
    simplified_equations: dict[str, str] = {}
    for target, expression in spec.equations:
        after = simplify_expression(expression)
        simplified_equations[target] = after
        laws.append(
            LawSimplification(
                target=target,
                before=expression,
                after=after,
                before_nodes=node_count(expression),
                after_nodes=node_count(after),
            )
        )
    simplified_world = spec.with_equations(simplified_equations).realize()
    return SimplifiedWorld(world, simplified_world, tuple(laws))


def _install() -> None:
    """Attach ``World.simplify`` to the native world type (best-effort, lazy)."""
    try:
        from ._native import World
    except Exception:  # pragma: no cover - native optional at import time
        return
    if not hasattr(World, "simplify"):
        World.simplify = lambda self: simplify_world(self)  # type: ignore[attr-defined]


_install()
