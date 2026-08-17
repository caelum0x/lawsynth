"""Group parsed commits into release-note sections and infer version bumps."""

from __future__ import annotations

from dataclasses import dataclass

from commits import Commit

# Ordered mapping of commit type -> human section heading. Order defines the
# order sections appear in rendered notes.
SECTION_TITLES: dict[str, str] = {
    "feat": "Features",
    "fix": "Bug Fixes",
    "perf": "Performance",
    "refactor": "Refactoring",
    "docs": "Documentation",
    "test": "Tests",
    "build": "Build System",
    "ci": "Continuous Integration",
    "chore": "Chores",
    "other": "Other Changes",
}

# Types that are hidden from release notes unless a commit is breaking.
_HIDDEN = frozenset({"test", "chore", "ci", "build", "other"})


@dataclass(frozen=True)
class Section:
    title: str
    entries: tuple[str, ...]


@dataclass(frozen=True)
class ChangeSet:
    """A rendered-ready grouping of changes plus the inferred version bump."""

    breaking: tuple[str, ...]
    sections: tuple[Section, ...]
    bump: str  # "major" | "minor" | "patch"

    @property
    def is_empty(self) -> bool:
        return not self.breaking and not self.sections


def _format_entry(commit: Commit) -> str:
    if commit.scope:
        return f"**{commit.scope}:** {commit.subject}"
    return commit.subject


def infer_bump(commits: list[Commit]) -> str:
    """Return the semantic-version bump implied by a set of commits."""
    if any(commit.breaking for commit in commits):
        return "major"
    if any(commit.type == "feat" for commit in commits):
        return "minor"
    return "patch"


def build_changeset(commits: list[Commit]) -> ChangeSet:
    """Group commits by type, collect breaking changes, and sort deterministically."""
    breaking: list[str] = []
    grouped: dict[str, list[str]] = {key: [] for key in SECTION_TITLES}

    for commit in commits:
        entry = _format_entry(commit)
        if commit.breaking:
            breaking.append(entry)
        bucket = commit.type if commit.type in grouped else "other"
        if bucket in _HIDDEN and not commit.breaking:
            # Hidden housekeeping types only surface via the breaking list.
            if bucket == "other":
                grouped[bucket].append(entry)
            continue
        grouped[bucket].append(entry)

    sections: list[Section] = []
    for key, title in SECTION_TITLES.items():
        entries = grouped[key]
        if not entries:
            continue
        # Deduplicate while sorting for stable, reproducible output.
        unique = tuple(sorted(dict.fromkeys(entries)))
        sections.append(Section(title=title, entries=unique))

    return ChangeSet(
        breaking=tuple(sorted(dict.fromkeys(breaking))),
        sections=tuple(sections),
        bump=infer_bump(commits),
    )
