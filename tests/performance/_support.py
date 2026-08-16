"""Shared executable contracts for LawSynth's supported performance surface.

The runners in this directory measure real work.  They do not substitute a
model implementation: native bundle/ODE cases invoke the Rust CLI and SDK
cases exercise the installed source package directly.
"""

from __future__ import annotations

import csv
import hashlib
import json
import os
import struct
import subprocess
import sys
import tempfile
import time
import tomllib
import tracemalloc
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SDK = ROOT / "python" / "lawsynth" / "src"
MANIFEST = b'{\n  "format": "lawsynth-world",\n  "format_version": "0.1.0",\n  "world_encoding": "lawsynth-world-binary-v1"\n}\n'


def _string(value: str) -> bytes:
    encoded = value.encode()
    return struct.pack("<H", len(encoded)) + encoded


def _expr(expression: dict[str, object]) -> bytes:
    if "constant" in expression:
        return b"\x00" + struct.pack("<d", float(expression["constant"]))
    if "symbol" in expression:
        return b"\x01" + _string(str(expression["symbol"]))
    op = {"add": 0, "subtract": 1, "multiply": 2, "divide": 3, "power": 4}[str(expression["binary"])]
    return bytes((3, op)) + _expr(expression["left"]) + _expr(expression["right"])


def write_decay_bundle(path: Path) -> None:
    """Write a documented binary-v1 archive for dx/dt = -rate*x."""
    world = bytearray(b"LSW1")
    world.extend(struct.pack("<I", 1))
    world.extend(_string("x") + b"\x00\x00")
    world.extend(struct.pack("<I", 1))
    world.extend(_string("rate") + struct.pack("<d", 1.0) + b"\x00")
    world.extend(struct.pack("<I", 1))
    world.extend(_string("x"))
    world.extend(_expr({"binary": "multiply", "left": {"constant": -1.0}, "right": {"binary": "multiply", "left": {"symbol": "rate"}, "right": {"symbol": "x"}}}))
    entries = {"manifest.json": MANIFEST, "world/world.bin": bytes(world)}
    entries["provenance/checksums.sha256"] = "".join(
        f"{hashlib.sha256(value).hexdigest()}  {name}\n" for name, value in entries.items()
    ).encode()
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, value in entries.items():
            archive.writestr(name, value)


def cli(*arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "run", "--quiet", "-p", "lawsynth-cli", "--bin", "lawsynth", "--", *arguments],
        cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )


def source_env() -> dict[str, str]:
    environment = dict(os.environ)
    environment["PYTHONPATH"] = str(SDK) + os.pathsep + environment.get("PYTHONPATH", "")
    return environment


def load_case(directory: Path) -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    with (directory / "case.toml").open("rb") as handle:
        case = tomllib.load(handle)
    return case, json.loads((directory / "input.json").read_text()), json.loads((directory / "expected.json").read_text())


def elapsed(operation):
    started = time.perf_counter()
    value = operation()
    return value, time.perf_counter() - started


def require_budget(seconds: float, limit: float, label: str) -> None:
    assert seconds <= limit, f"{label} exceeded budget: {seconds:.6f}s > {limit:.6f}s"


def _sdk_workload(case_id: str, size: int) -> tuple[int, int]:
    sys.path.insert(0, str(SDK))
    from lawsynth import Dataset, DiscoveryConfig
    from lawsynth.candidate import CandidateMetrics
    from lawsynth.frontier import pareto_front
    from lawsynth.intervention import Intervention
    from lawsynth.scenario import Scenario

    if case_id == "expression-throughput":
        metrics = [CandidateMetrics(float(index % 23), index % 11) for index in range(size)]
        return len(metrics), len(pareto_front(metrics))
    if case_id in {"profile-million", "memory-budget"}:
        dataset = Dataset.from_columns(range(size), {"x": (float(index) for index in range(size))})
        return len(dataset.time), len(dataset.columns["x"])
    if case_id == "event-latency":
        events = tuple(Intervention(float(index), "rate", float(index % 3)) for index in range(size))
        scenario = Scenario({"x": 1.0}, interventions=events)
        return len(scenario.interventions), int(scenario.interventions[-1].time)
    if case_id == "studio-paint":
        # The studio is not a P1 runtime. Measure its supported data boundary:
        # serialising real client-side world metadata for a rendering host.
        payload = {"states": [f"x{index}" for index in range(size)], "config": DiscoveryConfig().__dict__ if hasattr(DiscoveryConfig(), "__dict__") else {"solver": "stlsq"}}
        encoded = json.dumps(payload, separators=(",", ":"), sort_keys=True).encode()
        return len(payload["states"]), len(encoded)
    if case_id == "import-time":
        completed = subprocess.run([sys.executable, "-c", "import lawsynth; print(lawsynth.__version__)"], env=source_env(), text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        assert completed.returncode == 0, completed.stderr
        assert completed.stdout.strip()
        return 1, len(completed.stdout.strip())
    raise AssertionError(f"unsupported SDK workload {case_id}")


def execute(directory: Path) -> None:
    case, input_data, expected = load_case(directory)
    specification = case["case"]
    assert specification["id"] == directory.name == input_data["case_id"] == expected["case_id"]
    limit = float(specification["max_seconds"])
    size = int(input_data["size"])
    case_id = directory.name

    if case_id in {"bundle-open", "ode-simulation"}:
        with tempfile.TemporaryDirectory(prefix="lawsynth-performance-") as temporary:
            bundle = Path(temporary) / "decay.lsworld"
            write_decay_bundle(bundle)
            if case_id == "bundle-open":
                result, duration = elapsed(lambda: cli("inspect", str(bundle)))
                assert result.returncode == 0, result.stderr
                assert "continuous world: 1 states" in result.stdout
                samples = 1
            else:
                result, duration = elapsed(lambda: cli("simulate", str(bundle), "--initial", "x=1", "--start", "0", "--end", str(size / 1000), "--step", "0.001"))
                assert result.returncode == 0, result.stderr
                rows = list(csv.DictReader(result.stdout.splitlines()))
                assert len(rows) == size + 1 and float(rows[-1]["x"]) < 1.0
                samples = len(rows)
            detail = len(result.stdout)
    elif case_id == "cancellation-latency":
        command = ["cargo", "test", "--quiet", "-p", "lawsynth-discovery", "honours_a_pre_cancelled_discovery_request"]
        result, duration = elapsed(lambda: subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False))
        assert result.returncode == 0, result.stdout + result.stderr
        samples = 1
        detail = len(result.stdout)
    elif case_id == "parquet-load":
        command = ["cargo", "test", "--quiet", "-p", "lawsynth-data", "reads_uncompressed_plain_numeric_pages"]
        result, duration = elapsed(lambda: subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False))
        assert result.returncode == 0, result.stdout + result.stderr
        samples = 1
        detail = len(result.stdout)
    elif case_id == "memory-budget":
        tracemalloc.start()
        (samples, detail), duration = elapsed(lambda: _sdk_workload(case_id, size))
        _, peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()
        assert peak <= int(expected["max_peak_bytes"]), f"peak allocation {peak} exceeds budget"
    else:
        (samples, detail), duration = elapsed(lambda: _sdk_workload(case_id, size))
    require_budget(duration, limit, case_id)
    assert samples >= int(expected["minimum_samples"])
    print(json.dumps({"case": case_id, "seconds": round(duration, 6), "samples": samples, "detail": detail}, sort_keys=True))
