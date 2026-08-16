#!/usr/bin/env python3
"""Real executable workflow support for release-level LawSynth test cases.

The cases use a small, independent binary-v1 writer to exercise the public
bundle decoder through the CLI.  It is intentionally not a reimplementation
of the simulator: every trajectory and discovery assertion comes from the
compiled LawSynth commands.
"""

from __future__ import annotations

import csv
import hashlib
import importlib
import json
import struct
import subprocess
import sys
import tempfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = b'{\n  "format": "lawsynth-world",\n  "format_version": "0.1.0",\n  "world_encoding": "lawsynth-world-binary-v1"\n}\n'


def _string(value: str) -> bytes:
    data = value.encode("utf-8")
    if len(data) > 65_535:
        raise ValueError("binary-v1 strings are limited to 65535 bytes")
    return struct.pack("<H", len(data)) + data


def _expr(node: dict[str, object]) -> bytes:
    if "constant" in node:
        return b"\x00" + struct.pack("<d", float(node["constant"]))
    if "symbol" in node:
        return b"\x01" + _string(str(node["symbol"]))
    raise ValueError(f"fixture expression is not supported by binary-v1 writer: {node}")


def write_world_bundle(path: Path, world: dict[str, object]) -> None:
    payload = bytearray(b"LSW1")
    variables = world["variables"]
    parameters = world.get("parameters", [])
    laws = world["laws"]
    roles = {"state": 0, "control": 1, "exogenous": 2, "observed": 3, "latent": 4, "derived": 5}
    payload.extend(struct.pack("<I", len(variables)))
    for variable in variables:  # type: ignore[union-attr]
        payload.extend(_string(variable["id"]))  # type: ignore[index]
        payload.append(roles[variable["role"]])  # type: ignore[index]
        payload.append(0)  # no unit
    payload.extend(struct.pack("<I", len(parameters)))
    for parameter in parameters:  # type: ignore[union-attr]
        payload.extend(_string(parameter["id"]))  # type: ignore[index]
        payload.extend(struct.pack("<d", float(parameter["value"])))  # type: ignore[index]
        payload.append(0)
    payload.extend(struct.pack("<I", len(laws)))
    for law in laws:  # type: ignore[union-attr]
        payload.extend(_string(law["target"]))  # type: ignore[index]
        payload.extend(_expr(law["expression"]))  # type: ignore[index]
    entries = {"manifest.json": MANIFEST, "world/world.bin": bytes(payload)}
    checksums = "".join(f"{hashlib.sha256(data).hexdigest()}  {name}\n" for name, data in entries.items())
    entries["provenance/checksums.sha256"] = checksums.encode("ascii")
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, data in entries.items():
            archive.writestr(name, data)


def cli(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "run", "--quiet", "-p", "lawsynth-cli", "--bin", "lawsynth", "--", *args],
        cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )


def _assert_trajectory(result: subprocess.CompletedProcess[str], expected: dict[str, object]) -> None:
    assert result.returncode == 0, result.stderr
    rows = list(csv.DictReader(result.stdout.splitlines()))
    assert len(rows) == expected["samples"], result.stdout
    state = str(expected["state"])
    values = [float(row[state]) for row in rows]
    wanted = [float(value) for value in expected["values"]]  # type: ignore[index]
    assert len(values) == len(wanted)
    for got, want in zip(values, wanted, strict=True):
        assert abs(got - want) <= 1e-12, f"{got} != {want}"


def _native_boundary(bundle: Path) -> str:
    sdk = ROOT / "python" / "lawsynth" / "src"
    if str(sdk) not in sys.path:
        sys.path.insert(0, str(sdk))
    lawsynth = importlib.import_module("lawsynth")
    # The pure-Python facade is a real public boundary too: validate it before
    # probing the optional compiled module.
    dataset = lawsynth.Dataset(time=(0.0, 1.0), columns={"x": (2.0, 3.0)})
    assert len(dataset.time) == 2
    try:
        world_type = lawsynth.World
    except lawsynth.NativeError:
        return "unavailable"
    world = world_type(["x"], {}, {"x": "1"})
    trajectory = world.simulate({"x": 2.0}, end=1.0, step=0.5)
    assert trajectory.values["x"][-1] == 3.0
    world.save(str(bundle))
    loaded = cli("inspect", str(bundle))
    assert loaded.returncode == 0, loaded.stderr
    return "available"


def run_case(directory: Path) -> None:
    with (directory / "case.toml").open("rb") as handle:
        case = tomllib.load(handle)["case"]
    source = json.loads((directory / "input.json").read_text())
    expected = json.loads((directory / "expected.json").read_text())
    assert case["id"] == directory.name == source["case_id"]
    assert case["workflow"] == expected["workflow"]

    with tempfile.TemporaryDirectory(prefix=f"lawsynth-{directory.name}-") as temporary:
        temporary_path = Path(temporary)
        if expected["workflow"] == "boundary":
            rejected = cli(str(source["unsupported_command"]))
            assert rejected.returncode != 0, "unsupported CLI surface unexpectedly succeeded"
            assert expected["capability"] in source["capability"]
            print(f"{directory.name}: unavailable capability is explicitly bounded")
            return

        if expected["workflow"] == "discover":
            csv_path = temporary_path / "observations.csv"
            with csv_path.open("w", newline="") as handle:
                writer = csv.DictWriter(handle, fieldnames=source["dataset"]["columns"])
                writer.writeheader()
                writer.writerows(source["dataset"]["rows"])
            bundle = temporary_path / "discovered.lsworld"
            result = cli("discover", str(csv_path), "--time", "time", "--state", "x", "--degree", "1", "--threshold", "0.001", "--output", str(bundle))
            assert result.returncode == 0, result.stderr
            inspected = cli("inspect", str(bundle))
            assert inspected.returncode == 0 and "continuous world" in inspected.stdout, inspected.stderr
            print(f"{directory.name}: CLI discovered and serialized a world")
            return

        bundle = temporary_path / "world.lsworld"
        if expected["workflow"] == "native-optional":
            availability = _native_boundary(bundle)
            assert availability in expected["native_status"]
            print(f"{directory.name}: Python native boundary {availability}")
            return

        write_world_bundle(bundle, source["world"])
        inspected = cli("inspect", str(bundle))
        assert inspected.returncode == 0 and "continuous world" in inspected.stdout, inspected.stderr
        result = cli("simulate", str(bundle), "--initial", "x=2", "--start", "0", "--end", "1", "--step", "0.5")
        _assert_trajectory(result, expected)
        print(f"{directory.name}: bundle decoded and native simulation matched")
