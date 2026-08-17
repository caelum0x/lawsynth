"""Spec contract model for LawSynth schema generation.

This module holds a small, dependency-free intermediate representation (IR) of
the LawSynth specification contracts (World IR bundle payload, variables,
parameters, laws, and observation datasets described under ``specs/world-ir``).

The IR is the single source of truth consumed by the JSON Schema, TypeScript,
and Python emitters.  Keeping the contracts in one typed registry means the
three outputs cannot drift apart: they are all projections of the same data.
"""

from __future__ import annotations

from dataclasses import dataclass, field

# The primitive JSON types the IR understands.  ``object`` fields must use a
# ``ref`` to another contract so the emitters can generate a named type.
PRIMITIVES = frozenset({"string", "number", "integer", "boolean"})


@dataclass(frozen=True)
class Field:
    """A single field of a contract.

    Exactly one shape is described per field: a primitive ``type``, a ``ref`` to
    another contract, or an ``array`` whose element is described by ``items``.
    """

    name: str
    type: str = "string"
    description: str = ""
    optional: bool = False
    enum: tuple[str, ...] | None = None
    ref: str | None = None
    items: "Field | None" = None
    pattern: str | None = None
    minimum: float | None = None

    def __post_init__(self) -> None:
        if self.type == "array" and self.items is None:
            raise ValueError(f"array field {self.name!r} requires 'items'")
        if self.type == "object" and self.ref is None:
            raise ValueError(f"object field {self.name!r} requires 'ref'")
        if self.type not in PRIMITIVES and self.type not in {"array", "object"}:
            raise ValueError(f"field {self.name!r} has unknown type {self.type!r}")


@dataclass(frozen=True)
class Contract:
    """A named object contract with an ordered set of fields."""

    name: str
    title: str
    description: str
    fields: tuple[Field, ...] = field(default_factory=tuple)

    def required(self) -> list[str]:
        return [item.name for item in self.fields if not item.optional]


# ---------------------------------------------------------------------------
# The LawSynth specification contracts.
#
# These mirror the construction invariants in specs/world-ir/types.md: a World
# is a set of variables, constant scalar parameters, and one scalar law per
# state variable, serialized as the .lsworld bundle payload.
# ---------------------------------------------------------------------------

VARIABLE_ROLES = ("State", "Control", "Exogenous", "Observed", "Latent", "Derived")
WORLD_KINDS = ("continuous", "discrete")

_IDENTIFIER_PATTERN = r"^[A-Za-z_][A-Za-z0-9_]*$"


def _contracts() -> dict[str, Contract]:
    variable = Contract(
        name="Variable",
        title="Variable",
        description="A declared quantity with a role and optional physical unit.",
        fields=(
            Field("id", "string", "Unique lexical identifier.", pattern=_IDENTIFIER_PATTERN),
            Field("role", "string", "Role in the world.", enum=VARIABLE_ROLES),
            Field("unit", "string", "SI-derived unit expression.", optional=True),
        ),
    )
    parameter = Contract(
        name="Parameter",
        title="Parameter",
        description="A finite scalar constant held fixed for a run.",
        fields=(
            Field("id", "string", "Unique lexical identifier.", pattern=_IDENTIFIER_PATTERN),
            Field("value", "number", "Finite f64 value."),
            Field("unit", "string", "SI-derived unit expression.", optional=True),
        ),
    )
    law = Contract(
        name="Law",
        title="Law",
        description=(
            "A scalar law for one state variable. For a continuous world it is "
            "d target / dt = expression; for a discrete world target[t+1] = expression[t]."
        ),
        fields=(
            Field("target", "string", "Identifier of the governed state variable.",
                  pattern=_IDENTIFIER_PATTERN),
            Field("expression", "string", "Canonical expression-language source."),
        ),
    )
    bundle = Contract(
        name="WorldBundle",
        title="World IR bundle payload",
        description="The validated, deterministic payload serialized inside a .lsworld bundle.",
        fields=(
            Field("spec_version", "string", "World IR specification version, e.g. '0.1'."),
            Field("kind", "string", "Time semantics of the world.", enum=WORLD_KINDS),
            Field("variables", "array", "Declared variables, ordered lexically by id.",
                  items=Field("variables_item", "object", ref="Variable")),
            Field("parameters", "array", "Constant scalar parameters, ordered lexically by id.",
                  items=Field("parameters_item", "object", ref="Parameter")),
            Field("laws", "array", "One law per state variable, ordered by target id.",
                  items=Field("laws_item", "object", ref="Law")),
        ),
    )
    observation = Contract(
        name="ObservationDataset",
        title="Observation dataset",
        description="A regularly sampled multivariate time series fed to discovery.",
        fields=(
            Field("time_column", "string", "Name of the monotonically increasing time column."),
            Field("columns", "array", "Observed channel names in file order.",
                  items=Field("columns_item", "string")),
            Field("sample_count", "integer", "Number of rows.", minimum=3),
            Field("step", "number", "Constant sampling step; finite and positive.", minimum=0.0),
        ),
    )
    return {c.name: c for c in (variable, parameter, law, bundle, observation)}


_REGISTRY = _contracts()


def contracts() -> dict[str, Contract]:
    """Return the immutable registry of specification contracts by name."""
    return dict(_REGISTRY)


def get_contract(name: str) -> Contract:
    try:
        return _REGISTRY[name]
    except KeyError as error:
        known = ", ".join(sorted(_REGISTRY))
        raise KeyError(f"unknown contract {name!r}; known contracts: {known}") from error
