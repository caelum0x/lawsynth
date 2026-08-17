# lawsynth-release-notes

`lawsynth-release-notes` assembles human-readable release notes from
[Conventional Commits](https://www.conventionalcommits.org/). It parses commit
messages, groups them into Keep a Changelog style sections, infers the semantic
version bump, and can splice the result into `CHANGELOG.md`.

The tool is dependency-free and offline. Commits can be read from local `git`
history (the only place it shells out) or from a plain text file, which makes it
fully testable and reproducible in CI.

## Usage

```bash
# Print notes for everything since the last tag
lawsynth-release-notes render --version 0.2.0 --date 2026-08-17 --range v0.1.0..HEAD

# Print only the inferred bump: major | minor | patch
lawsynth-release-notes bump --range v0.1.0..HEAD

# Splice notes into CHANGELOG.md (use --dry-run to preview)
lawsynth-release-notes publish --version 0.2.0 --date 2026-08-17 --changelog CHANGELOG.md

# Render from a file instead of git (records separated by a line containing '---')
lawsynth-release-notes render --version 0.2.0 --commits-file commits.txt
```

## Rules

- A commit `type(scope)!: subject` is parsed into its parts; a trailing `!` or a
  `BREAKING CHANGE` footer marks a breaking change.
- Non-conventional messages are preserved under "Other Changes" rather than
  dropped.
- Bump precedence: any breaking change -> `major`; any `feat` -> `minor`;
  otherwise `patch`.
- Section entries are deduplicated and sorted for byte-stable output.

## Boundaries

The tool writes notes only. It does not tag releases, push, or publish
artifacts; those steps are owned by the release workflow (`release-plz`).
