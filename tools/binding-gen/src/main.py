"""Command-line entry point for the LawSynth binding generator.

Usage:
    binding-gen --lang {python,typescript,proto,rust} [--crate DIR] [--out FILE]

Scans ``crates/lawsynth-api-types`` for the public API type surface and emits
aligned binding stubs / type declarations in the requested language.  Output is
deterministic: the same crate always produces byte-identical files.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from pathlib import Path

import protobuf
import python
import rust
import typescript
from rust import Schema, scan_crate

DEFAULT_CRATE = Path("crates/lawsynth-api-types")

_GENERATORS = {
    "python": python.render,
    "typescript": typescript.render,
    "proto": protobuf.render,
    "rust": rust.render,
}


def generate(schema: Schema, language: str) -> str:
    try:
        return _GENERATORS[language](schema)
    except KeyError as error:
        raise ValueError(f"unknown language: {language}") from error


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="binding-gen",
        description="Generate LawSynth API bindings from the Rust api-types crate.",
    )
    parser.add_argument(
        "--lang",
        required=True,
        choices=sorted(_GENERATORS),
        help="target binding language",
    )
    parser.add_argument(
        "--crate",
        type=Path,
        default=DEFAULT_CRATE,
        help="path to the lawsynth-api-types crate",
    )
    parser.add_argument(
        "--out",
        type=Path,
        help="write to this file instead of stdout",
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        schema = scan_crate(args.crate)
    except FileNotFoundError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    output = generate(schema, args.lang)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(output, encoding="utf-8")
    else:
        sys.stdout.write(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
