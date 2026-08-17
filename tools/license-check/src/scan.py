"""Scan dependency manifests into a normalized dependency inventory.

Supported inputs:

* ``Cargo.lock``      — TOML; ``[[package]]`` name + version. Cargo lockfiles do
  not record licenses, so entries are emitted with ``license = None`` (surfaced
  as "needs review" by the policy step) unless resolved from a side table.
* ``package.json``    — the package's own ``license`` (SPDX string).
* ``*.sbom.json`` /   — a simple inventory array of
  ``inventory.json``      ``{"name", "version", "license"}`` objects, matching the
                        output of tools like ``cargo-license`` or an SBOM export.

The scanner is offline and only reads files it is handed.
"""

from __future__ import annotations

import json
import tomllib
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Dependency:
    name: str
    version: str
    license: str | None
    source: str  # the manifest file the entry came from

    def key(self) -> tuple[str, str]:
        return (self.name, self.version)


def scan_path(path: Path) -> list[Dependency]:
    """Dispatch a single manifest to the appropriate parser by file name."""
    name = path.name
    if name == "Cargo.lock":
        return _scan_cargo_lock(path)
    if name == "package.json":
        return _scan_package_json(path)
    if name.endswith(".json"):
        return _scan_inventory(path)
    raise ValueError(f"unsupported manifest: {path}")


def scan_paths(paths: list[Path]) -> list[Dependency]:
    """Scan several manifests and return a deduplicated, sorted inventory."""
    collected: dict[tuple[str, str], Dependency] = {}
    for path in paths:
        for dependency in scan_path(path):
            # A concrete license wins over an unknown one for the same package.
            existing = collected.get(dependency.key())
            if existing is None or (existing.license is None and dependency.license is not None):
                collected[dependency.key()] = dependency
    return sorted(collected.values(), key=lambda dep: (dep.name.lower(), dep.version))


def _scan_cargo_lock(path: Path) -> list[Dependency]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    packages = data.get("package", [])
    return [
        Dependency(
            name=str(entry["name"]),
            version=str(entry.get("version", "0.0.0")),
            license=entry.get("license"),
            source=path.name,
        )
        for entry in packages
    ]


def _scan_package_json(path: Path) -> list[Dependency]:
    data = json.loads(path.read_text(encoding="utf-8"))
    return [
        Dependency(
            name=str(data.get("name", path.parent.name)),
            version=str(data.get("version", "0.0.0")),
            license=data.get("license"),
            source=path.name,
        )
    ]


def _scan_inventory(path: Path) -> list[Dependency]:
    data = json.loads(path.read_text(encoding="utf-8"))
    entries = data.get("packages", data) if isinstance(data, dict) else data
    result: list[Dependency] = []
    for entry in entries:
        result.append(
            Dependency(
                name=str(entry["name"]),
                version=str(entry.get("version", "0.0.0")),
                license=entry.get("license") or entry.get("licenses"),
                source=path.name,
            )
        )
    return result
