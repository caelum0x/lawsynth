"""Emit frozen Python dataclasses from the contract IR."""

from __future__ import annotations

from schema import Contract, Field

_PRIMITIVE_PY = {
    "string": "str",
    "number": "float",
    "integer": "int",
    "boolean": "bool",
}


def _field_type(item: Field) -> str:
    if item.type == "object":
        return f'"{item.ref}"'
    if item.type == "array":
        assert item.items is not None
        return f"list[{_field_type(item.items)}]"
    if item.enum is not None:
        members = ", ".join(f'"{value}"' for value in item.enum)
        return f"Literal[{members}]"
    return _PRIMITIVE_PY[item.type]


def to_python(contract: Contract) -> str:
    """Return a ``@dataclass(frozen=True)`` definition for one contract.

    Optional fields are emitted last with a ``None`` default so the generated
    source is valid Python regardless of field ordering in the contract.
    """
    required = [item for item in contract.fields if not item.optional]
    optional = [item for item in contract.fields if item.optional]

    lines = [
        "@dataclass(frozen=True)",
        f"class {contract.name}:",
        f'    """{contract.description}"""',
        "",
    ]
    for item in required:
        lines.append(f"    {item.name}: {_field_type(item)}")
    for item in optional:
        lines.append(f"    {item.name}: {_field_type(item)} | None = None")
    return "\n".join(lines) + "\n"
