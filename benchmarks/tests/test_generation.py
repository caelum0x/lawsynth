"""Determinism tests for benchmark dataset generation."""

from __future__ import annotations

import tempfile
from pathlib import Path

import pytest

from _common import write_dataset


@pytest.mark.parametrize("case", ["dynamics/ode-chaotic", "dynamics/ode-small"])
def test_dataset_is_byte_reproducible(benchmarks_dir: Path, case: str) -> None:
    case_dir = benchmarks_dir / case
    with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
        one = write_dataset(case_dir, Path(first)).read_bytes()
        two = write_dataset(case_dir, Path(second)).read_bytes()
    assert one == two
    # A discovery dataset needs a header plus several observations.
    assert len(one.splitlines()) > 3


def test_lorenz_dataset_has_three_states(benchmarks_dir: Path) -> None:
    case_dir = benchmarks_dir / "dynamics/ode-chaotic"
    with tempfile.TemporaryDirectory() as workdir:
        header = write_dataset(case_dir, Path(workdir)).read_text().splitlines()[0]
    assert header == "time,x,y,z"
