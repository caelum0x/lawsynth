"""Command-line entry point for lawsynth-license-check.

Scans dependency manifests, verifies their licenses against an allowlist that
mirrors the repository ``deny.toml``, and can emit an attribution NOTICE.
Everything is offline and deterministic.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import notice as notice_mod
import policy as policy_mod
import report as report_mod
import scan


def _resolve_policy(path_arg: str | None) -> policy_mod.Policy:
    if path_arg is None:
        return policy_mod.Policy()
    return policy_mod.load_policy(Path(path_arg))


def _cmd_check(args: argparse.Namespace) -> int:
    dependencies = scan.scan_paths([Path(p) for p in args.manifests])
    policy = _resolve_policy(args.policy)
    result = report_mod.evaluate(dependencies, policy)
    if args.format == "json":
        sys.stdout.write(report_mod.format_json(result))
    else:
        sys.stdout.write(report_mod.format_text(result))
    if result.denied:
        return 1
    if result.unknown and not args.allow_unknown:
        return 1
    return 0


def _cmd_notice(args: argparse.Namespace) -> int:
    dependencies = scan.scan_paths([Path(p) for p in args.manifests])
    body = notice_mod.render_notice(dependencies)
    if args.out is not None:
        Path(args.out).write_text(body, encoding="utf-8")
        print(f"wrote {args.out}")
    else:
        sys.stdout.write(body)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="lawsynth-license-check", description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    check = sub.add_parser("check", help="verify dependency licenses against a policy")
    check.add_argument("manifests", nargs="+", help="Cargo.lock, package.json, or inventory JSON")
    check.add_argument("--policy", help="deny.toml providing the allowlist")
    check.add_argument("--format", choices=("text", "json"), default="text")
    check.add_argument(
        "--allow-unknown",
        action="store_true",
        help="do not fail when a dependency has no recorded license",
    )
    check.set_defaults(func=_cmd_check)

    notice = sub.add_parser("notice", help="generate an attribution NOTICE")
    notice.add_argument("manifests", nargs="+")
    notice.add_argument("--out", help="write to a file instead of stdout")
    notice.set_defaults(func=_cmd_notice)

    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
