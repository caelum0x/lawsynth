from pathlib import Path
import sys, tempfile, tomllib, unittest
ROOT = Path(__file__).resolve().parents[2]; sys.path.insert(0, str(ROOT))
from _common import write_dataset
CASE = Path(__file__).resolve().parent
class SimulationTest(unittest.TestCase):
    def test_deterministic_native_workload_fixture(self):
        with (CASE / "benchmark.toml").open("rb") as f: config = tomllib.load(f)
        self.assertEqual(config["id"], "performance/simulation")
        with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
            self.assertEqual(write_dataset(CASE, Path(a)).read_bytes(), write_dataset(CASE, Path(b)).read_bytes())
if __name__ == "__main__": unittest.main()
