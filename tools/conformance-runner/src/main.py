"""Command-line entry point for the LawSynth conformance runner.

Usage:
    conformance-runner run ROOT [--filter SUBSTRING] [--json] [--timeout SECONDS]

Discovers every conformance case under ``ROOT`` (each is a directory containing
``case.toml``), executes its runner, compares the observed result against the
case's ``expected.json``, and reports pass/fail deterministically.  Exit code is
``0`` when every case passes, ``1`` when any case fails, ``2`` on a usage or
discovery error.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence
from pathlib import Path

from compare import compare
from discover import DiscoveryError, discover_cases
from execute import Runner, execute_case
from report import CaseOutcome, Report, build_report, to_json, to_text


def run_suite(
    root: Path,
    name_filter: str | None = None,
    runner: Runner | None = None,
    timeout: float = 300.0,
) -> Report:
    """Discover, execute, and compare every case under ``root``."""
    cases = discover_cases(root, name_filter)
    outcomes: list[CaseOutcome] = []
    for case in cases:
        execution = execute_case(case, runner=runner, timeout=timeout)
        comparison = compare(case.expected, execution.observed)
        outcomes.append(CaseOutcome.build(case.case_id, execution, comparison))
    return build_report(outcomes)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="conformance-runner",
        description="Run LawSynth cross-language conformance cases.",
    )
    sub = parser.add_subparsers(dest="command", required=True)
    run_parser = sub.add_parser("run", help="run cases discovered under a root directory")
    run_parser.add_argument("root", type=Path, help="directory containing conformance cases")
    run_parser.add_argument("--filter", dest="name_filter", help="only run cases whose id matches")
    run_parser.add_argument("--json", action="store_true", help="emit a JSON report")
    run_parser.add_argument("--timeout", type=float, default=300.0, help="per-case timeout (s)")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.command == "run":
        try:
            report = run_suite(args.root, args.name_filter, timeout=args.timeout)
        except DiscoveryError as error:
            print(f"error: {error}", file=sys.stderr)
            return 2
        print(to_json(report) if args.json else to_text(report))
        return 0 if report.ok else 1
    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
