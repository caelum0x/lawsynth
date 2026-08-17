"""Render a small LawSynth report to Markdown, HTML, and JSON.

Run with the package on the path::

    PYTHONPATH=src python3 examples/basic.py
"""

from __future__ import annotations

from report_exporter.plugin import ReportExporter


def main() -> None:
    report = {
        "title": "Damped Oscillator Discovery",
        "sections": [
            {
                "title": "Summary",
                "body": "Recovered dx/dt = v and dv/dt = -x - 0.1 v from data.",
            },
            {
                "title": "Fit quality",
                "body": "Mean squared error: 3.1e-6 over 500 samples.",
            },
        ],
        "provenance": {"run_id": "2026-08-17T00:00:00Z", "seed": 42},
    }

    exporter = ReportExporter()
    for fmt in ("markdown", "html", "json"):
        artifact = exporter.invoke({"report": report, "format": fmt})
        print(f"--- {artifact['media_type']} ({artifact['bytes']} bytes) ---")
        print(artifact["content"])
        print()


if __name__ == "__main__":
    main()
