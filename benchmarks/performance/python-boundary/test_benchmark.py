from pathlib import Path
import json, subprocess, sys, tempfile, tomllib, unittest
ROOT = Path(__file__).resolve().parents[2]; sys.path.insert(0, str(ROOT))
CASE = Path(__file__).resolve().parent
class PythonBoundaryTest(unittest.TestCase):
    def test_public_sdk_workload_executes(self):
        with (CASE / "benchmark.toml").open("rb") as f: self.assertEqual(tomllib.load(f)["id"], "performance/python-boundary")
        with tempfile.TemporaryDirectory() as work:
            result = subprocess.run([sys.executable, str(CASE / "run.py"), "--workdir", work], text=True, capture_output=True, check=False)
            payload = json.loads((Path(work) / "result.json").read_text())
        self.assertEqual(result.returncode, 0, result.stderr); self.assertEqual(payload["rows"], 2000)
if __name__ == "__main__": unittest.main()
