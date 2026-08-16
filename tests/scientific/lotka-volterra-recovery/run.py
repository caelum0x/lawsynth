#!/usr/bin/env python3
"""Execute the native Lotka–Volterra scientific case."""
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from _support import assert_discovery_case

if __name__ == "__main__":
    assert_discovery_case(Path(__file__).resolve().parent)
