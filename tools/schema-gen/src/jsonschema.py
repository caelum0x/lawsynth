"""Emit JSON Schema (draft 2020-12) documents from the contract IR."""

from __future__ import annotations

from typing import Any

from schema import Contract, Field

SCHEMA_DIALECT = "https://json-schema.org/draft/2020-12/schema"


def _field_schema(item: Field) -> dict[str, Any]:
    """Render a single field to its JSON Schema fragment."""
    if item.type == "object":
        return {"$ref": f"./{item.ref}.schema.json"}
    if item.type == "array":
        assert item.items is not None  # guaranteed by Field.__post_init__
        return {"type": "array", "items": _field_schema(item.items)}

    node: dict[str, Any] = {"type": item.type}
    if item.description:
        node["description"] = item.description
    if item.enum is not None:
        node["enum"] = list(item.enum)
    if item.pattern is not None:
        node["pattern"] = item.pattern
    if item.minimum is not None:
        node["minimum"] = item.minimum
    return node


def to_json_schema(contract: Contract) -> dict[str, Any]:
    """Return a standalone JSON Schema object for one contract."""
    properties = {item.name: _field_schema(item) for item in contract.fields}
    return {
        "$schema": SCHEMA_DIALECT,
        "$id": f"https://lawsynth.dev/schemas/{contract.name}.schema.json",
        "title": contract.title,
        "description": contract.description,
        "type": "object",
        "properties": properties,
        "required": contract.required(),
        "additionalProperties": False,
    }
