#!/usr/bin/env python3
"""Profile a time-series dataset before discovery — text + themed HTML.

Run it (from the repository root)::

    PYTHONPATH=python/lawsynth/src python3 python/lawsynth/examples/profile_dataset.py

Everything here is deterministic, offline, and standard-library only: it never
touches the native engine. It builds a small synthetic dataset (with one clean
column, one constant/degenerate column, and — via a CSV — one column with a
missing value and a slightly irregular time step), profiles it, prints the text
report, and confirms the notebook HTML carries the LawSynth brand palette.
"""

from __future__ import annotations

import csv
import math
import tempfile
from pathlib import Path

import lawsynth
from lawsynth import Dataset


def _clean_dataset() -> Dataset:
    """A damped sine `x` and its velocity `v`, plus a constant `bias` column."""
    time = [round(0.1 * i, 3) for i in range(60)]
    x = [math.exp(-0.15 * t) * math.sin(1.3 * t) for t in time]
    v = [
        math.exp(-0.15 * t) * (1.3 * math.cos(1.3 * t) - 0.15 * math.sin(1.3 * t))
        for t in time
    ]
    bias = [2.0 for _ in time]  # degenerate/constant column
    return Dataset.from_columns(time, {"x": x, "v": v, "bias": bias})


def _write_messy_csv(path: Path) -> None:
    """Same series, but with a missing `v` cell and one irregular time step."""
    dataset = _clean_dataset()
    times = list(dataset.time)
    times[40] = times[40] + 0.07  # perturb one timestamp -> irregular sampling
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["time", "x", "v", "bias"])
        for i, t in enumerate(times):
            v_cell = "" if i == 25 else f"{dataset.columns['v'][i]:.10f}"  # one missing value
            writer.writerow([f"{t:.6f}", f"{dataset.columns['x'][i]:.10f}", v_cell, "2.0"])


def main() -> None:
    print("=" * 72)
    print("1) Profile a validated Dataset (via lawsynth.profile / Study.profile)")
    print("=" * 72)
    dataset = _clean_dataset()
    profile = lawsynth.profile(dataset, name="damped_oscillator")
    print(profile.to_text())

    # Study.profile() returns the same product for the study's dataset.
    study = lawsynth.Study.from_dataset(dataset, state=["x", "v"], name="damped_oscillator")
    study_profile = study.profile()
    assert study_profile.to_dict()["rows"] == profile.to_dict()["rows"]
    print("\nStudy.profile() agrees with lawsynth.profile():",
          study_profile.rows, "rows,",
          len(study_profile.warnings), "warnings.")

    print("\n" + "=" * 72)
    print("2) Profile a messy CSV (missing value + irregular sampling)")
    print("=" * 72)
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth_profile_"))
    csv_path = workdir / "messy.csv"
    _write_messy_csv(csv_path)
    csv_profile = lawsynth.profile(csv_path, time="time", state=["x", "v", "bias"])
    print(csv_profile.to_text())

    # Structured access for pipelines.
    v_column = csv_profile.column("v")
    print(f"\nprogrammatic: column 'v' has {v_column.missing} missing value(s), "
          f"mean={v_column.mean:.4f}")
    print("time regular?", csv_profile.time.regular,
          "| monotonic?", csv_profile.time.monotonic)

    print("\n" + "=" * 72)
    print("3) Confirm the notebook HTML carries the brand palette")
    print("=" * 72)
    # The clean profile has a regular, monotonic axis, so its HTML exercises the
    # success token too; the messy profile's HTML surfaces the warning wash.
    clean_html = profile._repr_html_()
    messy_html = csv_profile._repr_html_()
    brand = {
        "ink #18201d": "#18201d",
        "accentSoft #e5c3b4": "#e5c3b4",
        "surface #fffdf7": "#fffdf7",
        "line #c8c6ba": "#c8c6ba",
        "muted #59635e": "#59635e",
        "accent #b54b2a": "#b54b2a",
        "success #2f6f4f": "#2f6f4f",
        "serif Georgia": "Georgia",
        "sans Inter": "Inter, system-ui",
        "mono ui-monospace": "ui-monospace",
    }
    for label, needle in brand.items():
        assert needle in clean_html, f"brand token missing from HTML: {label}"
        print(f"  ok  {label:<26} present")
    # The messy profile's warning callout uses the accent wash + warning hue.
    assert "#e5c3b4" in messy_html and "#b8822a" in messy_html
    html_path = workdir / "profile.html"
    html_path.write_text(
        f"<!doctype html><meta charset='utf-8'><title>profile</title>{messy_html}",
        encoding="utf-8",
    )
    print(f"\nclean _repr_html_ is {len(clean_html)} bytes; "
          f"messy report written to {html_path}")
    print("done — deterministic, offline, standard-library only.")


if __name__ == "__main__":
    main()
