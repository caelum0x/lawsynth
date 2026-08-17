"""Aggregate case outcomes into a deterministic pass/fail report."""

from __future__ import annotations

import json
from dataclasses import dataclass

from compare import Comparison
from execute import ExecutionResult


@dataclass(frozen=True)
class CaseOutcome:
    case_id: str
    passed: bool
    returncode: int
    differences: tuple[str, ...]
    stderr: str

    @classmethod
    def build(
        cls,
        case_id: str,
        execution: ExecutionResult,
        comparison: Comparison,
    ) -> "CaseOutcome":
        passed = execution.succeeded and comparison.passed
        return cls(
            case_id=case_id,
            passed=passed,
            returncode=execution.returncode,
            differences=comparison.differences,
            stderr=execution.stderr.strip(),
        )


@dataclass(frozen=True)
class Report:
    outcomes: tuple[CaseOutcome, ...]

    @property
    def total(self) -> int:
        return len(self.outcomes)

    @property
    def passed(self) -> int:
        return sum(1 for outcome in self.outcomes if outcome.passed)

    @property
    def failed(self) -> int:
        return self.total - self.passed

    @property
    def ok(self) -> bool:
        return self.failed == 0


def build_report(outcomes: list[CaseOutcome]) -> Report:
    return Report(outcomes=tuple(sorted(outcomes, key=lambda outcome: outcome.case_id)))


def to_dict(report: Report) -> dict[str, object]:
    return {
        "total": report.total,
        "passed": report.passed,
        "failed": report.failed,
        "ok": report.ok,
        "cases": [
            {
                "case_id": outcome.case_id,
                "passed": outcome.passed,
                "returncode": outcome.returncode,
                "differences": list(outcome.differences),
            }
            for outcome in report.outcomes
        ],
    }


def to_json(report: Report) -> str:
    return json.dumps(to_dict(report), indent=2, sort_keys=True)


def to_text(report: Report) -> str:
    lines: list[str] = []
    for outcome in report.outcomes:
        status = "PASS" if outcome.passed else "FAIL"
        lines.append(f"[{status}] {outcome.case_id}")
        if not outcome.passed:
            if outcome.returncode != 0:
                lines.append(f"    exit code: {outcome.returncode}")
            for difference in outcome.differences:
                lines.append(f"    diff: {difference}")
            if outcome.stderr:
                first = outcome.stderr.splitlines()[0]
                lines.append(f"    stderr: {first}")
    lines.append("")
    lines.append(f"{report.passed}/{report.total} passed")
    return "\n".join(lines)
