"""Execute a conformance case and capture its observable result.

Each case ships an executable runner (``python3 run.py`` by convention) that
builds fixtures and drives the native LawSynth CLI.  This module runs that
runner as a subprocess in the case directory, captures the outcome, and extracts
any trailing JSON object the runner prints on stdout (the observed result).

For deterministic, offline testing the executor accepts an injectable ``runner``
callable, so a case can be exercised without spawning a real subprocess.
"""

from __future__ import annotations

import json
import subprocess
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path

from discover import Case

# A runner takes (command, cwd, timeout) and returns (returncode, stdout, stderr).
Runner = Callable[[list[str], Path, float], tuple[int, str, str]]


@dataclass(frozen=True)
class ExecutionResult:
    case_id: str
    returncode: int
    stdout: str
    stderr: str
    observed: dict[str, object] | None
    timed_out: bool = False

    @property
    def succeeded(self) -> bool:
        return self.returncode == 0 and not self.timed_out


def _subprocess_runner(command: list[str], cwd: Path, timeout: float) -> tuple[int, str, str]:
    completed = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )
    return completed.returncode, completed.stdout, completed.stderr


def extract_json(stdout: str) -> dict[str, object] | None:
    """Return the last JSON object printed on its own line, if any.

    Runners commonly emit human-readable lines followed by a final JSON summary.
    Scanning from the end keeps the most authoritative object.
    """
    for line in reversed(stdout.splitlines()):
        candidate = line.strip()
        if candidate.startswith("{") and candidate.endswith("}"):
            try:
                parsed = json.loads(candidate)
            except json.JSONDecodeError:
                continue
            if isinstance(parsed, dict):
                return parsed
    return None


def execute_case(
    case: Case,
    runner: Runner | None = None,
    timeout: float = 300.0,
) -> ExecutionResult:
    """Run a case's runner and capture its result."""
    runner = runner or _subprocess_runner
    try:
        returncode, stdout, stderr = runner(list(case.runner), case.directory, timeout)
    except subprocess.TimeoutExpired:
        return ExecutionResult(
            case_id=case.case_id,
            returncode=124,
            stdout="",
            stderr=f"timed out after {timeout}s",
            observed=None,
            timed_out=True,
        )
    return ExecutionResult(
        case_id=case.case_id,
        returncode=returncode,
        stdout=stdout,
        stderr=stderr,
        observed=extract_json(stdout),
    )
