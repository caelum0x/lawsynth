#!/usr/bin/env python3
"""Command-line entry point for the LawSynth benchmark execution engine.

Usage:
    python3 run.py [--family FAMILY] [--json PATH] [--workdir DIR] [--build]

The runner generates each case's dataset deterministically, executes the real
discovery/simulation engine through the compiled ``lawsynth`` CLI (or the
public Python SDK), scores the result with the architecture §16 candidate
score vector, and honestly reports capability boundaries.  It exits non-zero
only when a ``supported`` case regresses.
"""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path

BENCHMARKS = Path(__file__).resolve().parent
if str(BENCHMARKS) not in sys.path:
    sys.path.insert(0, str(BENCHMARKS))

from _runner import run_suite, write_reports


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Run the LawSynth benchmark suite.")
    parser.add_argument("--family", default=None, help="restrict to one family (e.g. dynamics)")
    parser.add_argument("--json", type=Path, default=None, help="write the JSON summary here")
    parser.add_argument("--workdir", type=Path, default=None, help="directory for generated data")
    parser.add_argument(
        "--build",
        action="store_true",
        help="build the CLI binary if it is not already compiled",
    )
    arguments = parser.parse_args(argv)

    if arguments.workdir is not None:
        arguments.workdir.mkdir(parents=True, exist_ok=True)
        summary = run_suite(
            BENCHMARKS,
            arguments.workdir,
            family_filter=arguments.family,
            allow_build=arguments.build,
        )
        write_reports(summary, arguments.json)
    else:
        with tempfile.TemporaryDirectory(prefix="lawsynth-bench-") as temporary:
            summary = run_suite(
                BENCHMARKS,
                Path(temporary),
                family_filter=arguments.family,
                allow_build=arguments.build,
            )
            write_reports(summary, arguments.json)

    return 0 if summary["passed_suite"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
