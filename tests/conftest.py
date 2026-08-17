"""Shared pytest configuration for the LawSynth system-verification suites.

Realizes architecture section 13.9 (cross-language and system verification) by
wiring the declarative case fixtures under ``tests/conformance``,
``tests/end-to-end``, and ``tests/cross-language`` into executable, asserting
pytest cases that drive the compiled ``lawsynth`` engine.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

_TESTS = Path(__file__).resolve().parent
_ROOT = _TESTS.parent

# Make the shared helpers and the pure-Python SDK importable regardless of the
# caller's PYTHONPATH, so ``python3 -m pytest tests`` works from a clean shell.
for path in (
    _TESTS,
    _TESTS / "end-to-end",
    _TESTS / "conformance",
    _ROOT / "python" / "lawsynth" / "src",
):
    text = str(path)
    if text not in sys.path:
        sys.path.insert(0, text)

import _engine  # noqa: E402


@pytest.fixture(scope="session")
def engine_binary() -> Path:
    """Ensure the compiled engine exists, or skip the depending case cleanly."""
    binary = _engine.ensure_engine()
    if binary is None:
        pytest.skip(
            "lawsynth CLI unavailable: build it with 'cargo build -p lawsynth-cli' "
            "(cargo missing or the offline build failed)"
        )
    return binary


def discover_cases(subdirectory: str) -> list[Path]:
    """Return every case directory (one holding a ``case.toml``) under a suite."""
    base = _TESTS / subdirectory
    return sorted(child for child in base.iterdir() if (child / "case.toml").is_file())
