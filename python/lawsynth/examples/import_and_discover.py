#!/usr/bin/env python3
"""Import observations from a real connector, then discover their laws.

This proves the product loop ``source -> Study -> discover -> explain`` runs end
to end against the ``lawsynth_connectors`` library — not just a local file read.

Run it from the repository root::

    PYTHONPATH="python/lawsynth/src:python/lawsynth-connectors/src" \
        python3 python/lawsynth/examples/import_and_discover.py

It (1) writes a small deterministic CSV, (2) loads it through the *filesystem*
connector via ``lawsynth.load_source`` — coercing the connector's string cells
into floats at the SDK boundary — discovers the governing laws and explains
them; then (3) serves the same CSV over a local loopback HTTP server and repeats
the import through the *http* connector with ``allow_private_network=True``.

Everything is deterministic and offline (loopback only).
"""

from __future__ import annotations

import csv
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import lawsynth


def _write_linear_system_csv(path: Path) -> None:
    """A stable 2-D linear system integrated with deterministic RK4.

    dx/dt = -0.5*x + 1.0*y
    dy/dt = -1.0*x - 0.5*y      (a decaying rotation)

    Written with string-formatted cells so the connector hands the SDK strings,
    exercising numeric coercion at the ``load_source`` boundary.
    """
    a, b, c, d = -0.5, 1.0, -1.0, -0.5
    dt, steps = 0.02, 700
    x, y = 1.0, 0.5
    rows: list[tuple[float, float, float]] = []
    for i in range(steps):
        rows.append((i * dt, x, y))

        def deriv(x_: float, y_: float) -> tuple[float, float]:
            return a * x_ + b * y_, c * x_ + d * y_

        k1x, k1y = deriv(x, y)
        k2x, k2y = deriv(x + 0.5 * dt * k1x, y + 0.5 * dt * k1y)
        k3x, k3y = deriv(x + 0.5 * dt * k2x, y + 0.5 * dt * k2y)
        k4x, k4y = deriv(x + dt * k3x, y + dt * k3y)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        y += dt / 6 * (k1y + 2 * k2y + 2 * k3y + k4y)

    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["t", "x", "y"])
        writer.writerows((f"{t:.6f}", f"{xv:.10f}", f"{yv:.10f}") for t, xv, yv in rows)


def _discover_and_report(study: lawsynth.Study, label: str) -> None:
    discovery = study.discover(threshold=0.05)
    print(f"[{label}] loaded {len(study.dataset.time)} samples via connector; "
          f"columns={sorted(study.dataset.columns)}")
    print(f"[{label}] discovered equations:")
    for target, expression in discovery.equations.items():
        print(f"    d{target}/dt = {expression}")
    explanation = discovery.explain()
    print(f"[{label}] fit quality:")
    for state, metrics in sorted(explanation.fit.items()):
        print(f"    {state}: R2={metrics['r_squared']:.4f}  RMSE={metrics['rmse']:.4g}")
    print()


def _serve_loopback(directory: Path) -> tuple[ThreadingHTTPServer, str]:
    """Serve ``directory`` on 127.0.0.1 and return (server, base_url)."""

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self) -> None:  # noqa: N802 - http.server API
            target = directory / self.path.lstrip("/")
            if not target.is_file():
                self.send_error(404)
                return
            payload = target.read_bytes()
            self.send_response(200)
            self.send_header("Content-Type", "text/csv; charset=utf-8")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, *_args: object) -> None:
            return  # keep the demo output clean

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    host, port = server.server_address[:2]
    return server, f"http://{host}:{port}"


def main() -> None:
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth_import_"))
    csv_path = workdir / "observations.csv"
    _write_linear_system_csv(csv_path)
    print(f"synthetic observations written to {csv_path}\n")

    # 1. Filesystem connector: the primary, always-available import path.
    #    load_source() reads batched string records and coerces them to floats.
    dataset = lawsynth.load_source(
        "filesystem",
        "observations.csv",
        time="t",
        state=["x", "y"],
        options={"root": str(workdir)},
    )
    fs_study = lawsynth.Study.from_dataset(dataset, state=["x", "y"], name="filesystem_import")
    _discover_and_report(fs_study, "filesystem")

    # 1b. The same import as a one-liner via the first-class Study entry point.
    oneliner = lawsynth.Study.from_source(
        "filesystem", "observations.csv",
        time="t", state=["x", "y"], options={"root": str(workdir)},
    )
    print("[from_source] plain-language explanation:")
    print("-" * 68)
    print(oneliner.discover().explain().to_text())
    print("-" * 68, "\n")

    # 2. HTTP connector against a local loopback server (offline, deterministic).
    server, base_url = _serve_loopback(workdir)
    try:
        http_study = lawsynth.Study.from_source(
            "http",
            f"{base_url}/observations.csv",
            time="t",
            state=["x", "y"],
            options={"allow_private_network": True},
            name="http_import",
        )
        _discover_and_report(http_study, "http")
    finally:
        server.shutdown()

    # 3. Optional-dependency connectors degrade cleanly (no driver installed).
    try:
        lawsynth.load_source("duckdb", "unused", time="t", state=["x"],
                             options={"query": "SELECT 1"})
    except lawsynth.SourceError as error:
        print(f"[graceful] duckdb connector without its driver -> SourceError: "
              f"{str(error)[:80]}...")

    print("\nsource -> study -> discovery loop verified across filesystem + http.")


if __name__ == "__main__":
    main()
