"""Shared pytest fixtures for the benchmark runner tests.

The tests exercise the *real* runner against the *real* compiled CLI.  If the
binary is not present, the fixture builds it once (offline); if that build is
impossible in the environment it skips with an explicit message rather than
silently passing.
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

BENCHMARKS = Path(__file__).resolve().parents[1]
ROOT = BENCHMARKS.parent
SDK_SRC = ROOT / "python" / "lawsynth" / "src"

for candidate in (BENCHMARKS, SDK_SRC):
    if str(candidate) not in sys.path:
        sys.path.insert(0, str(candidate))

from _engine import EngineUnavailable, ensure_binary  # noqa: E402


@pytest.fixture(scope="session")
def benchmarks_dir() -> Path:
    return BENCHMARKS


@pytest.fixture(scope="session")
def repo_root() -> Path:
    return ROOT


@pytest.fixture(scope="session")
def cli_binary() -> Path:
    """Return the compiled CLI binary, building it once or skipping cleanly."""
    try:
        return ensure_binary(ROOT, allow_build=True)
    except EngineUnavailable as error:
        pytest.skip(
            "lawsynth CLI binary unavailable and offline build failed; "
            f"run `cargo build -p lawsynth-cli` first ({error})"
        )
