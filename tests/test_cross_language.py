"""Cross-language suite: verify artifacts cross the Python/Rust wire boundary.

``bundle-roundtrip`` and ``schema-roundtrip`` assert a language-neutral binary
world produced on one side is read back identically by the Rust engine.
``rust-python`` exercises a portable world from Python-shaped data through the
CLI. ``python-rust`` probes the real Python ``lawsynth`` package against the
CLI and honestly reports ``available``/``unavailable`` for the native leg.
``typescript-rust`` records the honest boundary that no TypeScript binding
ships in this runtime.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

from conftest import discover_cases

_END_TO_END = Path(__file__).resolve().parent / "end-to-end"
if str(_END_TO_END) not in sys.path:
    sys.path.insert(0, str(_END_TO_END))

import _workflow  # noqa: E402

_CASES = discover_cases("cross-language")


@pytest.mark.parametrize("case_dir", _CASES, ids=[case.name for case in _CASES])
def test_cross_language_case(case_dir: Path, engine_binary: Path) -> None:
    _workflow.run_case(case_dir)
