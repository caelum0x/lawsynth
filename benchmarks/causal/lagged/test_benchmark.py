from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from _capability_contract import execute, score, write_generated


class BenchmarkContractTest(unittest.TestCase):
    directory = Path(__file__).resolve().parent

    def test_capability_boundary_is_explicit(self) -> None:
        result = execute(self.directory)
        self.assertEqual(result["status"], "capability_boundary")
        self.assertEqual(result["benchmark"], "causal/lagged")
        self.assertIn("causal identification and effect estimation", result["error"])

    def test_generated_dataset_is_deterministic_and_scoreable(self) -> None:
        first = write_generated(self.directory)
        first_payload = json.loads(first.read_text(encoding="utf-8"))
        second = write_generated(self.directory)
        self.assertEqual(first_payload, json.loads(second.read_text(encoding="utf-8")))
        self.assertEqual(score(self.directory)["status"], "capability_boundary")


if __name__ == "__main__":
    unittest.main()
