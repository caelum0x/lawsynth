"""Render the TypeScript surface of the API types as Markdown documentation."""

from __future__ import annotations

from rust import Schema, TypeRef

_TS = {
    "u8": "number", "u16": "number", "u32": "number", "u64": "number", "u128": "number",
    "i8": "number", "i16": "number", "i32": "number", "i64": "number",
    "f32": "number", "f64": "number",
    "bool": "boolean", "String": "string", "str": "string",
}


def render_type(type_ref: TypeRef) -> str:
    if type_ref.kind == "primitive":
        return _TS.get(type_ref.name, "unknown")
    if type_ref.kind == "named":
        return type_ref.name
    if type_ref.kind == "optional":
        assert type_ref.inner is not None
        return f"{render_type(type_ref.inner)} | null"
    assert type_ref.inner is not None
    return f"Array<{render_type(type_ref.inner)}>"


def render(schema: Schema) -> str:
    lines = [
        "# TypeScript API types (`@lawsynth/api-types`)",
        "",
        "The TypeScript surface mirrors the Rust `lawsynth-api-types` definitions.",
        "",
    ]

    if schema.newtypes:
        lines.append("## Identifiers")
        lines.append("")
        for newtype in schema.newtypes:
            base = _TS.get(newtype.base, "string")
            lines.append(f"- `type {newtype.name} = {base}`")
        lines.append("")

    if schema.enums:
        lines.append("## Union types")
        lines.append("")
        for enum in schema.enums:
            if enum.variants:
                members = " | ".join(f'"{variant}"' for variant in enum.variants)
                lines.append(f"- `type {enum.name} = {members}`")
            else:
                lines.append(f"- `type {enum.name} = never` _(data variants)_")
        lines.append("")

    if schema.structs:
        lines.append("## Interfaces")
        lines.append("")
        for struct in schema.structs:
            lines.append(f"### `{struct.name}`")
            lines.append("")
            if struct.fields:
                lines.append("| Field | Type |")
                lines.append("| --- | --- |")
                for field_def in struct.fields:
                    optional = "?" if field_def.type.kind == "optional" else ""
                    lines.append(
                        f"| `{field_def.name}{optional}` | `{render_type(field_def.type)}` |"
                    )
            else:
                lines.append("_No fields._")
            lines.append("")

    return "\n".join(lines).rstrip() + "\n"
