"""Render a bundle inspection as deterministic text or JSON."""

from __future__ import annotations

import json
from dataclasses import dataclass

from archive import Archive
from checksum import ChecksumReport
from manifest import World


@dataclass(frozen=True)
class Inspection:
    path: str
    entries: dict[str, int]
    warnings: tuple[str, ...]
    checksums_ok: bool
    checksum_failures: tuple[str, ...]
    world: World

    @property
    def ok(self) -> bool:
        return self.checksums_ok and not self.checksum_failures


def build_inspection(archive: Archive, checksums: ChecksumReport, world: World) -> Inspection:
    return Inspection(
        path=str(archive.path),
        entries={name: len(data) for name, data in sorted(archive.entries.items())},
        warnings=archive.warnings,
        checksums_ok=checksums.ok,
        checksum_failures=tuple(line.path for line in checksums.failures),
        world=world,
    )


def to_dict(inspection: Inspection) -> dict[str, object]:
    world = inspection.world
    return {
        "path": inspection.path,
        "ok": inspection.ok,
        "entries": inspection.entries,
        "warnings": list(inspection.warnings),
        "integrity": {
            "checksums_ok": inspection.checksums_ok,
            "failures": list(inspection.checksum_failures),
        },
        "world": {
            "kind": world.kind,
            "state_count": world.state_count,
            "variables": [
                {"id": v.id, "role": v.role, "unit": v.unit} for v in world.variables
            ],
            "parameters": [
                {"id": p.id, "value": p.value, "unit": p.unit} for p in world.parameters
            ],
            "laws": [{"target": law.target, "expression": law.expression} for law in world.laws],
        },
    }


def to_json(inspection: Inspection) -> str:
    return json.dumps(to_dict(inspection), indent=2, sort_keys=True)


def to_text(inspection: Inspection) -> str:
    world = inspection.world
    lines = [
        f"bundle: {inspection.path}",
        f"status: {'OK' if inspection.ok else 'INVALID'}",
        "",
        "entries:",
    ]
    for name, size in inspection.entries.items():
        lines.append(f"  {name} ({size} bytes)")
    lines.append("")
    lines.append(f"integrity: checksums {'verified' if inspection.checksums_ok else 'FAILED'}")
    for failure in inspection.checksum_failures:
        lines.append(f"  mismatch: {failure}")
    for warning in inspection.warnings:
        lines.append(f"  warning: {warning}")
    lines.append("")
    lines.append(f"world: {world.kind} ({world.state_count} states, {len(world.variables)} variables)")
    for variable in world.variables:
        unit = f" [{variable.unit}]" if variable.unit else ""
        lines.append(f"  var {variable.id}: {variable.role}{unit}")
    for parameter in world.parameters:
        unit = f" [{parameter.unit}]" if parameter.unit else ""
        lines.append(f"  param {parameter.id} = {parameter.value:.12g}{unit}")
    for law in world.laws:
        lines.append(f"  law d/dt {law.target} = {law.expression}")
    return "\n".join(lines)
