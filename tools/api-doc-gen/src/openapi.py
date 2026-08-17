"""Render an OpenAPI 3.1 document from the API-types schema.

Component schemas are derived directly from the Rust ``lawsynth-api-types``
surface.  Per ``specs/service-api/resources.md`` no endpoint paths are normative
in this release, so the generated read paths are illustrative collection/item
routes over the descriptor types and are marked as such.
"""

from __future__ import annotations

import json

from rust import Schema, TypeRef

_INTEGER = {"u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64"}
_NUMBER = {"f32", "f64"}

# Descriptor types that make sense as illustrative REST collections.
_RESOURCES = {
    "Project": "projects",
    "DatasetDescriptor": "datasets",
    "WorldRevision": "worlds",
    "RunSummary": "runs",
    "ArtifactDescriptor": "artifacts",
}


def _primitive_schema(name: str) -> dict[str, object]:
    if name in _INTEGER:
        schema: dict[str, object] = {"type": "integer"}
        if name in ("u64", "i64", "u128"):
            schema["format"] = "int64"
        return schema
    if name in _NUMBER:
        return {"type": "number", "format": "double"}
    if name == "bool":
        return {"type": "boolean"}
    return {"type": "string"}


def type_schema(type_ref: TypeRef, newtypes: set[str]) -> dict[str, object]:
    if type_ref.kind == "primitive":
        return _primitive_schema(type_ref.name)
    if type_ref.kind == "named":
        if type_ref.name in newtypes:
            return {"$ref": f"#/components/schemas/{type_ref.name}"}
        return {"$ref": f"#/components/schemas/{type_ref.name}"}
    if type_ref.kind == "optional":
        assert type_ref.inner is not None
        return type_schema(type_ref.inner, newtypes)
    assert type_ref.inner is not None
    return {"type": "array", "items": type_schema(type_ref.inner, newtypes)}


def _components(schema: Schema) -> dict[str, object]:
    newtype_names = {newtype.name for newtype in schema.newtypes}
    schemas: dict[str, object] = {}

    for newtype in schema.newtypes:
        schemas[newtype.name] = {
            "type": "string",
            "description": f"Validated identifier backed by {newtype.base}.",
        }
    for enum in schema.enums:
        if enum.variants:
            schemas[enum.name] = {"type": "string", "enum": list(enum.variants)}
        else:
            schemas[enum.name] = {"description": "Tagged union with data variants."}
    for struct in schema.structs:
        properties = {
            field_def.name: type_schema(field_def.type, newtype_names)
            for field_def in struct.fields
        }
        required = [
            field_def.name
            for field_def in struct.fields
            if field_def.type.kind != "optional"
        ]
        schemas[struct.name] = {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": False,
        }
    return {"schemas": dict(sorted(schemas.items()))}


def _paths(schema: Schema) -> dict[str, object]:
    paths: dict[str, object] = {}
    for type_name, collection in sorted(_RESOURCES.items(), key=lambda item: item[1]):
        if type_name not in schema.type_names:
            continue
        ref = {"$ref": f"#/components/schemas/{type_name}"}
        paths[f"/{collection}"] = {
            "get": {
                "summary": f"List {collection} (illustrative; not normative)",
                "responses": {
                    "200": {
                        "description": f"A page of {collection}.",
                        "content": {
                            "application/json": {
                                "schema": {"type": "array", "items": ref}
                            }
                        },
                    }
                },
            }
        }
        paths[f"/{collection}/{{id}}"] = {
            "get": {
                "summary": f"Fetch a single {type_name} (illustrative; not normative)",
                "parameters": [
                    {
                        "name": "id",
                        "in": "path",
                        "required": True,
                        "schema": {"type": "string"},
                    }
                ],
                "responses": {
                    "200": {
                        "description": type_name,
                        "content": {"application/json": {"schema": ref}},
                    }
                },
            }
        }
    return paths


def build_document(schema: Schema) -> dict[str, object]:
    return {
        "openapi": "3.1.0",
        "info": {
            "title": "LawSynth API",
            "version": "0.1.0",
            "description": (
                "Transport-neutral validated values from lawsynth-api-types. "
                "Endpoint paths are illustrative; see specs/service-api/resources.md."
            ),
        },
        "paths": _paths(schema),
        "components": _components(schema),
    }


def render(schema: Schema) -> str:
    return json.dumps(build_document(schema), indent=2, sort_keys=True) + "\n"
