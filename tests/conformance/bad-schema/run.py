#!/usr/bin/env python3
"""Standalone entry for this conformance case; the real logic lives in
``tests/conformance/_conformance.py`` and drives the compiled LawSynth CLI."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from _conformance import run_case

if __name__ == "__main__":
    run_case(Path(__file__).resolve().parent)
