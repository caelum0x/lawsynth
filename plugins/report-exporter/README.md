# report-exporter

A LawSynth **report exporter plugin** written in Python. It renders a structured
report object into a human-readable artifact: deterministic Markdown, a
standalone HTML document, or canonical JSON.

The plugin runs as an isolated child process (`kind = "process"`) that the host
drives over the LawSynth plugin frame protocol; the core rendering logic lives
in `ReportExporter` and is pure, deterministic, and dependency-free (Python
standard library only).

## Formats

| `format` | Media type | Notes |
| --- | --- | --- |
| `markdown` (default) | `text/markdown; charset=utf-8` | Title, sections, and an optional fenced-JSON provenance block. |
| `html` | `text/html; charset=utf-8` | Self-contained HTML document with all text HTML-escaped. |
| `json` | `application/json` | The report object re-emitted with sorted keys. |

## Report shape

```python
report = {
    "title": "Discovery Report",
    "sections": [
        {"title": "Summary", "body": "Recovered a damped oscillator."},
        {"title": "Fit", "body": "MSE = 3.1e-6."},
    ],
    "provenance": {"run_id": "r-1", "seed": 42},  # optional
}
```

## Guarantees

- **Deterministic:** identical input yields byte-identical output (JSON keys are
  sorted; there is no timestamp or nondeterministic ordering).
- **Bounded:** output larger than `max_output_bytes` (default 16 MiB) is
  rejected with `ValueError`, matching the manifest's declared limit.
- **Escaped:** HTML output escapes all text; JSON output is produced with
  `json.dumps`.

## Layout

| Path | Purpose |
| --- | --- |
| `src/report_exporter/plugin.py` | `ReportExporter` rendering core. |
| `pyproject.toml` | Package metadata (src layout, stdlib-only). |
| `plugin.toml` | Manifest advertising `kind = "process"`, `artifact.write`. |
| `examples/basic.py` | Renders a sample report in all three formats. |
| `tests/test_plugin.py` | pytest suite for the rendering contract. |
| `docs/usage.md` | Request/response contract and host wiring. |

## Quick start

```bash
cd plugins/report-exporter
PYTHONPATH=src python3 examples/basic.py
PYTHONPATH=src python3 -m pytest      # or: pip install -e .[test] && pytest
```

## License

Apache-2.0. See [LICENSE](LICENSE).
