"""Command-line entry point for the LawSynth benchmark site generator.

Usage:
    benchmark-site build BENCHMARKS_DIR [--out DIR]

Loads every benchmark case under ``BENCHMARKS_DIR``, classifies each against its
expected status, and renders a static ``index.html`` + ``results.json`` site.
Exit code is ``0`` when no regressions or failures are present, ``1`` otherwise.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from pathlib import Path

from compare import summarize
from publish import render_site, write_site
from results import load_results


def build_site(benchmarks_dir: Path, out_dir: Path | None) -> bool:
    """Build the site and return ``True`` when there are no problems."""
    results = load_results(benchmarks_dir)
    summary = summarize(results)
    site = render_site(results, summary)
    if out_dir is not None:
        write_site(site, out_dir)
    else:
        sys.stdout.write(site.results_json + "\n")
    return not summary.has_problems


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="benchmark-site",
        description="Render LawSynth benchmark results into a static site.",
    )
    sub = parser.add_subparsers(dest="command", required=True)
    build = sub.add_parser("build", help="build the static site")
    build.add_argument("benchmarks", type=Path, help="path to the benchmarks/ directory")
    build.add_argument("--out", type=Path, help="output directory for the site")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.command == "build":
        try:
            ok = build_site(args.benchmarks, args.out)
        except FileNotFoundError as error:
            print(f"error: {error}", file=sys.stderr)
            return 2
        return 0 if ok else 1
    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
