"""Compare an executed case's observed result against its expected outcome.

Comparison is deterministic and tolerant of floating-point representation:
numbers compare within a relative + absolute tolerance, nested structures are
compared recursively, and every mismatch is reported (not just the first).
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field

ABS_TOLERANCE = 1e-9
REL_TOLERANCE = 1e-9


@dataclass(frozen=True)
class Comparison:
    passed: bool
    differences: tuple[str, ...] = field(default_factory=tuple)


def _numbers_close(left: float, right: float) -> bool:
    if math.isnan(left) and math.isnan(right):
        return True
    return math.isclose(left, right, rel_tol=REL_TOLERANCE, abs_tol=ABS_TOLERANCE)


def _diff(expected: object, actual: object, path: str, out: list[str]) -> None:
    if isinstance(expected, bool) or isinstance(actual, bool):
        if expected != actual:
            out.append(f"{path}: expected {expected!r}, got {actual!r}")
        return
    if isinstance(expected, (int, float)) and isinstance(actual, (int, float)):
        if not _numbers_close(float(expected), float(actual)):
            out.append(f"{path}: expected {expected}, got {actual}")
        return
    if isinstance(expected, dict) and isinstance(actual, dict):
        for key in expected:
            if key not in actual:
                out.append(f"{path}.{key}: missing")
            else:
                _diff(expected[key], actual[key], f"{path}.{key}", out)
        return
    if isinstance(expected, list) and isinstance(actual, list):
        if len(expected) != len(actual):
            out.append(f"{path}: length {len(expected)} != {len(actual)}")
            return
        for index, (exp_item, act_item) in enumerate(zip(expected, actual)):
            _diff(exp_item, act_item, f"{path}[{index}]", out)
        return
    if expected != actual:
        out.append(f"{path}: expected {expected!r}, got {actual!r}")


def compare(expected: dict[str, object], actual: dict[str, object] | None) -> Comparison:
    """Compare ``expected`` against ``actual`` restricted to the expected keys.

    Only keys present in ``expected`` are checked; a runner may emit additional
    diagnostic fields.  A missing observed object with a non-empty expectation is
    a failure.
    """
    if not expected:
        # Nothing declared to compare; treat as vacuously satisfied.
        return Comparison(passed=True)
    if actual is None:
        return Comparison(passed=False, differences=("no observed result to compare",))

    differences: list[str] = []
    for key, expected_value in expected.items():
        if key not in actual:
            differences.append(f"{key}: missing")
        else:
            _diff(expected_value, actual[key], key, differences)
    return Comparison(passed=not differences, differences=tuple(differences))
