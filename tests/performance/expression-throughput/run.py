#!/usr/bin/env python3
"""Run the expression-throughput performance contract."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from _support import execute

if __name__ == "__main__":
    execute(Path(__file__).resolve().parent)
