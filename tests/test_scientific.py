"""Scientific suite: execute each case's declared ``run.py`` as a real test.

Each case under ``tests/scientific`` ships a standalone, directly-executable
``run.py`` (see the per-case ``README.md`` and ``case.toml``'s
``[execution] runner = "python3 run.py"``) that drives the compiled native
engine against a declarative fixture and asserts the result in
``_support.py``. Pointing a bare ``pytest`` invocation at the
``tests/scientific`` directory collects zero tests, because none of these
files match pytest's ``test_*.py`` discovery pattern. This module runs each
case's ``run.py`` exactly as its own ``if __name__ == "__main__":`` guard
intends, parametrized one case per test, so the suite is both collectible by
pytest and faithful to the fixtures' own declared execution contract.
"""

from __future__ import annotations

import runpy
from pathlib import Path

import pytest

from conftest import discover_cases

_CASES = discover_cases("scientific")


@pytest.mark.parametrize("case_dir", _CASES, ids=[case.name for case in _CASES])
def test_scientific_case(case_dir: Path) -> None:
    runpy.run_path(str(case_dir / "run.py"), run_name="__main__")
