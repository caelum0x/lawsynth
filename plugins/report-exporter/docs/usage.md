# Usage — report-exporter

## The rendering API

`ReportExporter.invoke` takes a request mapping and returns an artifact mapping:

```python
from report_exporter.plugin import ReportExporter

exporter = ReportExporter(max_output_bytes=16 * 1024 * 1024)
artifact = exporter.invoke({
    "report": {
        "title": "Discovery Report",
        "sections": [
            {"title": "Summary", "body": "Recovered dx/dt = v."},
        ],
        "provenance": {"run_id": "r-1", "seed": 42},
    },
    "format": "markdown",  # "markdown" | "html" | "json"
})

artifact["content"]     # the rendered string
artifact["media_type"]  # e.g. "text/markdown; charset=utf-8"
artifact["bytes"]       # UTF-8 byte length of content
```

### Request fields

| Field | Required | Meaning |
| --- | --- | --- |
| `report` | yes | A mapping with `title`, `sections`, and optional `provenance`. |
| `format` | no | One of `markdown` (default), `html`, `json`. |

Each `section` is a mapping with `title` and `body` strings.

### Errors

| Condition | Exception |
| --- | --- |
| `report` is not a mapping | `TypeError` |
| `report["sections"]` is not a sequence | `TypeError` |
| `format` is unknown | `ValueError` |
| rendered output exceeds `max_output_bytes` | `ValueError` |

## Running as a process worker

The manifest advertises `kind = "process"`. A worker binary wraps
`ReportExporter` and speaks the LawSynth frame protocol on stdin/stdout:

```text
loop:
    read a length-delimited Frame from stdin
    Hello    -> reply Hello (confirm protocol version 1)
    Request  -> decode {report, format}, call exporter.invoke, reply Response/Error
    Shutdown -> exit
```

Frame framing (4-byte length, 2-byte version, kind, reserved zero byte, 8-byte
request id, payload) is defined by `lawsynth-plugin-api`; the payload encoding
inside a frame is agreed between host and worker. The host owns process
spawning, sandboxing, CPU/memory limits, and timeouts, and enforces the declared
`max_output_bytes`.

## Host integration

1. Discover and validate the `plugin.manifest` (the content of `plugin.toml`).
2. Confirm the `artifact.write` capability is granted.
3. Spawn the worker; drive the lifecycle to `Running`.
4. Dispatch report requests; persist the returned `content` to the artifact
   store.
