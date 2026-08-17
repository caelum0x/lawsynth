"""Command-line entry point for lawsynth-release-notes.

Assembles release notes from Conventional Commits (read from local git history
or a plain-text file) and optionally splices them into ``CHANGELOG.md``.
Everything is deterministic and offline.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import changes
import commits as commits_mod
import publish
import render

_COMMIT_DELIMITER = "\n---\n"


def _load_commits(args: argparse.Namespace) -> list[commits_mod.Commit]:
    if args.commits_file is not None:
        text = Path(args.commits_file).read_text(encoding="utf-8")
        messages = [block for block in text.split(_COMMIT_DELIMITER)]
        return commits_mod.parse_messages(messages)
    return commits_mod.read_git_log(args.range)


def _cmd_render(args: argparse.Namespace) -> int:
    parsed = _load_commits(args)
    changeset = changes.build_changeset(parsed)
    sys.stdout.write(render.render_notes(changeset, args.version, args.date))
    return 0


def _cmd_bump(args: argparse.Namespace) -> int:
    parsed = _load_commits(args)
    print(changes.infer_bump(parsed))
    return 0


def _cmd_publish(args: argparse.Namespace) -> int:
    parsed = _load_commits(args)
    changeset = changes.build_changeset(parsed)
    notes = render.render_notes(changeset, args.version, args.date)
    changelog_path = Path(args.changelog)
    existing = changelog_path.read_text(encoding="utf-8") if changelog_path.exists() else "# Changelog\n"
    updated = publish.insert_release(existing, notes)
    if args.dry_run:
        sys.stdout.write(updated)
    else:
        changelog_path.write_text(updated, encoding="utf-8")
        print(f"updated {changelog_path}")
    return 0


def _add_source_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--range", default="HEAD", help="git revision range, e.g. v0.1.0..HEAD")
    parser.add_argument(
        "--commits-file",
        help="read commits from a file (records separated by a line containing only '---')",
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="lawsynth-release-notes", description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    render_cmd = sub.add_parser("render", help="print release notes as Markdown")
    render_cmd.add_argument("--version", required=True)
    render_cmd.add_argument("--date", default="unreleased")
    _add_source_args(render_cmd)
    render_cmd.set_defaults(func=_cmd_render)

    bump_cmd = sub.add_parser("bump", help="print the inferred semantic version bump")
    _add_source_args(bump_cmd)
    bump_cmd.set_defaults(func=_cmd_bump)

    publish_cmd = sub.add_parser("publish", help="splice notes into a changelog")
    publish_cmd.add_argument("--version", required=True)
    publish_cmd.add_argument("--date", default="unreleased")
    publish_cmd.add_argument("--changelog", default="CHANGELOG.md")
    publish_cmd.add_argument("--dry-run", action="store_true")
    _add_source_args(publish_cmd)
    publish_cmd.set_defaults(func=_cmd_publish)

    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
