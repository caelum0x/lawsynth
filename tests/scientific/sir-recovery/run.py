#!/usr/bin/env python3
"""Execute the native deterministic SIR case."""
from pathlib import Path
from _support import assert_discovery_case

if __name__ == "__main__":
    assert_discovery_case(Path(__file__).resolve().parent)
