from __future__ import annotations

import json
import sys
from pathlib import Path

SRC = Path(__file__).resolve().parents[1] / "src"
if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))

import canonicalize  # noqa: E402
import checksum  # noqa: E402
import generate  # noqa: E402
import main  # noqa: E402
import package as package_mod  # noqa: E402


def test_canonical_json_is_sorted_and_terminated() -> None:
    text = canonicalize.canonical_json({"b": 1, "a": 2})
    assert text.endswith("\n")
    assert text.index('"a"') < text.index('"b"')


def test_canonical_json_rejects_non_finite() -> None:
    try:
        canonicalize.canonical_json({"x": float("inf")})
    except ValueError:
        pass
    else:
        raise AssertionError("expected ValueError for non-finite float")


def test_sha256_and_stable_hash_are_deterministic() -> None:
    data = b"lawsynth"
    assert checksum.sha256_hex(data) == checksum.sha256_hex(data)
    assert checksum.stable_hash(data) == checksum.stable_hash(data)
    assert 0 <= checksum.stable_hash(data) <= 0xFFFFFFFFFFFFFFFF
    # Known FNV-1a 64-bit value for the empty input (offset basis).
    assert checksum.stable_hash(b"") == 0xCBF29CE484222325


def test_observation_fixture_shape_and_determinism() -> None:
    spec = {"name": "decay", "type": "observation", "kind": "exponential_decay",
            "samples": 5, "step": 0.1, "parameters": {"rate": 1.0}}
    first = generate.build_fixture(spec)
    second = generate.build_fixture(spec)
    assert first == second
    assert first["sample_count"] == 5
    assert first["columns"] == ["x"]
    assert first["rows"][0]["time"] == 0.0
    assert abs(first["rows"][0]["x"] - 1.0) < 1e-12


def test_noise_is_seeded_and_reproducible() -> None:
    spec = {"name": "noisy", "type": "observation", "kind": "exponential_decay",
            "samples": 4, "step": 0.1, "noise": 0.01}
    assert generate.build_fixture(spec) == generate.build_fixture(spec)


def test_world_bundle_orders_lexically() -> None:
    spec = {
        "name": "w", "type": "world_bundle", "kind": "continuous",
        "variables": [{"id": "x", "role": "State"}, {"id": "a", "role": "Parameter"}],
        "parameters": [{"id": "k", "value": 1.0}],
        "laws": [{"target": "x", "expression": "-k * x"}],
    }
    bundle = generate.build_fixture(spec)
    assert [v["id"] for v in bundle["variables"]] == ["a", "x"]
    assert bundle["spec_version"] == "0.1"


def test_build_set_detects_duplicates() -> None:
    specs = [
        {"name": "dup", "type": "observation", "kind": "harmonic", "samples": 3, "step": 0.5},
        {"name": "dup", "type": "observation", "kind": "harmonic", "samples": 3, "step": 0.5},
    ]
    try:
        package_mod.build_set(specs)
    except ValueError:
        pass
    else:
        raise AssertionError("expected duplicate-name ValueError")


def test_cli_build_and_verify_roundtrip(tmp_path: Path, capsys) -> None:
    spec_file = tmp_path / "spec.json"
    spec_file.write_text(
        json.dumps([
            {"name": "decay", "type": "observation", "kind": "exponential_decay",
             "samples": 6, "step": 0.05, "parameters": {"rate": 2.0}},
        ]),
        encoding="utf-8",
    )
    out_dir = tmp_path / "out"
    assert main.main(["build", str(spec_file), "--out", str(out_dir)]) == 0
    capsys.readouterr()

    manifest = json.loads((out_dir / "manifest.json").read_text(encoding="utf-8"))
    assert manifest["fixtures"][0]["name"] == "decay"

    # A freshly built copy is byte-identical (determinism).
    assert main.main(["verify", str(spec_file), str(out_dir)]) == 0
    assert "OK" in capsys.readouterr().out

    # Tampering is detected.
    (out_dir / "decay.json").write_text("{}", encoding="utf-8")
    assert main.main(["verify", str(spec_file), str(out_dir)]) == 1
