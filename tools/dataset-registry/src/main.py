"""Command-line entry point for lawsynth-dataset-registry.

Indexes the scientific benchmark datasets under ``benchmarks/``, verifies their
checksums, stages them locally, and renders dataset cards. Fully offline.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import card as card_mod
import download
import manifest as manifest_mod
import verify as verify_mod


def _canonical_json(document: object) -> str:
    """Deterministic JSON: sorted keys, two-space indent, trailing newline."""
    return json.dumps(document, indent=2, sort_keys=True) + "\n"


def _cmd_index(args: argparse.Namespace) -> int:
    root = Path(args.root)
    entries = manifest_mod.index_tree(root)
    document = manifest_mod.registry_document(entries)
    text = _canonical_json(document)
    if args.out is not None:
        Path(args.out).write_text(text, encoding="utf-8")
        print(f"indexed {len(entries)} datasets -> {args.out}")
    else:
        sys.stdout.write(text)
    return 0


def _cmd_verify(args: argparse.Namespace) -> int:
    entries = manifest_mod.load_registry(Path(args.registry))
    root = Path(args.root)
    problems = verify_mod.verify_registry(entries, root)
    for problem in problems:
        print(f"{problem.kind.upper():8} {problem.dataset} :: {problem.file}")
    print(f"verified {len(entries)} datasets: {len(problems)} problems")
    return 0 if not problems else 1


def _cmd_card(args: argparse.Namespace) -> int:
    entries = manifest_mod.load_registry(Path(args.registry))
    try:
        entry = download.find_entry(entries, args.id)
    except download.DatasetNotFound as error:
        print(error, file=sys.stderr)
        return 2
    sys.stdout.write(card_mod.render_card(entry))
    return 0


def _cmd_stage(args: argparse.Namespace) -> int:
    entries = manifest_mod.load_registry(Path(args.registry))
    try:
        entry = download.find_entry(entries, args.id)
        staged = download.stage(entry, Path(args.root), Path(args.dest))
    except download.DatasetNotFound as error:
        print(error, file=sys.stderr)
        return 2
    for path in staged:
        print(path)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="lawsynth-dataset-registry", description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    index = sub.add_parser("index", help="index benchmark datasets into a registry")
    index.add_argument("root", help="repository root or benchmarks directory")
    index.add_argument("--out", help="write the registry JSON to a file")
    index.set_defaults(func=_cmd_index)

    verify = sub.add_parser("verify", help="verify a registry against files on disk")
    verify.add_argument("registry", help="registry JSON produced by 'index'")
    verify.add_argument("--root", default=".", help="root the registry paths are relative to")
    verify.set_defaults(func=_cmd_verify)

    card = sub.add_parser("card", help="render a Markdown dataset card")
    card.add_argument("registry")
    card.add_argument("id", help="dataset id, e.g. dynamics/ode-small")
    card.set_defaults(func=_cmd_card)

    stage = sub.add_parser("stage", help="copy a dataset's declarative files locally")
    stage.add_argument("registry")
    stage.add_argument("id")
    stage.add_argument("dest")
    stage.add_argument("--root", default=".")
    stage.set_defaults(func=_cmd_stage)

    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
