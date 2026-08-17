"""Emit TypeScript interface declarations from the contract IR."""

from __future__ import annotations

from schema import Contract, Field

_PRIMITIVE_TS = {
    "string": "string",
    "number": "number",
    "integer": "number",
    "boolean": "boolean",
}


def _field_type(item: Field) -> str:
    if item.type == "object":
        return str(item.ref)
    if item.type == "array":
        assert item.items is not None
        return f"{_field_type(item.items)}[]"
    if item.enum is not None:
        return " | ".join(f'"{value}"' for value in item.enum)
    return _PRIMITIVE_TS[item.type]


def to_typescript(contract: Contract) -> str:
    """Return a TypeScript ``export interface`` block for one contract."""
    lines = [f"/** {contract.description} */", f"export interface {contract.name} {{"]
    for item in contract.fields:
        if item.description:
            lines.append(f"  /** {item.description} */")
        optional = "?" if item.optional else ""
        lines.append(f"  {item.name}{optional}: {_field_type(item)};")
    lines.append("}")
    return "\n".join(lines) + "\n"
