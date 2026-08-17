"""Assemble a set of fixtures into a directory with a checksum manifest."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from canonicalize import canonical_bytes
from checksum import sha256_hex
from generate import build_fixture


@dataclass(frozen=True)
class BuiltFixture:
    name: str
    filename: str
    content: bytes
    sha256: str


def build_set(specs: list[dict[str, Any]]) -> list[BuiltFixture]:
    """Build every fixture in ``specs`` in memory, without touching disk."""
    built: list[BuiltFixture] = []
    seen: set[str] = set()
    for spec in specs:
        name = str(spec["name"])
        if name in seen:
            raise ValueError(f"duplicate fixture name: {name}")
        seen.add(name)
        content = canonical_bytes(build_fixture(spec))
        built.append(
            BuiltFixture(
                name=name,
                filename=f"{name}.json",
                content=content,
                sha256=sha256_hex(content),
            )
        )
    return sorted(built, key=lambda fixture: fixture.name)


def manifest(built: list[BuiltFixture]) -> dict[str, Any]:
    """Return a deterministic manifest describing the built fixture set."""
    return {
        "fixtures": [
            {"name": fixture.name, "file": fixture.filename, "sha256": fixture.sha256}
            for fixture in built
        ]
    }


def write_set(built: list[BuiltFixture], out_dir: Path) -> Path:
    """Write fixtures and ``manifest.json`` to ``out_dir``; return the manifest path."""
    out_dir.mkdir(parents=True, exist_ok=True)
    for fixture in built:
        (out_dir / fixture.filename).write_bytes(fixture.content)
    manifest_path = out_dir / "manifest.json"
    manifest_path.write_bytes(canonical_bytes(manifest(built)))
    return manifest_path
