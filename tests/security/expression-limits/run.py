#!/usr/bin/env python3
"""Run the expression-limits executable LawSynth boundary case."""
from __future__ import annotations
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from _support import assert_case
if __name__=="__main__": assert_case(Path(__file__).resolve().parent)
