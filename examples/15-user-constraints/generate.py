#!/usr/bin/env python3
"""generate workflow for User constraints."""
from pathlib import Path
import sys

EXAMPLES = Path(__file__).resolve().parents[1]
if str(EXAMPLES) not in sys.path:
    sys.path.insert(0, str(EXAMPLES))
from _workflow import cli

if __name__ == "__main__":
    raise SystemExit(cli("generate", Path(__file__).parent))
