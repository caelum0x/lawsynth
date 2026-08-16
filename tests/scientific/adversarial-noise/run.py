#!/usr/bin/env python3
"""Execute deterministic finite-noise discovery using the native CLI."""
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from _support import assert_noise_case

if __name__ == "__main__":
    assert_noise_case(Path(__file__).resolve().parent)
