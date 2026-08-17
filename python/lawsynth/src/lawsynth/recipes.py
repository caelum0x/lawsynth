"""Curated, per-domain discovery presets — the approachable front door to tuning.

``discover_world`` exposes a dozen orthogonal knobs (feature-library degree,
sparsity threshold, solver, derivative estimator, trig/rational toggles, …).
Great defaults exist, but the *right* settings depend on the physics of the
system: an ecological predator–prey model wants quadratic interaction terms, a
mechanical oscillator often needs a cubic library to catch ``x²·v`` damping, and
an epidemic's bilinear ``β·S·I`` transmission hides behind small coefficients
that a coarse threshold prunes away.

A :class:`Recipe` packages a small, documented, *immutable* set of overrides
tuned for one family of systems, together with a human description and the kinds
of systems it suits. Recipes never touch the native engine — importing this
module loads no compiled code — they merely describe configuration.

Usage::

    import lawsynth
    lawsynth.recipes.list()                     # every recipe
    lawsynth.recipes.get("ecology").describe()  # what it does + why
    study.discover(recipe="epidemiology")       # apply it
    study.discover(recipe="mechanics", threshold=0.02)  # override wins

Explicit ``**overrides`` on :meth:`Study.discover` always win over the recipe:
the recipe is a starting point, never a cage.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Mapping

from .config import DiscoveryConfig
from .errors import ValidationError

__all__ = ["Recipe", "list", "get", "names"]


# --------------------------------------------------------------------------- #
# Recipe — a named, immutable bundle of discovery overrides                    #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class Recipe:
    """A named, documented set of discovery-config overrides for one domain.

    ``settings`` holds only the fields that differ from :class:`DiscoveryConfig`
    defaults, keeping each recipe legible: you see exactly what it changes and
    why. :meth:`config` materialises a full validated config; :meth:`merge`
    layers explicit user overrides on top (explicit always wins).
    """

    name: str
    summary: str
    systems: tuple[str, ...]
    settings: Mapping[str, object] = field(default_factory=dict)
    aliases: tuple[str, ...] = ()

    def __post_init__(self) -> None:
        # Validate eagerly so a broken recipe fails at definition time, not deep
        # inside a discovery run. Building the config also checks field names.
        unknown = set(self.settings) - _CONFIG_FIELDS
        if unknown:
            raise ValidationError(
                f"recipe {self.name!r} sets unknown discovery options: {sorted(unknown)}"
            )
        self.config()  # raises ValidationError on bad values

    def config(self) -> DiscoveryConfig:
        """The recipe's settings as a fully validated :class:`DiscoveryConfig`."""
        return DiscoveryConfig(**dict(self.settings))

    def merge(self, overrides: Mapping[str, object]) -> DiscoveryConfig:
        """Recipe settings with ``overrides`` layered on top — overrides win."""
        merged = {**self.settings, **dict(overrides)}
        unknown = set(merged) - _CONFIG_FIELDS
        if unknown:
            raise ValidationError(f"unknown discovery options: {sorted(unknown)}")
        return DiscoveryConfig(**merged)

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "aliases": list(self.aliases),
            "summary": self.summary,
            "systems": list(self.systems),
            "settings": dict(self.settings),
        }

    def describe(self) -> str:
        """A human-readable, multi-line description of the recipe."""
        config = self.config()
        alias_note = f"  (aliases: {', '.join(self.aliases)})" if self.aliases else ""
        lines = [
            f"Recipe: {self.name}{alias_note}",
            f"  {self.summary}",
            f"  Suited to: {', '.join(self.systems)}.",
            "  Discovery settings (differences from defaults):",
        ]
        if self.settings:
            for key in sorted(self.settings):
                lines.append(f"    {key} = {getattr(config, key)!r}")
        else:
            lines.append("    (uses library defaults)")
        lines.append("  Explicit overrides passed to discover() always win.")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.describe()

    def _repr_html_(self) -> str:
        rows = "".join(
            f"<tr><td style='padding:2px 10px;font-family:ui-monospace,monospace'>{key}</td>"
            f"<td style='padding:2px 10px'>{getattr(self.config(), key)!r}</td></tr>"
            for key in sorted(self.settings)
        )
        alias_note = (
            f" <span style='color:#53627a'>(aliases: {', '.join(self.aliases)})</span>"
            if self.aliases
            else ""
        )
        return (
            '<section style="font:14px system-ui;border:1px solid #cbd5e1;'
            'border-radius:10px;padding:14px;margin:8px 0">'
            f'<h3 style="margin:0 0 6px;color:#155e75">Recipe — {self.name}{alias_note}</h3>'
            f"<p style='margin:0 0 6px'>{self.summary}</p>"
            f"<p style='margin:0 0 8px;color:#53627a'>Suited to: {', '.join(self.systems)}.</p>"
            f"<table style='border-collapse:collapse'>{rows}</table></section>"
        )


_CONFIG_FIELDS: frozenset[str] = frozenset(DiscoveryConfig.__dataclass_fields__)


# --------------------------------------------------------------------------- #
# The curated catalog                                                          #
# --------------------------------------------------------------------------- #

_RECIPES: tuple[Recipe, ...] = (
    Recipe(
        name="mechanics",
        aliases=("physics",),
        summary=(
            "A cubic polynomial library with tight, clean derivatives — sized "
            "for classical mechanical systems whose damping and restoring "
            "forces are nonlinear (e.g. x²·v, x³ terms)."
        ),
        systems=(
            "harmonic and damped oscillators",
            "pendulums and springs",
            "Van der Pol and Duffing oscillators",
            "coupled mechanical systems",
        ),
        settings={
            "polynomial_degree": 3,
            "threshold": 0.05,
            "solver": "stlsq",
            "derivative_method": "finite",
        },
    ),
    Recipe(
        name="ecology",
        summary=(
            "A quadratic library capturing pairwise species interactions "
            "(the α·x·y coupling at the heart of predator–prey and competition "
            "models) with a moderate sparsity threshold."
        ),
        systems=(
            "Lotka–Volterra predator–prey",
            "competitive Lotka–Volterra",
            "logistic population growth",
            "resource–consumer systems",
        ),
        settings={
            "polynomial_degree": 2,
            "threshold": 0.05,
            "solver": "stlsq",
            "derivative_method": "finite",
        },
    ),
    Recipe(
        name="epidemiology",
        summary=(
            "A quadratic library with a fine sparsity threshold so the small "
            "bilinear transmission term (β·S·I) survives pruning — the defining "
            "structure of compartmental epidemic models."
        ),
        systems=(
            "SIR / SIS / SIRS compartmental models",
            "SEIR with an exposed compartment",
            "logistic epidemic (early outbreak) growth",
        ),
        settings={
            "polynomial_degree": 2,
            "threshold": 0.01,
            "solver": "stlsq",
            "derivative_method": "finite",
        },
    ),
    Recipe(
        name="finance",
        summary=(
            "A low-degree library with a fine threshold and the SR3 solver, "
            "tuned for smooth mean-reverting and drift dynamics where "
            "coefficients are small and structure is nearly linear."
        ),
        systems=(
            "Ornstein–Uhlenbeck mean reversion",
            "deterministic drift of price / rate means",
            "coupled factor / spread models",
        ),
        settings={
            "polynomial_degree": 2,
            "threshold": 0.02,
            "solver": "sr3",
            "derivative_method": "finite",
        },
    ),
    Recipe(
        name="general",
        summary=(
            "The balanced default: a quadratic library with a moderate "
            "threshold. A sensible starting point when the domain is unknown."
        ),
        systems=(
            "unknown or mixed dynamics",
            "quick first-pass discovery",
        ),
        settings={
            "polynomial_degree": 2,
            "threshold": 0.05,
            "solver": "stlsq",
            "derivative_method": "finite",
        },
    ),
)


def _build_index(recipes: tuple[Recipe, ...]) -> dict[str, Recipe]:
    index: dict[str, Recipe] = {}
    for recipe in recipes:
        for key in (recipe.name, *recipe.aliases):
            lowered = key.lower()
            if lowered in index:
                raise ValidationError(f"duplicate recipe key {key!r}")
            index[lowered] = recipe
    return index


_INDEX: dict[str, Recipe] = _build_index(_RECIPES)


# --------------------------------------------------------------------------- #
# Public API                                                                   #
# --------------------------------------------------------------------------- #


def list() -> tuple[Recipe, ...]:  # noqa: A001 - deliberate, mirrors dict.list feel
    """Every curated recipe, in catalog order."""
    return _RECIPES


def names() -> tuple[str, ...]:
    """Every canonical recipe name (aliases excluded)."""
    return tuple(recipe.name for recipe in _RECIPES)


def get(name: str) -> Recipe:
    """Resolve a recipe by name or alias (case-insensitive).

    Raises :class:`ValidationError` naming the available recipes when ``name``
    is unknown, so a typo produces a helpful message rather than a KeyError.
    """
    if not isinstance(name, str) or not name:
        raise ValidationError("recipe name must be a non-empty string")
    recipe = _INDEX.get(name.lower())
    if recipe is None:
        available = ", ".join(sorted(_INDEX))
        raise ValidationError(f"unknown recipe {name!r}; available: {available}")
    return recipe
