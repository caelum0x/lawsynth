from __future__ import annotations

import json
import sys
from pathlib import Path

SRC = Path(__file__).resolve().parents[1] / "src"
if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))

import main  # noqa: E402
import notice as notice_mod  # noqa: E402
import policy as policy_mod  # noqa: E402
import report as report_mod  # noqa: E402
import scan  # noqa: E402


def test_default_policy_matches_deny_toml_allowlist() -> None:
    policy = policy_mod.Policy()
    assert policy.permits_license("MIT")
    assert policy.permits_license("Apache-2.0")
    assert not policy.permits_license("GPL-3.0-only")


def test_spdx_expression_and_or() -> None:
    policy = policy_mod.Policy()
    assert policy.permits_expression("MIT OR Apache-2.0")
    assert policy.permits_expression("MIT AND Apache-2.0")
    assert not policy.permits_expression("MIT AND GPL-3.0-only")
    assert policy.permits_expression("GPL-3.0-only OR MIT")
    assert policy.permits_expression("Apache-2.0 WITH LLVM-exception")


def test_load_policy_from_deny_toml(tmp_path: Path) -> None:
    deny = tmp_path / "deny.toml"
    deny.write_text('[licenses]\nallow = ["MIT", "ISC"]\n', encoding="utf-8")
    policy = policy_mod.load_policy(deny)
    assert policy.permits_license("ISC")
    assert not policy.permits_license("Apache-2.0")


def test_scan_cargo_lock_and_inventory(tmp_path: Path) -> None:
    cargo = tmp_path / "Cargo.lock"
    cargo.write_text(
        '[[package]]\nname = "serde"\nversion = "1.0.0"\n\n'
        '[[package]]\nname = "anyhow"\nversion = "1.0.0"\n',
        encoding="utf-8",
    )
    inventory = tmp_path / "inventory.json"
    inventory.write_text(
        json.dumps([{"name": "serde", "version": "1.0.0", "license": "MIT OR Apache-2.0"}]),
        encoding="utf-8",
    )
    deps = scan.scan_paths([cargo, inventory])
    by_name = {d.name: d for d in deps}
    # The inventory license overrides the license-less Cargo.lock entry.
    assert by_name["serde"].license == "MIT OR Apache-2.0"
    assert by_name["anyhow"].license is None


def test_evaluate_classifies_findings() -> None:
    deps = [
        scan.Dependency("ok", "1.0", "MIT", "inv"),
        scan.Dependency("bad", "1.0", "GPL-3.0-only", "inv"),
        scan.Dependency("mystery", "1.0", None, "Cargo.lock"),
    ]
    result = report_mod.evaluate(deps, policy_mod.Policy())
    statuses = {f.name: f.status for f in result.findings}
    assert statuses == {"ok": "allowed", "bad": "denied", "mystery": "unknown"}
    assert not result.ok


def test_notice_groups_by_license() -> None:
    deps = [
        scan.Dependency("a", "1.0", "MIT", "inv"),
        scan.Dependency("b", "2.0", "MIT", "inv"),
        scan.Dependency("c", "3.0", None, "inv"),
    ]
    body = notice_mod.render_notice(deps)
    assert "## MIT" in body
    assert "## UNKNOWN" in body
    assert "- a 1.0" in body


def test_cli_check_denies_and_json(tmp_path: Path, capsys) -> None:
    inventory = tmp_path / "inv.json"
    inventory.write_text(
        json.dumps([{"name": "bad", "version": "1.0", "license": "GPL-3.0-only"}]),
        encoding="utf-8",
    )
    code = main.main(["check", str(inventory), "--format", "json"])
    assert code == 1
    payload = json.loads(capsys.readouterr().out)
    assert payload["ok"] is False
    assert payload["denied"] == ["bad"]


def test_cli_check_allow_unknown(tmp_path: Path) -> None:
    cargo = tmp_path / "Cargo.lock"
    cargo.write_text('[[package]]\nname = "x"\nversion = "1.0"\n', encoding="utf-8")
    assert main.main(["check", str(cargo)]) == 1
    assert main.main(["check", str(cargo), "--allow-unknown"]) == 0
