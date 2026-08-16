"""Executable performance harnesses for public LawSynth workflows.

The benchmark fixtures describe a workload rather than a machine-specific
number.  Runs measure a real command (or the public Python SDK) with a
monotonic clock and persist their observations in the requested work
directory.  Checked-in baselines are deliberately qualitative; absolute
timings belong to the machine and CI environment that produced them.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Any

from _common import invoke_discovery, read_config, repository_root, write_dataset, write_json


def _python_sdk_workload(case_dir: Path) -> dict[str, Any]:
    """Exercise immutable public SDK objects without fabricating native work."""
    root = repository_root(case_dir)
    source = root / "python" / "lawsynth" / "src"
    if str(source) not in sys.path:
        sys.path.insert(0, str(source))
    from lawsynth import Dataset, DiscoveryConfig
    from lawsynth.variable import Variable

    started = time.perf_counter_ns()
    times = tuple(index / 1_000.0 for index in range(2_000))
    dataset = Dataset.from_columns(times, {"x": tuple(index / 2_000.0 for index in range(2_000))})
    variable = Variable("x", unit="1")
    config = DiscoveryConfig(polynomial_degree=3, threshold=0.05)
    elapsed = time.perf_counter_ns() - started
    return {
        "returncode": 0,
        "operation": "public-python-sdk-object-construction",
        "rows": len(dataset.time),
        "variable": variable.name,
        "degree": config.polynomial_degree,
        "elapsed_ns": elapsed,
    }


def run_workload(case_dir: Path, workdir: Path) -> dict[str, Any]:
    """Run the configured public workload and attach an elapsed duration."""
    workflow = read_config(case_dir)["workload"]["workflow"]
    if workflow == "python-sdk":
        return _python_sdk_workload(case_dir)
    if workflow != "native-cli":
        raise ValueError(f"unknown benchmark workflow {workflow!r}")
    started = time.perf_counter_ns()
    result = invoke_discovery(case_dir, workdir)
    result["elapsed_ns"] = time.perf_counter_ns() - started
    result["operation"] = "native-discover-inspect" + ("-simulate" if "simulate_returncode" in result else "")
    return result


def score_workload(case_dir: Path, result: dict[str, Any]) -> dict[str, Any]:
    """Report a portable, correctness-gated result instead of a fake SLA."""
    workflow = read_config(case_dir)["workload"]["workflow"]
    passed = result.get("returncode") == 0
    if workflow == "native-cli":
        passed = passed and result.get("inspect_returncode") == 0
        if "simulate_returncode" in result:
            passed = passed and result["simulate_returncode"] == 0
    return {
        "status": "passed" if passed else "failed",
        "passed": passed,
        "elapsed_ns": result.get("elapsed_ns"),
        "operation": result.get("operation"),
        "machine_independent": True,
    }


def script_main(case_dir: Path, mode: str) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workdir", type=Path, default=case_dir / ".benchmark-run")
    arguments = parser.parse_args()
    if mode == "generate":
        print(write_dataset(case_dir, arguments.workdir))
        return 0
    result = run_workload(case_dir, arguments.workdir)
    if mode == "run":
        write_json(arguments.workdir / "result.json", result)
        print(json.dumps(result, sort_keys=True))
        return 0 if result.get("returncode") == 0 else 2
    score = score_workload(case_dir, result)
    write_json(arguments.workdir / "score.json", score)
    print(json.dumps(score, sort_keys=True))
    return 0 if score["passed"] else 2
