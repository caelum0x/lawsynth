#!/usr/bin/env python3
"""Exercise the CSV path and record its explicit unit-metadata boundary."""
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from _support import assert_unit_boundary

if __name__ == "__main__":
    assert_unit_boundary(Path(__file__).resolve().parent)
