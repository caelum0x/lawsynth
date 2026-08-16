#!/usr/bin/env python3
"""Run the switched-regime boundary case through native discovery."""
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from _support import assert_regime_boundary

if __name__ == "__main__":
    assert_regime_boundary(Path(__file__).resolve().parent)
