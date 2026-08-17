from __future__ import annotations

import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from _common import read_config, repository_root
from _engine import EngineUnavailable, ensure_binary
from _families import run_family_case, write_family_dataset


class BenchmarkExecutionTest(unittest.TestCase):
    """The promoted family case now executes and scores through the real CLI."""

    directory = Path(__file__).resolve().parent

    def test_dataset_generation_is_deterministic(self) -> None:
        config = read_config(self.directory)
        one = write_family_dataset(config, self.directory / ".benchmark-run").read_bytes()
        two = write_family_dataset(config, self.directory / ".benchmark-run").read_bytes()
        self.assertEqual(one, two)

    def test_executes_and_meets_ground_truth_minimum(self) -> None:
        root = repository_root(self.directory)
        try:
            binary = ensure_binary(root, allow_build=True)
        except EngineUnavailable as error:
            self.skipTest(str(error))
        result = run_family_case(self.directory, self.directory / ".benchmark-run", binary)
        self.assertEqual(result["status"], "passed", result)
        self.assertIsNotNone(result["signal_value"])
        self.assertGreaterEqual(result["signal_value"], result["expected_minimum"])
        self.assertEqual(result["discover_returncode"], 0)
        self.assertEqual(result["inspect_returncode"], 0)


if __name__ == "__main__":
    unittest.main()
