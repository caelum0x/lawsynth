"""Command-line entry point for lawsynth-schema-gen.

Generates JSON Schema, TypeScript, and Python type definitions from the
LawSynth specification contracts.  All output is deterministic and offline.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import jsonschema as js
import python as py
import schema
import typescript as ts

_PY_HEADER = (
    '"""Generated LawSynth types. Do not edit by hand."""\n\n'
    "from __future__ import annotations\n\n"
    "from dataclasses import dataclass\n"
    "from typing import Literal\n\n\n"
)


def _ordered_contracts(name: str | None) -> list[schema.Contract]:
    registry = schema.contracts()
    if name is not None:
        return [schema.get_contract(name)]
    return [registry[key] for key in sorted(registry)]


def render(kind: str, name: str | None) -> str:
    """Return the generated artifact for ``kind`` as a single string."""
    selected = _ordered_contracts(name)
    if kind == "json":
        payload = {c.name: js.to_json_schema(c) for c in selected}
        return json.dumps(payload, indent=2, sort_keys=True) + "\n"
    if kind == "ts":
        return "\n".join(ts.to_typescript(c) for c in selected)
    if kind == "py":
        return _PY_HEADER + "\n\n".join(py.to_python(c) for c in selected)
    raise ValueError(f"unknown output kind {kind!r}")


def _write_files(kind: str, name: str | None, out: Path) -> list[Path]:
    out.mkdir(parents=True, exist_ok=True)
    written: list[Path] = []
    if kind == "json":
        for contract in _ordered_contracts(name):
            path = out / f"{contract.name}.schema.json"
            body = json.dumps(js.to_json_schema(contract), indent=2, sort_keys=True) + "\n"
            path.write_text(body, encoding="utf-8")
            written.append(path)
    else:
        suffix = {"ts": "ts", "py": "py"}[kind]
        path = out / f"lawsynth_types.{suffix}"
        path.write_text(render(kind, name), encoding="utf-8")
        written.append(path)
    return written


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="lawsynth-schema-gen", description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("list", help="list available specification contracts")

    for kind, helptext in (
        ("json", "emit JSON Schema documents"),
        ("ts", "emit TypeScript interfaces"),
        ("py", "emit Python dataclasses"),
    ):
        emitter = sub.add_parser(kind, help=helptext)
        emitter.add_argument("--contract", help="restrict output to a single contract")
        emitter.add_argument("--out", type=Path, help="write to a directory instead of stdout")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "list":
        for name in sorted(schema.contracts()):
            print(name)
        return 0
    try:
        if args.out is not None:
            for path in _write_files(args.command, args.contract, args.out):
                print(path)
        else:
            sys.stdout.write(render(args.command, args.contract))
    except KeyError as error:
        print(error, file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
