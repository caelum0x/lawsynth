"""Command line interface for inspecting stored benchmark results."""
import argparse, json
from pathlib import Path
from .baseline import compare
from .dataset import load_observations
from .render import markdown
from .report import build

def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(prog="lawsynth-bench", description="Analyze recorded LawSynth benchmark data")
    sub = result.add_subparsers(dest="command", required=True)
    summarize = sub.add_parser("summarize"); summarize.add_argument("input", type=Path); summarize.add_argument("--format", choices=("json", "markdown"), default="markdown")
    comparison = sub.add_parser("compare"); comparison.add_argument("baseline", type=Path); comparison.add_argument("candidate", type=Path); comparison.add_argument("--format", choices=("json", "markdown"), default="markdown")
    return result

def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    if args.command == "summarize": report = build(load_observations(args.input))
    else:
        candidate = load_observations(args.candidate)
        report = build(candidate, compare(load_observations(args.baseline), candidate))
    print(markdown(report) if args.format == "markdown" else json.dumps(report, sort_keys=True, indent=2))
    return 1 if report["regression_count"] else 0

if __name__ == "__main__": raise SystemExit(main())
