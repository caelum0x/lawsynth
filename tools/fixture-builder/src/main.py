"""Command-line entry point for lawsynth-fixture-builder.

Builds deterministic, canonically-encoded JSON test fixtures from a declarative
spec file. Output is byte-stable and offline: rerunning ``build`` on the same
spec yields identical files and checksums.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import package as package_mod
from canonicalize import canonical_json
from checksum import sha256_hex


def _load_specs(path: Path) -> list[dict]:
    data = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(data, dict):
        data = data.get("fixtures", [data])
    if not isinstance(data, list):
        raise ValueError("spec file must be a list of fixtures or an object with 'fixtures'")
    return data


def _cmd_build(args: argparse.Namespace) -> int:
    specs = _load_specs(Path(args.spec))
    built = package_mod.build_set(specs)
    if args.out is not None:
        manifest_path = package_mod.write_set(built, Path(args.out))
        for fixture in built:
            print(f"{fixture.sha256[:12]}  {fixture.filename}")
        print(f"manifest: {manifest_path}")
    else:
        sys.stdout.write(canonical_json(package_mod.manifest(built)))
    return 0


def _cmd_checksum(args: argparse.Namespace) -> int:
    data = Path(args.file).read_bytes()
    print(sha256_hex(data))
    return 0


def _cmd_verify(args: argparse.Namespace) -> int:
    """Rebuild from a spec and confirm files on disk still match the manifest."""
    specs = _load_specs(Path(args.spec))
    built = {fixture.filename: fixture for fixture in package_mod.build_set(specs)}
    root = Path(args.dir)
    ok = True
    for filename, fixture in built.items():
        path = root / filename
        if not path.exists():
            print(f"MISSING {filename}")
            ok = False
            continue
        actual = sha256_hex(path.read_bytes())
        if actual != fixture.sha256:
            print(f"CHANGED {filename}")
            ok = False
    print("OK" if ok else "FAIL")
    return 0 if ok else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="lawsynth-fixture-builder", description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    build = sub.add_parser("build", help="build fixtures from a spec file")
    build.add_argument("spec", help="JSON spec: a list of fixtures")
    build.add_argument("--out", help="write fixtures and manifest to a directory")
    build.set_defaults(func=_cmd_build)

    checksum = sub.add_parser("checksum", help="print the SHA-256 of a file")
    checksum.add_argument("file")
    checksum.set_defaults(func=_cmd_checksum)

    verify = sub.add_parser("verify", help="verify built fixtures against a spec")
    verify.add_argument("spec")
    verify.add_argument("dir")
    verify.set_defaults(func=_cmd_verify)

    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
