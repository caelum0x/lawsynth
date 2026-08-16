from __future__ import annotations

import sys
from pathlib import Path

BENCHMARKS = Path(__file__).resolve().parents[2]
if str(BENCHMARKS) not in sys.path:
    sys.path.insert(0, str(BENCHMARKS))

from _common import script_main

CASE = Path(__file__).resolve().parent

if __name__ == "__main__":
    raise SystemExit(script_main(CASE, "score"))
