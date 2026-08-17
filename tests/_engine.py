#!/usr/bin/env python3
"""Shared location and invocation helpers for the compiled LawSynth engine.

Every cross-language, end-to-end, and conformance suite drives the real
``lawsynth`` CLI through this module. The engine is built once with
``cargo build -p lawsynth-cli`` and reused from ``target/debug/lawsynth``.
When the prebuilt binary is missing we fall back to ``cargo run`` so a case is
still executed against the real engine rather than an emulation; when neither
the binary nor ``cargo`` is available the suites skip cleanly with an explicit
message instead of silently passing.
"""

from __future__ import annotations

import functools
import shutil
import subprocess
from pathlib import Path

# ``tests/_engine.py`` -> parents[1] is the repository root.
ROOT = Path(__file__).resolve().parents[1]

_PROFILES = ("debug", "release")
_BINARY_NAMES = ("lawsynth", "lawsynth.exe")


def prebuilt_binary() -> Path | None:
    """Return the compiled ``lawsynth`` binary if one is already on disk."""
    for profile in _PROFILES:
        for name in _BINARY_NAMES:
            candidate = ROOT / "target" / profile / name
            if candidate.exists():
                return candidate
    return None


@functools.lru_cache(maxsize=1)
def ensure_engine() -> Path | None:
    """Locate the engine binary, building it once when necessary.

    Returns the path to the binary, or ``None`` when the engine cannot be
    produced offline (missing ``cargo`` toolchain or a failed build). Callers
    translate ``None`` into an explicit skip.
    """
    existing = prebuilt_binary()
    if existing is not None:
        return existing
    if shutil.which("cargo") is None:
        return None
    result = subprocess.run(
        ["cargo", "build", "-p", "lawsynth-cli"],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        return None
    return prebuilt_binary()


def cli(*arguments: str) -> subprocess.CompletedProcess[str]:
    """Invoke the real engine, preferring the prebuilt binary for speed."""
    binary = prebuilt_binary()
    if binary is not None:
        command: list[str] = [str(binary), *arguments]
    else:
        command = ["cargo", "run", "--quiet", "-p", "lawsynth-cli", "--bin", "lawsynth", "--", *arguments]
    return subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
