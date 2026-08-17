from __future__ import annotations

import sys
from pathlib import Path

SRC = Path(__file__).resolve().parents[1] / "src"
if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))

import changes  # noqa: E402
import commits as commits_mod  # noqa: E402
import main  # noqa: E402
import publish  # noqa: E402
import render  # noqa: E402


def test_parse_conventional_header() -> None:
    commit = commits_mod.parse_commit("feat(world): add discrete bundle codec")
    assert commit.type == "feat"
    assert commit.scope == "world"
    assert commit.subject == "add discrete bundle codec"
    assert commit.breaking is False


def test_parse_breaking_marker_and_footer() -> None:
    bang = commits_mod.parse_commit("feat!: drop legacy field")
    assert bang.breaking is True
    footer = commits_mod.parse_commit("fix: tweak\n\nBREAKING CHANGE: removed x")
    assert footer.breaking is True


def test_non_conventional_is_preserved_as_other() -> None:
    commit = commits_mod.parse_commit("dragon warrior")
    assert commit.type == "other"
    assert commit.subject == "dragon warrior"


def test_infer_bump_precedence() -> None:
    parsed = commits_mod.parse_messages(["fix: a", "feat: b"])
    assert changes.infer_bump(parsed) == "minor"
    parsed = commits_mod.parse_messages(["feat!: c"])
    assert changes.infer_bump(parsed) == "major"
    parsed = commits_mod.parse_messages(["fix: d"])
    assert changes.infer_bump(parsed) == "patch"


def test_changeset_groups_and_dedupes() -> None:
    parsed = commits_mod.parse_messages(["feat: a", "feat: a", "fix(io): b"])
    changeset = changes.build_changeset(parsed)
    titles = [section.title for section in changeset.sections]
    assert titles == ["Features", "Bug Fixes"]
    features = changeset.sections[0].entries
    assert features == ("a",)  # deduped
    assert changeset.sections[1].entries == ("**io:** b",)


def test_render_markdown_contains_headings() -> None:
    parsed = commits_mod.parse_messages(["feat!: breaking one", "fix: repair"])
    changeset = changes.build_changeset(parsed)
    text = render.render_notes(changeset, "0.2.0", "2026-08-17")
    assert "## 0.2.0 - 2026-08-17" in text
    assert "### BREAKING CHANGES" in text
    assert "- repair" in text


def test_publish_inserts_after_unreleased() -> None:
    changelog = "# Changelog\n\n## Unreleased\n\n- pending\n\n## 0.1.0 - old\n\n- first\n"
    notes = "## 0.2.0 - 2026-08-17\n\n### Features\n\n- new thing\n"
    updated = publish.insert_release(changelog, notes)
    assert updated.index("## 0.2.0") < updated.index("## 0.1.0")
    assert updated.index("## Unreleased") < updated.index("## 0.2.0")


def test_cli_render_from_file(tmp_path: Path, capsys) -> None:
    commits_file = tmp_path / "commits.txt"
    commits_file.write_text("feat: alpha\n---\nfix: beta\n", encoding="utf-8")
    code = main.main(
        ["render", "--version", "1.0.0", "--date", "2026-08-17",
         "--commits-file", str(commits_file)]
    )
    assert code == 0
    out = capsys.readouterr().out
    assert "## 1.0.0 - 2026-08-17" in out
    assert "- alpha" in out


def test_cli_publish_dry_run(tmp_path: Path, capsys) -> None:
    commits_file = tmp_path / "commits.txt"
    commits_file.write_text("feat: gamma\n", encoding="utf-8")
    changelog = tmp_path / "CHANGELOG.md"
    changelog.write_text("# Changelog\n\n## Unreleased\n\n- wip\n", encoding="utf-8")
    code = main.main(
        ["publish", "--version", "2.0.0", "--changelog", str(changelog),
         "--commits-file", str(commits_file), "--dry-run"]
    )
    assert code == 0
    assert "## 2.0.0" in capsys.readouterr().out
    # dry-run must not modify the file
    assert "2.0.0" not in changelog.read_text(encoding="utf-8")
