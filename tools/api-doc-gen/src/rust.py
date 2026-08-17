"""Shared API-types model plus the Rust documentation surface.

``crates/lawsynth-api-types`` is the authoritative definition of LawSynth's
public API values.  This module scans that crate into a small language-neutral
schema (IR) that the OpenAPI, Python, and TypeScript doc renderers consume, and
renders the Rust surface itself as Markdown.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

PRIMITIVES = {
    "u8", "u16", "u32", "u64", "u128",
    "i8", "i16", "i32", "i64",
    "f32", "f64", "bool", "String", "str",
}


@dataclass(frozen=True)
class TypeRef:
    kind: str  # "primitive" | "named" | "optional" | "list"
    name: str = ""
    inner: "TypeRef | None" = None

    def render_rust(self) -> str:
        if self.kind == "primitive" or self.kind == "named":
            return self.name
        if self.kind == "optional":
            assert self.inner is not None
            return f"Option<{self.inner.render_rust()}>"
        assert self.inner is not None
        return f"Vec<{self.inner.render_rust()}>"


@dataclass(frozen=True)
class Field:
    name: str
    type: TypeRef


@dataclass(frozen=True)
class EnumDef:
    name: str
    variants: tuple[str, ...]


@dataclass(frozen=True)
class NewtypeDef:
    name: str
    base: str


@dataclass(frozen=True)
class StructDef:
    name: str
    fields: tuple[Field, ...]


@dataclass(frozen=True)
class Schema:
    enums: tuple[EnumDef, ...] = field(default_factory=tuple)
    newtypes: tuple[NewtypeDef, ...] = field(default_factory=tuple)
    structs: tuple[StructDef, ...] = field(default_factory=tuple)

    @property
    def type_names(self) -> set[str]:
        return (
            {enum.name for enum in self.enums}
            | {newtype.name for newtype in self.newtypes}
            | {struct.name for struct in self.structs}
        )


def parse_type(raw: str) -> TypeRef:
    text = raw.strip()
    if text.startswith("Option<") and text.endswith(">"):
        return TypeRef("optional", inner=parse_type(text[len("Option<") : -1]))
    if text.startswith("Vec<") and text.endswith(">"):
        return TypeRef("list", inner=parse_type(text[len("Vec<") : -1]))
    if text in PRIMITIVES:
        return TypeRef("primitive", name=text)
    return TypeRef("named", name=text)


_ENUM_RE = re.compile(r"pub enum\s+(\w+)\s*\{(.*?)\}", re.DOTALL)
_NEWTYPE_RE = re.compile(r"pub struct\s+(\w+)\s*\(\s*(\w+)\s*\)\s*;")
_STRUCT_RE = re.compile(r"pub struct\s+(\w+)\s*\{(.*?)\}", re.DOTALL)
_VARIANT_RE = re.compile(r"^\s*([A-Z]\w*)\s*,?\s*$")
_FIELD_RE = re.compile(r"pub\s+(\w+)\s*:\s*([^,]+),")


def _enum_variants(body: str) -> tuple[str, ...]:
    return tuple(
        match.group(1)
        for line in body.splitlines()
        if (match := _VARIANT_RE.match(line))
    )


def _struct_fields(body: str) -> tuple[Field, ...]:
    return tuple(
        Field(name=match.group(1), type=parse_type(match.group(2)))
        for match in _FIELD_RE.finditer(body)
    )


def scan_source(text: str) -> Schema:
    enums = tuple(
        EnumDef(match.group(1), _enum_variants(match.group(2)))
        for match in _ENUM_RE.finditer(text)
    )
    newtypes = tuple(
        NewtypeDef(match.group(1), match.group(2))
        for match in _NEWTYPE_RE.finditer(text)
    )
    structs = tuple(
        StructDef(match.group(1), _struct_fields(match.group(2)))
        for match in _STRUCT_RE.finditer(text)
        if "(" not in match.group(0).split("{", 1)[0]
    )
    return Schema(
        enums=tuple(sorted(enums, key=lambda item: item.name)),
        newtypes=tuple(sorted(newtypes, key=lambda item: item.name)),
        structs=tuple(sorted(structs, key=lambda item: item.name)),
    )


def scan_crate(crate_dir: str | Path) -> Schema:
    src = Path(crate_dir) / "src"
    if not src.is_dir():
        raise FileNotFoundError(f"no src directory under {crate_dir}")
    blob = "\n".join(path.read_text(encoding="utf-8") for path in sorted(src.glob("*.rs")))
    return scan_source(blob)


def render(schema: Schema) -> str:
    """Render the Rust surface as Markdown reference documentation."""
    lines = ["# Rust API types (`lawsynth-api-types`)", ""]

    if schema.newtypes:
        lines.append("## Identifiers")
        lines.append("")
        lines.append("| Type | Underlying |")
        lines.append("| --- | --- |")
        for newtype in schema.newtypes:
            lines.append(f"| `{newtype.name}` | `{newtype.base}` |")
        lines.append("")

    if schema.enums:
        lines.append("## Enums")
        lines.append("")
        for enum in schema.enums:
            variants = ", ".join(f"`{variant}`" for variant in enum.variants) or "_(data variants)_"
            lines.append(f"### `{enum.name}`")
            lines.append("")
            lines.append(f"Variants: {variants}")
            lines.append("")

    if schema.structs:
        lines.append("## Structs")
        lines.append("")
        for struct in schema.structs:
            lines.append(f"### `{struct.name}`")
            lines.append("")
            if struct.fields:
                lines.append("| Field | Type |")
                lines.append("| --- | --- |")
                for field_def in struct.fields:
                    lines.append(f"| `{field_def.name}` | `{field_def.type.render_rust()}` |")
            else:
                lines.append("_No public fields._")
            lines.append("")

    return "\n".join(lines).rstrip() + "\n"
