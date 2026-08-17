"""Tests for the bundle inspector.

These tests build real stored-ZIP ``.lsworld`` bundles using the documented
binary-v1 wire format (``specs/bundle/``) and exercise the inspector end to end,
including tamper detection.  They require no Rust toolchain and are fully
deterministic and offline.
"""

from __future__ import annotations

import hashlib
import struct
import sys
import zipfile
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from archive import InvalidArchive, read_archive  # noqa: E402
from checksum import parse_checksums, verify_archive  # noqa: E402
from main import inspect  # noqa: E402
from manifest import decode_world, validate_manifest  # noqa: E402

MANIFEST = (
    b"{\n"
    b'  "format": "lawsynth-world",\n'
    b'  "format_version": "0.1.0",\n'
    b'  "world_encoding": "lawsynth-world-binary-v1"\n'
    b"}\n"
)


def _string(value: str) -> bytes:
    encoded = value.encode("utf-8")
    return struct.pack("<H", len(encoded)) + encoded


def _minimal_world() -> bytes:
    # One continuous state variable x with law d/dt x = 1.
    payload = bytearray(b"LSW1")
    payload += struct.pack("<I", 1)  # variable count
    payload += _string("x") + bytes((0,)) + b"\x00"  # id, role=state, no unit
    payload += struct.pack("<I", 0)  # parameter count
    payload += struct.pack("<I", 1)  # law count
    payload += _string("x") + b"\x00" + struct.pack("<d", 1.0)  # target, constant 1.0
    return bytes(payload)


def _write_bundle(path: Path, world: bytes, *, corrupt: bool = False) -> None:
    entries = {"manifest.json": MANIFEST, "world/world.bin": world}
    sums = "".join(f"{hashlib.sha256(v).hexdigest()}  {n}\n" for n, v in entries.items())
    entries["provenance/checksums.sha256"] = sums.encode("utf-8")
    if corrupt:
        # Change the encoded constant (last 8 bytes) so the payload still decodes
        # but its SHA-256 no longer matches the recorded checksum.
        entries["world/world.bin"] = world[:-8] + struct.pack("<d", 2.0)
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, value in entries.items():
            archive.writestr(name, value)


@pytest.fixture
def bundle(tmp_path: Path) -> Path:
    path = tmp_path / "world.lsworld"
    _write_bundle(path, _minimal_world())
    return path


def test_read_archive_has_required_entries(bundle: Path) -> None:
    archive = read_archive(bundle)
    assert set(archive.entries) == {
        "manifest.json",
        "provenance/checksums.sha256",
        "world/world.bin",
    }
    assert archive.warnings == ()


def test_manifest_and_checksums_verify(bundle: Path) -> None:
    archive = read_archive(bundle)
    validate_manifest(archive)
    report = verify_archive(archive)
    assert report.ok
    assert report.failures == ()


def test_decode_world(bundle: Path) -> None:
    world = decode_world(read_archive(bundle))
    assert world.kind == "continuous"
    assert world.state_count == 1
    assert world.variables[0].id == "x"
    assert world.laws[0].target == "x"
    assert world.laws[0].expression == "1"


def test_inspect_ok(bundle: Path) -> None:
    ok, text, payload = inspect(str(bundle))
    assert ok
    assert "continuous world: 1 states" not in text  # human phrasing differs
    assert "continuous (1 states, 1 variables)" in text
    assert '"ok": true' in payload


def test_checksum_tamper_detected(tmp_path: Path) -> None:
    path = tmp_path / "bad.lsworld"
    _write_bundle(path, _minimal_world(), corrupt=True)
    ok, _text, _payload = inspect(str(path))
    assert not ok


def test_missing_entry_rejected(tmp_path: Path) -> None:
    path = tmp_path / "partial.lsworld"
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_STORED) as archive:
        archive.writestr("manifest.json", MANIFEST)
    with pytest.raises(InvalidArchive):
        read_archive(path)


def test_parse_checksums_rejects_malformed() -> None:
    with pytest.raises(InvalidArchive):
        parse_checksums("not-a-digest world/world.bin\n")
