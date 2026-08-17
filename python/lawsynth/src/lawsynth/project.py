"""A local workspace for organizing and sharing discovered worlds.

A single discovery is a portable ``.lsworld`` bundle; real work quickly produces
many of them. :class:`Project` is the SDK-side workspace that keeps a collection
of discovered worlds navigable: register a world under a name with tags and a
note, list/search/get/remove them, persist the whole set to a directory, and
share it as one archive.

**CLI interoperability.** A project persists to the *same* format the
``lawsynth library`` CLI uses: a tab-separated ``library.tsv`` index (name, path,
tags, description, world hash, data hash, data columns, config) plus the
``.lsworld`` bundle files it references. Point a :class:`Project` at
``~/.lawsynth`` and it reads and writes the very index the CLI maintains, so both
tools share one workspace. The index is written deterministically (entries sorted
by name), and legacy four-column indices still load.

**Sharing.** :meth:`Project.export` packs the entire workspace — a JSON manifest
with a SHA-256 for every world, followed by the raw world bytes — into one
self-describing container file. :meth:`Project.import_archive` verifies each hash
and reconstructs the workspace in a fresh directory, so a colleague gets a
byte-identical set of worlds.
"""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, replace
from os import PathLike
from pathlib import Path
from typing import Iterator, Sequence

from .errors import LawSynthError, ValidationError

__all__ = ["Project", "ProjectEntry"]

# Index file name and header — byte-compatible with the CLI `library` command.
_INDEX_NAME = "library.tsv"
_HEADER = "name\tpath\ttags\tdescription\tworld_hash\tdata_hash\tdata_columns\tconfig"

# Archive container magic + version. The archive is: a magic line, a one-line
# JSON manifest, then the concatenated raw world bytes in manifest order.
_ARCHIVE_MAGIC = "LAWSYNTH-WORKSPACE\tv1"


def _sha256_hex(data: bytes) -> str:
    """Lowercase SHA-256 hex digest — matches the CLI bundle content hash."""
    return hashlib.sha256(data).hexdigest()


def _sanitize(value: str) -> str:
    """Strip tabs/newlines so a value survives a round-trip through the TSV."""
    return value.replace("\t", " ").replace("\n", " ").replace("\r", " ")


def _slug(name: str) -> str:
    """A filesystem-safe bundle stem derived deterministically from ``name``."""
    safe = "".join(char if (char.isalnum() or char in "-_.") else "_" for char in name)
    return safe or "world"


def _parse_tags(value: str) -> tuple[str, ...]:
    return tuple(tag.strip() for tag in value.split(",") if tag.strip())


# --------------------------------------------------------------------------- #
# Entry                                                                       #
# --------------------------------------------------------------------------- #


@dataclass(frozen=True, slots=True)
class ProjectEntry:
    """One registered world plus its provenance, mirroring a library.tsv row."""

    name: str
    tags: tuple[str, ...] = ()
    note: str = ""
    world_hash: str = ""
    data_hash: str = ""
    data_columns: str = ""
    config_summary: str = ""
    path: str | None = None  # absolute path to the .lsworld bundle once persisted

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "tags": list(self.tags),
            "note": self.note,
            "world_hash": self.world_hash,
            "data_hash": self.data_hash,
            "data_columns": self.data_columns,
            "config": self.config_summary,
            "path": self.path,
        }


# --------------------------------------------------------------------------- #
# Provenance extraction from a Study / DiscoveryResult / native World          #
# --------------------------------------------------------------------------- #


def _config_summary(config: object) -> str:
    if config is None:
        return ""
    parts = []
    for field in ("polynomial_degree", "threshold", "solver", "derivative_method"):
        value = getattr(config, field, None)
        if value is not None:
            parts.append(f"{field}={value}")
    return _sanitize(" ".join(parts))


def _resolve_source(source: object) -> tuple[object, str, str]:
    """Return ``(native_world, data_columns, config_summary)`` for any source.

    Accepts a :class:`~lawsynth.study.Study`, a
    :class:`~lawsynth.study.DiscoveryResult`, or a native ``World``. A Study must
    be discovered first.
    """
    # DiscoveryResult / Study expose a `.world`; a native World exposes
    # `.equations()`. Import lazily to keep this module native-free until needed.
    world = getattr(source, "world", None)
    if world is None and hasattr(source, "equations"):
        world = source  # already a native world
    if world is None:
        raise ValidationError(
            "add() needs a discovered Study, a DiscoveryResult, or a native World; "
            "call discover() on a Study first"
        )
    if not hasattr(world, "save") or not hasattr(world, "equations"):
        raise ValidationError("resolved object is not a persistable native World")

    data_columns = ""
    dataset = getattr(source, "dataset", None)
    states = getattr(source, "states", None)
    if dataset is not None and getattr(dataset, "columns", None):
        data_columns = ",".join(dataset.columns.keys())
    elif states:
        data_columns = ",".join(states)

    config_summary = _config_summary(getattr(source, "_config", None))
    return world, _sanitize(data_columns), config_summary


# --------------------------------------------------------------------------- #
# Project                                                                      #
# --------------------------------------------------------------------------- #


class Project:
    """A directory-backed workspace of named, discovered worlds."""

    __slots__ = ("_dir", "_entries", "_worlds")

    def __init__(self, directory: str | PathLike[str]) -> None:
        self._dir = Path(directory)
        # Ordered registry; insertion order is preserved for display, the index
        # file is always written sorted by name for deterministic, CLI-matching output.
        self._entries: dict[str, ProjectEntry] = {}
        # Live native worlds pending or loaded, keyed by name (None until loaded).
        self._worlds: dict[str, object | None] = {}

    # -- properties --------------------------------------------------------- #

    @property
    def directory(self) -> Path:
        return self._dir

    @property
    def index_path(self) -> Path:
        return self._dir / _INDEX_NAME

    def __len__(self) -> int:
        return len(self._entries)

    def __contains__(self, name: object) -> bool:
        return name in self._entries

    def __iter__(self) -> Iterator[ProjectEntry]:
        return iter(self.list())

    def __repr__(self) -> str:
        return f"Project(dir={str(self._dir)!r}, worlds={len(self._entries)})"

    # -- mutation ----------------------------------------------------------- #

    def add(
        self,
        name: str,
        study_or_world: object,
        *,
        tags: Sequence[str] = (),
        note: str = "",
    ) -> ProjectEntry:
        """Register a discovered world under ``name`` with optional metadata.

        ``study_or_world`` may be a discovered :class:`Study`, a
        :class:`DiscoveryResult`, or a native ``World``. The world's content hash
        and provenance are captured on :meth:`save`. Names are unique within a
        project; adding a duplicate name raises.
        """
        self._validate_name(name)
        if name in self._entries:
            raise ValidationError(f"project already has a world named {name!r} (remove it first)")
        world, data_columns, config_summary = _resolve_source(study_or_world)
        entry = ProjectEntry(
            name=name,
            tags=tuple(str(tag).strip() for tag in tags if str(tag).strip()),
            note=_sanitize(note),
            data_columns=data_columns,
            config_summary=config_summary,
        )
        self._entries[name] = entry
        self._worlds[name] = world
        return entry

    def remove(self, name: str, *, delete_bundle: bool = False) -> None:
        """Remove ``name`` from the project.

        By default only the index entry is dropped; pass ``delete_bundle=True`` to
        also delete the persisted ``.lsworld`` file from the directory.
        """
        entry = self._entries.pop(name, None)
        if entry is None:
            raise ValidationError(f"no world named {name!r} in project")
        self._worlds.pop(name, None)
        if delete_bundle and entry.path:
            bundle = Path(entry.path)
            if bundle.exists():
                bundle.unlink()

    # -- access ------------------------------------------------------------- #

    def list(self) -> list[ProjectEntry]:
        """All entries, sorted by name (matching the persisted index order)."""
        return [self._entries[name] for name in sorted(self._entries)]

    def names(self) -> list[str]:
        return sorted(self._entries)

    def entry(self, name: str) -> ProjectEntry:
        """The metadata entry for ``name`` (no world loading)."""
        try:
            return self._entries[name]
        except KeyError:
            raise ValidationError(f"no world named {name!r} in project") from None

    def get(self, name: str) -> object:
        """Return the native ``World`` for ``name``, loading its bundle if needed."""
        entry = self.entry(name)
        world = self._worlds.get(name)
        if world is not None:
            return world
        bundle = self._resolve_bundle_path(entry)
        if bundle is None:
            raise LawSynthError(
                f"world {name!r} has no loadable bundle; save the project first"
            )
        try:
            from ._native import World
        except ImportError as error:  # pragma: no cover - native optional
            raise LawSynthError("the lawsynth native extension is unavailable") from error
        world = World.load(str(bundle))
        self._worlds[name] = world
        return world

    def search(self, query: str) -> list[ProjectEntry]:
        """Case-insensitive substring match across name, tags, and note."""
        needle = query.lower()
        return [
            entry for entry in self.list()
            if needle in entry.name.lower()
            or needle in entry.note.lower()
            or any(needle in tag.lower() for tag in entry.tags)
        ]

    # -- persistence -------------------------------------------------------- #

    def save(self) -> Path:
        """Write every world's bundle and the ``library.tsv`` index to the directory.

        Bundles for in-memory worlds are written as ``<name>.lsworld``; entries
        loaded from an existing index keep their recorded bundle path. Each entry's
        world hash is (re)computed from the persisted bundle bytes. Returns the
        index path.
        """
        self._dir.mkdir(parents=True, exist_ok=True)
        for name in list(self._entries):
            entry = self._entries[name]
            world = self._worlds.get(name)
            if world is not None:
                bundle = self._dir / f"{_slug(name)}.lsworld"
                world.save(str(bundle))
                digest = _sha256_hex(bundle.read_bytes())
                self._entries[name] = replace(entry, path=str(bundle.resolve()), world_hash=digest)
            else:
                # No live world: keep the existing bundle, refresh hash if readable.
                bundle = self._resolve_bundle_path(entry)
                if bundle is not None and bundle.exists():
                    digest = _sha256_hex(bundle.read_bytes())
                    self._entries[name] = replace(
                        entry, path=str(bundle.resolve()), world_hash=digest
                    )
        self._write_index()
        return self.index_path

    @classmethod
    def load(cls, directory: str | PathLike[str]) -> "Project":
        """Load a workspace from a directory holding a ``library.tsv`` index.

        Interoperable with ``lawsynth library``: an index written by the CLI loads
        here, and vice versa. Worlds are loaded lazily on first :meth:`get`.
        """
        project = cls(directory)
        index = project.index_path
        if not index.exists():
            return project
        text = index.read_text(encoding="utf-8")
        for line in text.splitlines():
            if not line or line.startswith("name\tpath\t"):
                continue  # skip blanks and header (old four-col or new eight-col)
            fields = line.split("\t")
            if len(fields) < 2:
                continue
            name = fields[0]
            entry = ProjectEntry(
                name=name,
                path=fields[1] or None,
                tags=_parse_tags(fields[2]) if len(fields) > 2 else (),
                note=fields[3] if len(fields) > 3 else "",
                world_hash=fields[4] if len(fields) > 4 else "",
                data_hash=fields[5] if len(fields) > 5 else "",
                data_columns=fields[6] if len(fields) > 6 else "",
                config_summary=fields[7] if len(fields) > 7 else "",
            )
            project._entries[name] = entry
            project._worlds[name] = None
        return project

    def _write_index(self) -> None:
        lines = [_HEADER]
        for name in sorted(self._entries):
            entry = self._entries[name]
            lines.append("\t".join([
                entry.name,
                entry.path or "",
                ",".join(entry.tags),
                entry.note,
                entry.world_hash,
                entry.data_hash,
                entry.data_columns,
                entry.config_summary,
            ]))
        self.index_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    def _resolve_bundle_path(self, entry: ProjectEntry) -> Path | None:
        """Find the entry's bundle robustly: recorded path, then dir fallbacks.

        Supports moving a workspace (absolute paths in the index no longer exist)
        and CLI-written indices (paths recorded relative to the CLI's cwd).
        """
        candidates: list[Path] = []
        if entry.path:
            candidates.append(Path(entry.path))
            candidates.append(self._dir / Path(entry.path).name)
        candidates.append(self._dir / f"{_slug(entry.name)}.lsworld")
        for candidate in candidates:
            if candidate.exists():
                return candidate
        return None

    # -- archive (single-file share) --------------------------------------- #

    def export(self, archive_path: str | PathLike[str]) -> Path:
        """Pack the whole workspace into one self-describing archive file.

        Container format (documented, deterministic):

        1. A magic line ``LAWSYNTH-WORKSPACE\\tv1``.
        2. A one-line JSON manifest: ``{"entries": [ {name, tags, note,
           data_columns, config, world_hash, size}, ... ]}`` — entries sorted by
           name, ``world_hash`` a SHA-256 of the world bytes, ``size`` their byte
           length.
        3. The raw world bytes, concatenated in manifest order.

        Worlds must be persisted (call :meth:`save` first) so their bytes and
        hashes are available. Returns the archive path.
        """
        target = Path(archive_path)
        manifest_entries: list[dict[str, object]] = []
        payloads: list[bytes] = []
        for name in sorted(self._entries):
            entry = self._entries[name]
            bundle = self._resolve_bundle_path(entry)
            if bundle is None:
                raise LawSynthError(
                    f"cannot export {name!r}: no persisted bundle; call save() first"
                )
            data = bundle.read_bytes()
            digest = _sha256_hex(data)
            payloads.append(data)
            manifest_entries.append({
                "name": entry.name,
                "tags": list(entry.tags),
                "note": entry.note,
                "data_columns": entry.data_columns,
                "data_hash": entry.data_hash,
                "config": entry.config_summary,
                "world_hash": digest,
                "size": len(data),
            })
        manifest = json.dumps({"entries": manifest_entries}, sort_keys=True, separators=(",", ":"))
        header = f"{_ARCHIVE_MAGIC}\n{manifest}\n".encode("utf-8")
        target.write_bytes(header + b"".join(payloads))
        return target

    @classmethod
    def import_archive(cls, archive_path: str | PathLike[str], directory: str | PathLike[str]) -> "Project":
        """Reconstruct a workspace from an archive into a fresh ``directory``.

        Parses the manifest, slices out each world's bytes, and verifies its
        SHA-256 against the manifest before writing the bundle. Then materialises a
        ``library.tsv`` index and returns a loaded :class:`Project`. Raises if any
        integrity check fails.
        """
        source = Path(archive_path)
        raw = source.read_bytes()
        # Split the two header lines (magic + manifest) from the binary payload.
        first_nl = raw.find(b"\n")
        if first_nl < 0:
            raise ValidationError("archive is truncated: missing magic line")
        magic = raw[:first_nl].decode("utf-8", "replace")
        if magic != _ARCHIVE_MAGIC:
            raise ValidationError(f"unrecognized workspace archive header: {magic!r}")
        second_nl = raw.find(b"\n", first_nl + 1)
        if second_nl < 0:
            raise ValidationError("archive is truncated: missing manifest line")
        try:
            manifest = json.loads(raw[first_nl + 1:second_nl].decode("utf-8"))
        except json.JSONDecodeError as error:
            raise ValidationError(f"archive manifest is not valid JSON: {error}") from error
        entries = manifest.get("entries")
        if not isinstance(entries, list):
            raise ValidationError("archive manifest has no entries list")

        target_dir = Path(directory)
        target_dir.mkdir(parents=True, exist_ok=True)
        project = cls(target_dir)
        offset = second_nl + 1
        for item in entries:
            name = item["name"]
            size = int(item["size"])
            expected = item["world_hash"]
            data = raw[offset:offset + size]
            offset += size
            if len(data) != size:
                raise ValidationError(f"archive truncated while reading world {name!r}")
            digest = _sha256_hex(data)
            if digest != expected:
                raise ValidationError(
                    f"integrity check failed for {name!r}: expected {expected[:12]}…, "
                    f"got {digest[:12]}…"
                )
            bundle = target_dir / f"{_slug(name)}.lsworld"
            bundle.write_bytes(data)
            entry = ProjectEntry(
                name=name,
                tags=tuple(item.get("tags", ())),
                note=item.get("note", ""),
                world_hash=digest,
                data_hash=item.get("data_hash", ""),
                data_columns=item.get("data_columns", ""),
                config_summary=item.get("config", ""),
                path=str(bundle.resolve()),
            )
            project._entries[name] = entry
            project._worlds[name] = None
        project._write_index()
        return project

    # -- validation --------------------------------------------------------- #

    @staticmethod
    def _validate_name(name: str) -> None:
        if not name or not isinstance(name, str):
            raise ValidationError("world name must be a non-empty string")
        if "\t" in name or "\n" in name or name.startswith("--"):
            raise ValidationError(f"invalid world name {name!r}")
