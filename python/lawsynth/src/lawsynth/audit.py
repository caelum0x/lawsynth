"""Append-only, tamper-evident local **audit log** for governance (P9).

Every governance-relevant action — submission, evaluation, approval, edit,
export, share — appends an immutable entry to an audit log. The log is a hash
chain: each entry carries the prior entry's digest, so a consumer can detect a
gap (a missing entry) or an alteration (any changed field) by recomputing the
chain. Ordering is by a strictly increasing **content ordinal**, never a wall
clock, so a log is deterministic and reproducible.

``AuditLog(path)`` persists to a JSON-lines file and reloads it on construction;
``append`` adds an entry, ``verify`` recomputes and validates the whole chain.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping

from ._content import content_digest

__all__ = ["AuditEntry", "AuditLog"]

# The genesis digest chained into the very first entry.
_GENESIS = "0" * 64


def _entry_digest(
    ordinal: int, actor: str, action: str, details: Mapping[str, object], prev: str
) -> str:
    """Digest binding an entry's content to its predecessor (the chain link)."""
    return content_digest(
        {
            "ordinal": ordinal,
            "actor": actor,
            "action": action,
            "details": dict(details),
            "prev": prev,
        }
    )


@dataclass(frozen=True, slots=True)
class AuditEntry:
    """One immutable audit event in the chain."""

    ordinal: int
    actor: str
    action: str
    details: Mapping[str, object]
    prev_digest: str
    digest: str

    def to_dict(self) -> dict[str, object]:
        return {
            "ordinal": self.ordinal,
            "actor": self.actor,
            "action": self.action,
            "details": dict(self.details),
            "prev_digest": self.prev_digest,
            "digest": self.digest,
        }

    @classmethod
    def from_dict(cls, data: Mapping[str, object]) -> "AuditEntry":
        return cls(
            ordinal=int(data["ordinal"]),  # type: ignore[arg-type]
            actor=str(data["actor"]),
            action=str(data["action"]),
            details=dict(data.get("details", {})),  # type: ignore[arg-type]
            prev_digest=str(data["prev_digest"]),
            digest=str(data["digest"]),
        )


class AuditLog:
    """An append-only, tamper-evident, file-backed audit log.

    The in-memory entries are the source of truth; ``append`` recomputes the next
    chain link and (when a ``path`` is set) persists the whole log atomically as
    JSON lines. ``verify`` detects any gap or alteration.
    """

    __slots__ = ("_entries", "_path")

    def __init__(self, path: str | Path | None = None) -> None:
        self._entries: tuple[AuditEntry, ...] = ()
        self._path = Path(path) if path is not None else None
        if self._path is not None and self._path.exists():
            self._entries = tuple(self._read(self._path))

    # -- accessors ---------------------------------------------------------- #

    @property
    def entries(self) -> tuple[AuditEntry, ...]:
        return self._entries

    @property
    def head(self) -> str:
        """The digest of the last entry, or the genesis digest if empty."""
        return self._entries[-1].digest if self._entries else _GENESIS

    def __len__(self) -> int:
        return len(self._entries)

    # -- append ------------------------------------------------------------- #

    def append(self, actor: str, action: str, **details: object) -> AuditEntry:
        """Append an event; the ordinal is the next integer (no wall clock)."""
        if not actor:
            raise ValueError("audit entry requires a non-empty actor")
        if not action:
            raise ValueError("audit entry requires a non-empty action")
        ordinal = len(self._entries)
        prev = self.head
        digest = _entry_digest(ordinal, actor, action, details, prev)
        entry = AuditEntry(
            ordinal=ordinal,
            actor=actor,
            action=action,
            details=dict(details),
            prev_digest=prev,
            digest=digest,
        )
        self._entries = (*self._entries, entry)
        if self._path is not None:
            self._persist(self._path, self._entries)
        return entry

    # -- integrity ---------------------------------------------------------- #

    def verify(self) -> bool:
        """True iff the chain is intact: ordinals, digests and links all hold.

        Detects a **gap** (ordinals not strictly 0,1,2,…), an **alteration** (any
        field changed, breaking the recomputed digest), and a **broken link** (an
        entry whose ``prev_digest`` does not match its predecessor's digest).
        """
        return self._verify(self._entries)

    @staticmethod
    def _verify(entries: tuple[AuditEntry, ...]) -> bool:
        prev = _GENESIS
        for expected_ordinal, entry in enumerate(entries):
            if entry.ordinal != expected_ordinal:
                return False  # gap or reordering
            if entry.prev_digest != prev:
                return False  # broken link
            recomputed = _entry_digest(
                entry.ordinal, entry.actor, entry.action, entry.details, entry.prev_digest
            )
            if recomputed != entry.digest:
                return False  # alteration
            prev = entry.digest
        return True

    @classmethod
    def verify_file(cls, path: str | Path) -> bool:
        """Verify a persisted audit log on disk without mutating anything."""
        entries = tuple(cls._read(Path(path)))
        return cls._verify(entries)

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "valid": self.verify(),
            "count": len(self._entries),
            "head": self.head,
            "entries": [entry.to_dict() for entry in self._entries],
        }

    def to_json(self, *, indent: int | None = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent, sort_keys=True)

    def to_text(self) -> str:
        status = "intact" if self.verify() else "TAMPERED"
        lines = [f"Audit log ({len(self._entries)} entries, {status}):"]
        for entry in self._entries:
            detail = ", ".join(f"{k}={v}" for k, v in sorted(entry.details.items()))
            suffix = f" [{detail}]" if detail else ""
            lines.append(
                f"  #{entry.ordinal} {entry.actor} · {entry.action}{suffix} "
                f"→ {entry.digest[:12]}"
            )
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return f"AuditLog(entries={len(self._entries)}, valid={self.verify()})"

    # -- persistence -------------------------------------------------------- #

    @staticmethod
    def _read(path: Path) -> list[AuditEntry]:
        entries: list[AuditEntry] = []
        for line in path.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if not line:
                continue
            entries.append(AuditEntry.from_dict(json.loads(line)))
        return entries

    @staticmethod
    def _persist(path: Path, entries: tuple[AuditEntry, ...]) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        payload = "\n".join(
            json.dumps(entry.to_dict(), sort_keys=True) for entry in entries
        )
        path.write_text(payload + ("\n" if payload else ""), encoding="utf-8")
