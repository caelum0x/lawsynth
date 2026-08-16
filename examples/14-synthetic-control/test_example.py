"""Executable contract for Synthetic control."""
from pathlib import Path
import sys

EXAMPLES = Path(__file__).resolve().parents[1]
if str(EXAMPLES) not in sys.path:
    sys.path.insert(0, str(EXAMPLES))
from _workflow import verify_example


def test_14_synthetic_control_workflow() -> None:
    verify_example(Path(__file__).parent)
