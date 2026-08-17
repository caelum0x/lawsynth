"""Evaluate a dependency inventory against a policy and format the result."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass

from policy import Policy
from scan import Dependency


@dataclass(frozen=True)
class Finding:
    name: str
    version: str
    license: str | None
    status: str  # "allowed" | "denied" | "unknown"


@dataclass(frozen=True)
class Result:
    findings: tuple[Finding, ...]

    @property
    def denied(self) -> tuple[Finding, ...]:
        return tuple(f for f in self.findings if f.status == "denied")

    @property
    def unknown(self) -> tuple[Finding, ...]:
        return tuple(f for f in self.findings if f.status == "unknown")

    @property
    def ok(self) -> bool:
        return not self.denied and not self.unknown


def evaluate(dependencies: list[Dependency], policy: Policy) -> Result:
    """Classify each dependency as allowed, denied, or unknown (missing license)."""
    findings: list[Finding] = []
    for dependency in dependencies:
        if dependency.license is None:
            status = "unknown"
        elif policy.permits_expression(dependency.license):
            status = "allowed"
        else:
            status = "denied"
        findings.append(
            Finding(dependency.name, dependency.version, dependency.license, status)
        )
    return Result(findings=tuple(findings))


def format_text(result: Result) -> str:
    lines = []
    counts = {"allowed": 0, "denied": 0, "unknown": 0}
    for finding in result.findings:
        counts[finding.status] += 1
    lines.append(
        f"scanned {len(result.findings)} dependencies: "
        f"{counts['allowed']} allowed, {counts['denied']} denied, "
        f"{counts['unknown']} unknown"
    )
    for finding in result.denied:
        lines.append(f"  DENIED  {finding.name} {finding.version} ({finding.license})")
    for finding in result.unknown:
        lines.append(f"  UNKNOWN {finding.name} {finding.version} (no license metadata)")
    lines.append("OK" if result.ok else "FAIL")
    return "\n".join(lines) + "\n"


def format_json(result: Result) -> str:
    payload = {
        "ok": result.ok,
        "denied": [f.name for f in result.denied],
        "unknown": [f.name for f in result.unknown],
        "findings": [asdict(f) for f in result.findings],
    }
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"
