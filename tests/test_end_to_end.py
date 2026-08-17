"""End-to-end suite: exercise each case through the real CLI via ``_workflow``.

``discover`` cases must produce a serialized world, ``simulate`` cases must
produce the expected trajectory shape, ``native-optional`` cases probe the
optional Python native surface, and ``boundary`` cases assert the honest limit
of an unimplemented capability instead of forcing a pass.
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

_CASES = discover_cases("end-to-end")


@pytest.mark.parametrize("case_dir", _CASES, ids=[case.name for case in _CASES])
def test_end_to_end_case(case_dir: Path, engine_binary: Path) -> None:
    _workflow.run_case(case_dir)
