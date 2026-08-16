#!/usr/bin/env python3
"""Executable native-CLI conformance case for the current LawSynth runtime.

The case files are deliberately declarative. This runner creates a real stored-ZIP
.lsworld archive using the documented binary-v1 layout, then invokes the Rust CLI.
It does not emulate simulation or acceptance behavior.
"""

from __future__ import annotations

import csv
import hashlib
import json
import struct
import subprocess
import tempfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = b'{\n  "format": "lawsynth-world",\n  "format_version": "0.1.0",\n  "world_encoding": "lawsynth-world-binary-v1"\n}\n'


def put_string(value: str) -> bytes:
    encoded = value.encode("utf-8")
    if len(encoded) > 65535:
        raise ValueError("bundle string is too long")
    return struct.pack("<H", len(encoded)) + encoded


def put_optional_string(value: str | None) -> bytes:
    return b"\x00" if value is None else b"\x01" + put_string(value)


def put_expression(expression: dict[str, object]) -> bytes:
    if "constant" in expression:
        return b"\x00" + struct.pack("<d", float(expression["constant"]))
    if "symbol" in expression:
        return b"\x01" + put_string(str(expression["symbol"]))
    if "unary" in expression:
        operators = {"negate": 0, "exp": 1, "log": 2, "sin": 3, "cos": 4}
        return bytes((2, operators[str(expression["unary"])])) + put_expression(expression["operand"])  # type: ignore[arg-type]
    operators = {"add": 0, "subtract": 1, "multiply": 2, "divide": 3, "power": 4}
    return bytes((3, operators[str(expression["binary"])])) + put_expression(expression["left"]) + put_expression(expression["right"])  # type: ignore[arg-type]


def encode_world(bundle: dict[str, object]) -> bytes:
    kind = str(bundle["kind"])
    output = bytearray(b"LSW1" if kind == "continuous" else b"LSD1")
    variables = bundle["variables"]
    parameters = bundle["parameters"]
    laws = bundle["laws"]
    output.extend(struct.pack("<I", len(variables)))  # type: ignore[arg-type]
    roles = {"state": 0, "control": 1, "exogenous": 2, "observed": 3, "latent": 4, "derived": 5}
    for variable in variables:  # type: ignore[union-attr]
        output.extend(put_string(variable["id"]))  # type: ignore[index]
        output.append(roles[variable["role"]])  # type: ignore[index]
        output.extend(put_optional_string(variable.get("unit")))  # type: ignore[union-attr]
    output.extend(struct.pack("<I", len(parameters)))  # type: ignore[arg-type]
    for parameter in parameters:  # type: ignore[union-attr]
        output.extend(put_string(parameter["id"]))  # type: ignore[index]
        output.extend(struct.pack("<d", float(parameter["value"])))  # type: ignore[index]
        output.extend(put_optional_string(parameter.get("unit")))  # type: ignore[union-attr]
    output.extend(struct.pack("<I", len(laws)))  # type: ignore[arg-type]
    for law in laws:  # type: ignore[union-attr]
        output.extend(put_string(law["target"]))  # type: ignore[index]
        output.extend(put_expression(law["expression"]))  # type: ignore[index]
    return bytes(output)


def write_bundle(path: Path, world: bytes, checksum: bool = True, unsafe: bool = False) -> None:
    entries = {
        "manifest.json": MANIFEST,
        "world/world.bin": world,
    }
    sums = "".join(f"{hashlib.sha256(value).hexdigest()}  {name}\n" for name, value in entries.items())
    entries["provenance/checksums.sha256"] = sums.encode()
    if not checksum:
        entries["provenance/checksums.sha256"] = b"0" * 64 + b"  manifest.json\n"
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        if unsafe:
            archive.writestr("../world/world.bin", world)
        else:
            for name, value in entries.items():
                archive.writestr(name, value)


def invalid_world(mutation: str) -> bytes:
    if mutation == "unknown-expression-tag":
        return b"LSW1" + struct.pack("<III", 0, 0, 1) + put_string("x") + b"\xff"
    if mutation == "invalid-unit":
        return (
            b"LSW1"
            + struct.pack("<I", 1)
            + put_string("x")
            + b"\x00\x01"
            + put_string("definitely_not_a_unit")
            + struct.pack("<II", 0, 0)
        )
    raise ValueError(f"unknown invalid fixture mutation: {mutation}")


def cli(*arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", "run", "--quiet", "-p", "lawsynth-cli", "--bin", "lawsynth", "--", *arguments],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def check_columns(stdout: str, expected: dict[str, list[float]]) -> None:
    rows = list(csv.DictReader(stdout.splitlines()))
    assert rows, "simulation produced no CSV rows"
    assert set(rows[0]) == set(expected), f"unexpected CSV columns: {rows[0].keys()}"
    for name, values in expected.items():
        actual = [float(row[name]) for row in rows]
        assert len(actual) == len(values), f"{name}: wrong sample count"
        for index, (left, right) in enumerate(zip(actual, values, strict=True)):
            assert abs(left - right) <= 1e-12, f"{name}[{index}]: {left} != {right}"


def main() -> None:
    directory = Path(__file__).resolve().parent
    with (directory / "case.toml").open("rb") as handle:
        case = tomllib.load(handle)
    input_data = json.loads((directory / "input.json").read_text())
    expected = json.loads((directory / "expected.json").read_text())
    assert case["case"]["id"] == directory.name == input_data["case_id"]
    assert case["case"]["mode"] == input_data["mode"]
    assert {"valid": "accepted", "invalid": "rejected", "unsupported": "unsupported"}[case["case"]["mode"]] == expected["outcome"]

    if expected["outcome"] == "unsupported":
        assert input_data["feature"] == expected["feature"]
        print(f"{directory.name}: unsupported capability documented: {expected['feature']}")
        return

    with tempfile.TemporaryDirectory(prefix="lawsynth-conformance-") as temporary:
        bundle = Path(temporary) / "fixture.lsworld"
        if expected["outcome"] == "accepted":
            write_bundle(bundle, encode_world(input_data["bundle"]))
            inspected = cli("inspect", str(bundle))
            assert inspected.returncode == 0, inspected.stderr
            assert expected["inspect_contains"] in inspected.stdout
            command = input_data["command"]
            result = cli(command["operation"], str(bundle), *command["arguments"])
            assert result.returncode == 0, result.stderr
            check_columns(result.stdout, expected["columns"])
            print(f"{directory.name}: native CLI accepted fixture")
            return

        mutation = input_data["fixture"]["mutation"]
        if mutation == "not-a-zip":
            bundle.write_bytes(b"not a ZIP archive")
        else:
            write_bundle(
                bundle,
                invalid_world(mutation) if mutation in {"unknown-expression-tag", "invalid-unit"} else b"LSW1" + struct.pack("<III", 0, 0, 0),
                checksum=mutation != "checksum-mismatch",
                unsafe=mutation == "unsafe-path",
            )
        result = cli("inspect", str(bundle))
        assert result.returncode != 0, "invalid fixture unexpectedly loaded"
        combined = result.stdout + result.stderr
        assert expected["error_contains"] in combined, combined
        print(f"{directory.name}: native CLI rejected fixture")


if __name__ == "__main__":
    main()
