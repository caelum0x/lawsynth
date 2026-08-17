#!/usr/bin/env python3
"""Drive the LawSynth API's discovery-as-a-service loop from Python --- offline.

This stands up the real ``lawsynth_api`` WSGI application in-process (a temp
SQLite metadata store, a temp object root, and a single bearer token), then uses
:class:`lawsynth.Client` to run the full *remote* product loop against it without
opening a socket:

    upload dataset -> submit discovery run -> wait for completion ->
    fetch + explain the discovered world -> forecast -> write an HTML report.

Because the client talks to the app object through an in-process WSGI transport,
the whole thing is deterministic and needs no network. Run it with::

    PYTHONPATH="python/lawsynth/src:services/api/src:python/lawsynth-server/src" \
        python3 python/lawsynth/examples/service_client.py

The native engine (built via ``python/lawsynth/scripts/build-native.sh``) backs
discovery, forecast, and simulation on the server side.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import lawsynth
from lawsynth_api import ApiSettings, create_wsgi_app
from lawsynth_server.settings import Settings as ServerSettings

TOKEN = "0123456789abcdef0123456789abcdef"  # 32-char demo token (>= 16 required)


def lotka_volterra(n: int = 80, dt: float = 0.1) -> tuple[list[float], dict[str, list[float]]]:
    """A short, clean predator-prey trajectory --- ideal for the ecology preset."""
    time: list[float] = []
    x_series: list[float] = []
    y_series: list[float] = []
    x, y = 10.0, 5.0
    alpha, beta, delta, gamma = 1.1, 0.4, 0.1, 0.4
    for step in range(n):
        time.append(round(step * dt, 6))
        x_series.append(x)
        y_series.append(y)
        dx = alpha * x - beta * x * y
        dy = delta * x * y - gamma * y
        x += dx * dt
        y += dy * dt
    return time, {"x": x_series, "y": y_series}


def build_app(root: Path):
    """Construct the real API WSGI app over a temp SQLite store and object root."""
    server = ServerSettings(
        database_url=f"sqlite:///{root / 'metadata.sqlite3'}",
        object_root=root / "objects",
        tokens={TOKEN: ("acme", frozenset({"read", "write"}))},
        max_upload_bytes=8 * 1024 * 1024,
    )
    return create_wsgi_app(
        ApiSettings(server=server, environment="test", max_request_bytes=8 * 1024 * 1024)
    )


def main() -> None:
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth-service-"))
    app = build_app(workdir)
    client = lawsynth.Client(wsgi_app=app, token=TOKEN)

    print("LawSynth service client --- offline in-process transcript")
    print("=" * 60)

    banner = client.version()
    print(f"connected: API version {banner.get('version')} (protocol {banner.get('protocol')})")

    time, columns = lotka_volterra()
    print(f"dataset:   {len(time)} samples, columns {sorted(columns)}")

    # Submit a discovery run. The client uploads the dataset, then references it
    # in POST /v1/runs with the 'ecology' preset (resolved client-side into
    # concrete discovery knobs the service accepts).
    run = client.submit_discovery(
        columns=columns,
        time=time,
        state=["x", "y"],
        preset="ecology",
        name="predator-prey",
    )
    print(f"submit:    run {run.id} -> status {run.status!r}")

    run = client.wait(run)
    print(f"wait:      status {run.status!r}, world_id {run.world_id}")
    if not run.succeeded:
        raise SystemExit(f"discovery did not succeed: {run.raw}")

    # The run-scoped world endpoint may still be pending on the API side; the
    # client degrades to GET /v1/worlds/{id} transparently.
    world = client.world(run)
    print(f"world:     {world.get('name')!r} with states {world.get('states')}")
    for target, expression in sorted(dict(world.get("equations", {})).items()):
        print(f"             d{target}/dt = {expression}")

    explanation = client.explain(run.world_id)
    complexity = explanation.get("complexity", {})
    print(
        f"explain:   {complexity.get('laws')} laws, "
        f"{complexity.get('total_terms')} total terms across "
        f"{len(explanation.get('variables', []))} variables"
    )
    for law in explanation.get("laws", []):
        print(f"             {law.get('readable')}   (dominant: {law.get('dominant_term')})")

    # 'Use' the world: a native forecast from the first observation.
    forecast = client.forecast(
        run.world_id,
        initial={"x": columns["x"][0], "y": columns["y"][0]},
        horizon=4.0,
        step=0.1,
    )
    samples = len(forecast["trajectory"]["time"])
    finals = {name: series[-1] for name, series in forecast["trajectory"]["values"].items()}
    print(f"forecast:  {samples} samples, final state "
          f"{{{', '.join(f'{k}={v:.3f}' for k, v in sorted(finals.items()))}}}")

    report_path = client.report(run.world_id, workdir / "world-report.html")
    print(f"report:    wrote {report_path.stat().st_size} bytes -> {report_path}")

    print("=" * 60)
    print("done: submit -> succeeded -> explained -> forecast -> report, fully offline.")

    app.close()


if __name__ == "__main__":
    main()
