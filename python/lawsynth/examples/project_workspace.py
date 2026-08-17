#!/usr/bin/env python3
"""Organize and share discovered worlds with a LawSynth Project workspace.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/project_workspace.py

A single discovery is a portable ``.lsworld`` bundle; real work produces many.
:class:`lawsynth.Project` is the SDK workspace that keeps them organized and
shareable. This demo discovers two different worlds, registers them in a project
with tags and notes, saves the workspace (a ``library.tsv`` index — the very
format the ``lawsynth library`` CLI uses — plus the bundle files), reloads it
from disk, exports the whole workspace as one archive file, and re-imports that
archive into a fresh directory — confirming every world's content hash survives
the round-trip. Deterministic and offline.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import lawsynth


def _oscillator_study() -> lawsynth.Study:
    """Damped harmonic oscillator, discovered."""
    k, c, dt, steps = 1.0, 0.3, 0.05, 240
    x, v = 1.0, 0.0
    time: list[float] = []
    columns: dict[str, list[float]] = {"x": [], "v": []}
    for i in range(steps):
        time.append(i * dt)
        columns["x"].append(x)
        columns["v"].append(v)

        def deriv(x_: float, v_: float) -> tuple[float, float]:
            return v_, -k * x_ - c * v_

        k1x, k1v = deriv(x, v)
        k2x, k2v = deriv(x + 0.5 * dt * k1x, v + 0.5 * dt * k1v)
        k3x, k3v = deriv(x + 0.5 * dt * k2x, v + 0.5 * dt * k2v)
        k4x, k4v = deriv(x + dt * k3x, v + dt * k3v)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        v += dt / 6 * (k1v + 2 * k2v + 2 * k3v + k4v)
    study = lawsynth.Study.from_columns(time, columns, state=["x", "v"], name="oscillator")
    study.discover(threshold=0.05)
    return study


def _decay_study() -> lawsynth.Study:
    """Exponential decay dy/dt = -0.5·y, discovered."""
    dt, steps = 0.05, 240
    y = 2.0
    time: list[float] = []
    column: list[float] = []
    for i in range(steps):
        time.append(i * dt)
        column.append(y)
        y += dt * (-0.5 * y)
    study = lawsynth.Study.from_columns(time, {"y": column}, state=["y"], name="decay")
    study.discover(threshold=0.05)
    return study


def main() -> None:
    workspace = Path(tempfile.mkdtemp(prefix="lawsynth_project_"))

    # 1. Discover two different worlds.
    oscillator = _oscillator_study()
    decay = _decay_study()

    # 2. Register them in a project with tags + notes, then save the workspace.
    project = lawsynth.Project(workspace)
    project.add("oscillator", oscillator, tags=["physics", "mechanics"],
                note="damped harmonic oscillator")
    # `save_to_project` is the Study-side convenience (add + save in one call).
    decay.save_to_project(project, "decay", tags=["physics", "growth"],
                          note="exponential decay")
    index = project.save()

    print(f"workspace directory: {workspace}")
    print(f"index (CLI-compatible library.tsv): {index}")
    print("bundles on disk:", sorted(p.name for p in workspace.glob("*.lsworld")))
    print("\nlibrary.tsv contents:")
    print(index.read_text().rstrip())
    print()

    print("registered worlds:")
    for entry in project.list():
        print(f"  {entry.name:<12} tags={list(entry.tags)}  hash={entry.world_hash[:12]}…  "
              f"note={entry.note!r}")
    print()

    # 3. Reload the workspace from disk (worlds load lazily on first get()).
    reloaded = lawsynth.Project.load(workspace)
    print(f"reloaded {len(reloaded)} world(s) from disk: {reloaded.names()}")
    osc_eqs = dict(reloaded.get("oscillator").equations())
    print(f"  oscillator laws after reload: {osc_eqs}")

    # 4. Export the whole workspace as one shareable archive file.
    archive = workspace / "workspace.lswork"
    reloaded.export(archive)
    print(f"\nexported archive: {archive}  ({archive.stat().st_size} bytes)")

    # 5. Re-import the archive into a FRESH directory and confirm the round-trip.
    fresh = Path(tempfile.mkdtemp(prefix="lawsynth_project_import_"))
    imported = lawsynth.Project.import_archive(archive, fresh)
    print(f"re-imported into fresh dir: {fresh}")
    print(f"  worlds: {imported.names()}")

    print("\nround-trip integrity (content hashes must match):")
    all_ok = True
    for name in reloaded.names():
        before = reloaded.entry(name).world_hash
        after = imported.entry(name).world_hash
        equations_match = dict(reloaded.get(name).equations()) == dict(imported.get(name).equations())
        ok = (before == after) and equations_match
        all_ok = all_ok and ok
        print(f"  {name:<12} hash {before[:12]}… -> {after[:12]}…  "
              f"match={before == after}  equations_match={equations_match}")

    print(f"\nround-trip {'VERIFIED — every world survived byte-for-byte.' if all_ok else 'FAILED.'}")
    assert all_ok


if __name__ == "__main__":
    main()
