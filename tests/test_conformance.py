"""Conformance suite: valid worlds must round-trip, ``bad-*`` must be rejected.

Each case under ``tests/conformance`` is executed against the real compiled
engine. Valid worlds are inspected and simulated; the negative fixtures
(``bad-expression``, ``bad-hash``, ``bad-schema``, ``bad-units``,
``unsafe-archive``) must be rejected with a non-zero exit and the documented
diagnostic. Cases documenting an unimplemented capability assert the honest
boundary rather than forcing a pass.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

from conftest import discover_cases

_CONFORMANCE = Path(__file__).resolve().parent / "conformance"
if str(_CONFORMANCE) not in sys.path:
    sys.path.insert(0, str(_CONFORMANCE))

import _conformance  # noqa: E402

_CASES = discover_cases("conformance")


@pytest.mark.parametrize("case_dir", _CASES, ids=[case.name for case in _CASES])
def test_conformance_case(case_dir: Path, engine_binary: Path) -> None:
    _conformance.run_case(case_dir)
