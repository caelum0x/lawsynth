"""Command-line entry point for the LawSynth bundle inspector.

Usage:
    bundle-inspector inspect WORLD.lsworld [--json]

The command reads a ``.lsworld`` bundle, validates its container and manifest,
verifies the SHA-256 checksum manifest, decodes the world payload, and prints a
human-readable or JSON report.  Exit code is ``0`` when the bundle is valid and
all checksums verify, ``1`` on any integrity or format error.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence

from archive import InvalidArchive, read_archive
from checksum import verify_archive
from manifest import decode_world, validate_manifest
from report import build_inspection, to_json, to_text


def inspect(path: str) -> tuple[bool, str, str]:
    """Inspect a bundle and return ``(ok, text_report, json_report)``."""
    archive = read_archive(path)
    validate_manifest(archive)
    checksums = verify_archive(archive)
    world = decode_world(archive)
    inspection = build_inspection(archive, checksums, world)
    return inspection.ok, to_text(inspection), to_json(inspection)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="bundle-inspector",
        description="Inspect a LawSynth .lsworld bundle.",
    )
    sub = parser.add_subparsers(dest="command", required=True)
    inspect_parser = sub.add_parser("inspect", help="inspect a .lsworld bundle")
    inspect_parser.add_argument("bundle", help="path to a .lsworld file")
    inspect_parser.add_argument(
        "--json", action="store_true", help="emit a JSON report instead of text"
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.command == "inspect":
        try:
            ok, text, payload = inspect(args.bundle)
        except InvalidArchive as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        print(payload if args.json else text)
        return 0 if ok else 1
    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
