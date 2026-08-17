"""Discover cross-language conformance cases on disk.

A conformance case is a directory containing a ``case.toml`` descriptor.  The
LawSynth repository stores these under ``tests/conformance`` and
``tests/cross-language``.  A well-formed case also carries an ``input.json``
fixture contract and an ``expected.json`` observable outcome, plus an executable
runner (by convention ``run.py``) named by ``case.toml``.

See ``tests/conformance/*/case.toml`` for the on-disk shape.
"""

from __future__ import annotations

import json
import tomllib
from dataclasses import dataclass
from pathlib import Path

CASE_DESCRIPTOR = "case.toml"


class DiscoveryError(ValueError):
    """Raised when a case directory does not satisfy the descriptor contract."""


@dataclass(frozen=True)
class Case:
    """A single discovered conformance case."""

    case_id: str
    directory: Path
    mode: str
    runner: list[str]
    expected: dict[str, object]

    @property
    def name(self) -> str:
        return self.case_id


def _load_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def load_case(directory: Path) -> Case:
    """Load a single case from its directory."""
    directory = Path(directory)
    descriptor = directory / CASE_DESCRIPTOR
    if not descriptor.is_file():
        raise DiscoveryError(f"missing {CASE_DESCRIPTOR} in {directory}")

    config = _load_toml(descriptor)
    case_section = config.get("case")
    if not isinstance(case_section, dict) or "id" not in case_section:
        raise DiscoveryError(f"{descriptor} has no [case] id")

    case_id = str(case_section["id"])
    mode = str(case_section.get("mode", case_section.get("workflow", "valid")))

    execution = config.get("execution")
    runner_command = "python3 run.py"
    if isinstance(execution, dict) and "runner" in execution:
        runner_command = str(execution["runner"])
    runner = runner_command.split()

    expected: dict[str, object] = {}
    expected_path = directory / "expected.json"
    if expected_path.is_file():
        loaded = json.loads(expected_path.read_text(encoding="utf-8"))
        if isinstance(loaded, dict):
            expected = loaded

    return Case(
        case_id=case_id,
        directory=directory,
        mode=mode,
        runner=runner,
        expected=expected,
    )


def discover_cases(root: Path, name_filter: str | None = None) -> list[Case]:
    """Discover every case under ``root``, sorted by ``case_id`` for determinism.

    A directory is a case when it directly contains ``case.toml``.
    ``name_filter`` keeps only cases whose id contains the given substring.
    """
    root = Path(root)
    if not root.is_dir():
        raise DiscoveryError(f"not a directory: {root}")

    cases: list[Case] = []
    for descriptor in root.rglob(CASE_DESCRIPTOR):
        case = load_case(descriptor.parent)
        if name_filter is None or name_filter in case.case_id:
            cases.append(case)
    cases.sort(key=lambda case: case.case_id)
    return cases
