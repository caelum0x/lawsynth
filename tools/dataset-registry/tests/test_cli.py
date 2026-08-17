from __future__ import annotations

import json
import sys
from pathlib import Path

SRC = Path(__file__).resolve().parents[1] / "src"
if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))

import card as card_mod  # noqa: E402
import download  # noqa: E402
import main  # noqa: E402
import manifest as manifest_mod  # noqa: E402
import verify as verify_mod  # noqa: E402

CASE_TOML = """\
id = "dynamics/ode-small"
title = "Small continuous ODE"
version = 1

[capability]
status = "supported"
reason = "native discovery is exercised"

[generation]
kind = "exponential_decay"
samples = 81
step = 0.05
"""


def _make_case(root: Path) -> Path:
    case_dir = root / "dynamics" / "ode-small"
    case_dir.mkdir(parents=True)
    (case_dir / "benchmark.toml").write_text(CASE_TOML, encoding="utf-8")
    (case_dir / "expected.json").write_text('{"status":"passed"}\n', encoding="utf-8")
    (case_dir / "baseline.json").write_text('{"baseline":"native"}\n', encoding="utf-8")
    return case_dir


def test_index_case_records_metadata_and_checksums(tmp_path: Path) -> None:
    _make_case(tmp_path)
    entries = manifest_mod.index_tree(tmp_path)
    assert len(entries) == 1
    entry = entries[0]
    assert entry.id == "dynamics/ode-small"
    assert entry.title == "Small continuous ODE"
    assert entry.capability == "supported"
    assert entry.path == "dynamics/ode-small"
    names = {digest.path for digest in entry.files}
    assert {"benchmark.toml", "expected.json", "baseline.json"} <= names
    assert all(len(d.sha256) == 64 for d in entry.files)


def test_registry_roundtrip(tmp_path: Path) -> None:
    _make_case(tmp_path)
    entries = manifest_mod.index_tree(tmp_path)
    document = manifest_mod.registry_document(entries)
    reg_path = tmp_path / "registry.json"
    reg_path.write_text(json.dumps(document), encoding="utf-8")
    loaded = manifest_mod.load_registry(reg_path)
    assert loaded[0].id == entries[0].id
    assert loaded[0].files == entries[0].files


def test_verify_detects_tampering(tmp_path: Path) -> None:
    case_dir = _make_case(tmp_path)
    entries = manifest_mod.index_tree(tmp_path)
    assert verify_mod.verify_registry(entries, tmp_path) == []

    (case_dir / "expected.json").write_text('{"status":"failed"}\n', encoding="utf-8")
    problems = verify_mod.verify_registry(entries, tmp_path)
    assert any(p.kind == "changed" and p.file == "expected.json" for p in problems)

    (case_dir / "baseline.json").unlink()
    problems = verify_mod.verify_registry(entries, tmp_path)
    assert any(p.kind == "missing" and p.file == "baseline.json" for p in problems)


def test_find_entry_and_stage(tmp_path: Path) -> None:
    _make_case(tmp_path)
    entries = manifest_mod.index_tree(tmp_path)
    entry = download.find_entry(entries, "dynamics/ode-small")
    dest = tmp_path / "staged"
    staged = download.stage(entry, tmp_path, dest)
    assert (dest / "benchmark.toml") in staged
    # Staged copy must verify clean.
    assert verify_mod.verify_entry(entry, tmp_path) == []
    try:
        download.find_entry(entries, "nope")
    except download.DatasetNotFound:
        pass
    else:
        raise AssertionError("expected DatasetNotFound")


def test_card_render(tmp_path: Path) -> None:
    _make_case(tmp_path)
    entry = manifest_mod.index_tree(tmp_path)[0]
    text = card_mod.render_card(entry)
    assert "# Small continuous ODE" in text
    assert "`dynamics/ode-small`" in text
    assert "| File | SHA-256 | Bytes |" in text


def test_cli_index_and_verify(tmp_path: Path, capsys) -> None:
    _make_case(tmp_path)
    registry = tmp_path / "registry.json"
    assert main.main(["index", str(tmp_path), "--out", str(registry)]) == 0
    capsys.readouterr()
    assert main.main(["verify", str(registry), "--root", str(tmp_path)]) == 0
    assert "0 problems" in capsys.readouterr().out


def test_cli_card(tmp_path: Path, capsys) -> None:
    _make_case(tmp_path)
    registry = tmp_path / "registry.json"
    main.main(["index", str(tmp_path), "--out", str(registry)])
    capsys.readouterr()
    assert main.main(["card", str(registry), "dynamics/ode-small"]) == 0
    assert "Small continuous ODE" in capsys.readouterr().out
