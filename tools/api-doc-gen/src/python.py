"""Render the Python surface of the API types as Markdown documentation."""

from __future__ import annotations

from rust import Schema, TypeRef

_PY = {
    "u8": "int", "u16": "int", "u32": "int", "u64": "int", "u128": "int",
    "i8": "int", "i16": "int", "i32": "int", "i64": "int",
    "f32": "float", "f64": "float",
    "bool": "bool", "String": "str", "str": "str",
}


def render_type(type_ref: TypeRef) -> str:
    if type_ref.kind == "primitive":
        return _PY.get(type_ref.name, "Any")
    if type_ref.kind == "named":
        return type_ref.name
    if type_ref.kind == "optional":
        assert type_ref.inner is not None
        return f"{render_type(type_ref.inner)} | None"
    assert type_ref.inner is not None
    return f"list[{render_type(type_ref.inner)}]"


def render(schema: Schema) -> str:
    lines = [
        "# Python API types (`lawsynth`)",
        "",
        "The Python surface mirrors the Rust `lawsynth-api-types` definitions.",
        "",
    ]

    if schema.newtypes:
        lines.append("## Identifiers")
        lines.append("")
        for newtype in schema.newtypes:
            base = _PY.get(newtype.base, "str")
            lines.append(f"- `{newtype.name} = NewType(\"{newtype.name}\", {base})`")
        lines.append("")

    if schema.enums:
        lines.append("## Enums")
        lines.append("")
        for enum in schema.enums:
            values = ", ".join(f"`{variant}`" for variant in enum.variants) or "_(data variants)_"
            lines.append(f"- `class {enum.name}(str, Enum)` — {values}")
        lines.append("")

    if schema.structs:
        lines.append("## Dataclasses")
        lines.append("")
        for struct in schema.structs:
            lines.append(f"### `{struct.name}`")
            lines.append("")
            if struct.fields:
                lines.append("| Field | Type |")
                lines.append("| --- | --- |")
                for field_def in struct.fields:
                    lines.append(f"| `{field_def.name}` | `{render_type(field_def.type)}` |")
            else:
                lines.append("_No fields._")
            lines.append("")

    return "\n".join(lines).rstrip() + "\n"
