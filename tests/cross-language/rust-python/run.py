#!/usr/bin/env python3
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "end-to-end"))
from _workflow import run_case
if __name__ == "__main__": run_case(Path(__file__).resolve().parent)
