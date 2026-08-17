"""Command-line entry point for the LawSynth API documentation generator.

Usage:
    api-doc-gen --surface {openapi,python,rust,typescript,all} \\
                [--crate DIR] [--out PATH]

Scans ``crates/lawsynth-api-types`` for the public API surface and renders
documentation for the requested surface:

* ``openapi``     — an OpenAPI 3.1 JSON document
* ``rust``        — Markdown reference for the Rust types
* ``python``      — Markdown reference for the Python surface
* ``typescript``  — Markdown reference for the TypeScript surface
* ``all``         — writes every surface into ``--out`` (a directory)

Output is deterministic.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from pathlib import Path

import openapi
import python
import rust
import typescript
from rust import Schema, scan_crate

DEFAULT_CRATE = Path("crates/lawsynth-api-types")

_SURFACES = {
    "openapi": (openapi.render, "openapi.json"),
    "python": (python.render, "python.md"),
    "rust": (rust.render, "rust.md"),
    "typescript": (typescript.render, "typescript.md"),
}


def render_surface(schema: Schema, surface: str) -> str:
    return _SURFACES[surface][0](schema)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="api-doc-gen",
        description="Generate LawSynth API documentation from the Rust api-types crate.",
    )
    parser.add_argument(
        "--surface",
        required=True,
        choices=[*sorted(_SURFACES), "all"],
        help="which surface to document",
    )
    parser.add_argument("--crate", type=Path, default=DEFAULT_CRATE, help="api-types crate path")
    parser.add_argument("--out", type=Path, help="output file (single surface) or directory (all)")
    return parser


def _write_all(schema: Schema, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    for surface, (renderer, filename) in sorted(_SURFACES.items()):
        (out_dir / filename).write_text(renderer(schema), encoding="utf-8")


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        schema = scan_crate(args.crate)
    except FileNotFoundError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2

    if args.surface == "all":
        out_dir = args.out or Path("api-docs")
        _write_all(schema, out_dir)
        print(f"wrote {len(_SURFACES)} documents to {out_dir}")
        return 0

    output = render_surface(schema, args.surface)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(output, encoding="utf-8")
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
