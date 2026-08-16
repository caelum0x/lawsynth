"""Executable contract for Lotka–Volterra predator prey."""
from pathlib import Path
import sys

EXAMPLES = Path(__file__).resolve().parents[1]
if str(EXAMPLES) not in sys.path:
    sys.path.insert(0, str(EXAMPLES))
from _workflow import verify_example


def test_02_lotka_volterra_workflow() -> None:
    verify_example(Path(__file__).parent)
