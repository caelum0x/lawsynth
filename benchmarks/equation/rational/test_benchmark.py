from __future__ import annotations

import sys
from pathlib import Path

BENCHMARKS = Path(__file__).resolve().parents[2]
if str(BENCHMARKS) not in sys.path:
    sys.path.insert(0, str(BENCHMARKS))

import json
import subprocess
import tempfile
import tomllib
import unittest

from _common import write_dataset

CASE = Path(__file__).resolve().parent


class RationalBenchmarkTest(unittest.TestCase):
    def test_configuration_and_generator_are_deterministic(self) -> None:
        with (CASE / "benchmark.toml").open("rb") as handle:
            config = tomllib.load(handle)
        self.assertEqual(config["id"], "equation/rational")
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            one = write_dataset(CASE, Path(first)).read_bytes()
            two = write_dataset(CASE, Path(second)).read_bytes()
        self.assertEqual(one, two)
        self.assertGreater(len(one.splitlines()), 3)

    def test_runner_reports_declared_native_capability(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            outcome = subprocess.run(
                [sys.executable, str(CASE / "run.py"), "--workdir", temporary],
                cwd=CASE, text=True, capture_output=True, check=False,
            )
            result = json.loads((Path(temporary) / "result.json").read_text(encoding="utf-8"))
        if "boundary" == "supported":
            self.assertEqual(outcome.returncode, 0, outcome.stderr)
            self.assertEqual(result["returncode"], 0, result.get("stderr"))
            self.assertEqual(result["inspect_returncode"], 0, result.get("inspect_stderr"))
        else:
            self.assertEqual(outcome.returncode, 2)
            self.assertEqual(result["status"], "capability-boundary")
            self.assertIn("reason", result)


if __name__ == "__main__":
    unittest.main()
