"""Render the benchmark results into a static ``index.html`` + ``results.json``."""

from __future__ import annotations

import html
import json
from dataclasses import dataclass
from pathlib import Path

from charts import status_bar_chart
from compare import Summary
from results import BenchmarkResult


@dataclass(frozen=True)
class Site:
    html: str
    results_json: str


def _results_payload(results: list[BenchmarkResult], summary: Summary) -> dict[str, object]:
    verdicts = {verdict.benchmark_id: verdict.status for verdict in summary.verdicts}
    return {
        "total": summary.total,
        "counts": dict(sorted(summary.counts.items())),
        "has_problems": summary.has_problems,
        "benchmarks": [
            {
                "id": result.benchmark_id,
                "title": result.title,
                "category": result.category,
                "capability": result.capability,
                "expected_status": result.expected_status,
                "observed_status": result.observed_status,
                "verdict": verdicts[result.benchmark_id],
            }
            for result in results
        ],
    }


def _render_rows(results: list[BenchmarkResult], summary: Summary) -> str:
    verdicts = {verdict.benchmark_id: verdict.status for verdict in summary.verdicts}
    rows = []
    for result in results:
        status = verdicts[result.benchmark_id]
        rows.append(
            "<tr>"
            f"<td>{html.escape(result.benchmark_id)}</td>"
            f"<td>{html.escape(result.title)}</td>"
            f"<td>{html.escape(result.capability)}</td>"
            f"<td>{html.escape(result.expected_status)}</td>"
            f"<td>{html.escape(result.observed_status or '-')}</td>"
            f'<td class="verdict {status}">{status}</td>'
            "</tr>"
        )
    return "\n".join(rows)


def render_site(results: list[BenchmarkResult], summary: Summary) -> Site:
    chart = status_bar_chart(summary)
    rows = _render_rows(results, summary)
    counts = ", ".join(f"{status}: {count}" for status, count in sorted(summary.counts.items()))
    document = f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>LawSynth Benchmark Results</title>
<style>
body {{ font-family: sans-serif; margin: 2rem; color: #212529; }}
table {{ border-collapse: collapse; width: 100%; margin-top: 1rem; }}
th, td {{ text-align: left; padding: 0.4rem 0.6rem; border-bottom: 1px solid #dee2e6; }}
.verdict {{ font-weight: 600; }}
.pass {{ color: #2a9d8f; }} .fail, .regression {{ color: #e63946; }}
.pending {{ color: #6c757d; }} .capability-boundary {{ color: #457b9d; }}
</style>
</head>
<body>
<h1>LawSynth Benchmark Results</h1>
<p>{summary.total} benchmarks &mdash; {html.escape(counts)}</p>
{chart}
<table>
<thead><tr><th>Benchmark</th><th>Title</th><th>Capability</th>
<th>Expected</th><th>Observed</th><th>Verdict</th></tr></thead>
<tbody>
{rows}
</tbody>
</table>
</body>
</html>
"""
    payload = json.dumps(_results_payload(results, summary), indent=2, sort_keys=True)
    return Site(html=document, results_json=payload)


def write_site(site: Site, out_dir: Path) -> None:
    out_dir = Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "index.html").write_text(site.html, encoding="utf-8")
    (out_dir / "results.json").write_text(site.results_json + "\n", encoding="utf-8")
