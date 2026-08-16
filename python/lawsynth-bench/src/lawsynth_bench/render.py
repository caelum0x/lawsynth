"""Human-readable Markdown rendering of structured reports."""
from typing import Mapping

def markdown(report: Mapping[str, object]) -> str:
    lines = ["# LawSynth benchmark report", "", f"Observations: {report['observation_count']}",
             f"Regressions: {report['regression_count']}", "", "| Problem | Implementation | Metric | Mean |", "|---|---|---|---:|"]
    for item in report.get("summaries", []):
        summary = dict(item)
        lines.append(f"| {summary['problem']} | {summary['implementation']} | {summary['metric']} | {summary['mean']:.6g} {summary['unit']} |")
    changes = report.get("changes", [])
    if changes:
        lines.extend(["", "## Comparison", "", "| Metric group | Ratio | Regression |", "|---|---:|---|"])
        for change in changes:
            value = dict(change); lines.append(f"| {' / '.join(value['key'])} | {value['ratio']:.4g} | {str(value['regression']).lower()} |")
    return "\n".join(lines) + "\n"
