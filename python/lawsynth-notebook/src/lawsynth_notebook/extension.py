"""Explicit optional frontend-extension metadata, without installation hooks."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class ExtensionSpec:
    name: str = "lawsynth-notebook"
    version: str = "0.1.0"
    requires_jupyterlab: str = ">=4"


def extension_spec() -> dict[str, str]:
    spec = ExtensionSpec()
    return {"name": spec.name, "version": spec.version, "requires_jupyterlab": spec.requires_jupyterlab}
