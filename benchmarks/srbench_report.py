#!/usr/bin/env python3
"""SRBench-format report over LawSynth's credibility benchmark families.

Discovers every ``benchmark.toml`` under the SRBench-style families
(``strogatz/``, ``feynman/``, ``blackbox/``), executes each through the real
compiled ``lawsynth`` CLI, and prints the standard accuracy--simplicity--time
summary plus a **determinism** column no stochastic competitor can report (each
case is discovered twice and the worlds compared byte-for-byte).

Deterministic and offline. Exits non-zero only on a real regression of a
``supported`` case, so it is CI-usable.

Usage:
    PYTHONPATH="benchmarks:python/lawsynth/src" python3 benchmarks/srbench_report.py [--json out.json] [--family strogatz]
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

from _engine import ensure_binary, locate_binary
from _srbench import run_srbench_case

FAMILIES = ("strogatz", "feynman", "blackbox")
ROOT = Path(__file__).resolve().parent.parent


def discover_cases(family: str | None) -> list[Path]:
    families = [family] if family else list(FAMILIES)
    cases: list[Path] = []
    for fam in families:
        base = ROOT / "benchmarks" / fam
        if base.is_dir():
            cases.extend(sorted(p.parent for p in base.glob("*/benchmark.toml")))
    return cases


def _fmt(value: Any) -> str:
    if value is None:
        return "—"
    if isinstance(value, float):
        return f"{value:.4f}"
    return str(value)


def build_report(family: str | None = None, *, allow_build: bool = False) -> dict[str, Any]:
    cases = discover_cases(family)
    binary = ensure_binary(ROOT, allow_build=allow_build) if allow_build else locate_binary(ROOT)
    rows: list[dict[str, Any]] = []
    regressions: list[str] = []
    with tempfile.TemporaryDirectory(prefix="srbench-") as tmp:
        for index, case in enumerate(cases):
            workdir = Path(tmp) / f"case-{index:03d}"
            result = run_srbench_case(case, workdir, binary, check_determinism=True)
            sv = result.get("score_vector", {})
            row = {
                "id": result.get("id") or case.name,
                "family": result.get("family"),
                "status": result.get("status"),
                "passed": result.get("passed"),
                "trajectory_r2": sv.get("trajectory_r2"),
                "complexity": sv.get("complexity_nodes"),
                "training_time_ns": sv.get("training_time_ns"),
                "symbolic_level": sv.get("symbolic_level"),
                "determinism": result.get("determinism"),
            }
            rows.append(row)
            if result.get("status") == "regression":
                regressions.append(row["id"])
    return {
        "binary_available": binary is not None,
        "total": len(rows),
        "supported_regressions": regressions,
        "rows": rows,
    }


def render_table(report: dict[str, Any]) -> str:
    header = f"{'id':32} {'status':10} {'R²':>8} {'nodes':>6} {'det':>5}"
    lines = [header, "-" * len(header)]
    for row in report["rows"]:
        lines.append(
            f"{row['id']:32} {str(row['status']):10} "
            f"{_fmt(row['trajectory_r2']):>8} {_fmt(row['complexity']):>6} "
            f"{_fmt(row['determinism']):>5}"
        )
    passed = sum(1 for r in report["rows"] if r["passed"])
    boundaries = sum(1 for r in report["rows"] if r["status"] == "capability-boundary")
    lines.append("-" * len(header))
    lines.append(
        f"{report['total']} case(s): {passed} passed, {boundaries} boundary, "
        f"{len(report['supported_regressions'])} regression"
    )
    det = [r["determinism"] for r in report["rows"] if r["determinism"] is not None]
    if det:
        lines.append(f"determinism: {sum(1 for d in det if d)}/{len(det)} byte-identical on replay")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="SRBench-format report for LawSynth")
    parser.add_argument("--json", type=Path, default=None, help="write the report JSON here")
    parser.add_argument("--family", choices=FAMILIES, default=None, help="restrict to one family")
    parser.add_argument("--build", action="store_true", help="build the CLI if missing")
    args = parser.parse_args(argv)

    report = build_report(args.family, allow_build=args.build)
    if not report["binary_available"]:
        print("SRBench: lawsynth CLI binary not found (build with `cargo build -p lawsynth-cli` "
              "or pass --build).", file=sys.stderr)
        return 0
    print(render_table(report))
    if args.json is not None:
        args.json.write_text(json.dumps(report, indent=2, sort_keys=True), encoding="utf-8")
    return 1 if report["supported_regressions"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
