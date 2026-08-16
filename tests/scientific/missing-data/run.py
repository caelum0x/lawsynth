#!/usr/bin/env python3
"""Validate the native missing-observation capability boundary."""
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from _support import assert_missing_data_boundary

if __name__ == "__main__":
    assert_missing_data_boundary(Path(__file__).resolve().parent)
