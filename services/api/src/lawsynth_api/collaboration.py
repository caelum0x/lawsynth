"""Collaboration primitives: membership, revision lineage, annotations, review, merge.

This module is the *domain core* of the P6 collaboration boundary
(``specs/collaboration/README.md``).  It owns pure, deterministic, offline data
structures and validation -- no HTTP, no authentication -- exactly like the other
API-side, tenant-scoped, in-process stores this process composes (``EventBus``,
``ArtifactOwnership``, ``MeteringLog``).  The transport wiring (auth, scope, role
enforcement, event emission) lives in :mod:`collaboration_routes`.

Everything here is keyed by ``(organization_id, resource_id)`` so the tenant
isolation the rest of the service guarantees holds for collaboration state too:
a caller for tenant A can never observe or mutate tenant B's memberships,
revisions, or annotations.
"""

from __future__ import annotations

import hashlib
import json
from copy import deepcopy
from datetime import UTC, datetime
from threading import RLock
from typing import Mapping

from lawsynth_server.errors import ConflictError, NotFoundError, ValidationError

# -- Roles ------------------------------------------------------------------- #

OWNER = "owner"
EDITOR = "editor"
VIEWER = "viewer"
ROLES = (OWNER, EDITOR, VIEWER)
# Ordered from most to least privileged; index gives a comparable rank.
_RANK = {OWNER: 2, EDITOR: 1, VIEWER: 0}


def role_can_write(role: str | None) -> bool:
    """True when ``role`` may add/update/remove worlds and annotate (editor+)."""

    return role in (OWNER, EDITOR)


def _now() -> str:
    return datetime.now(UTC).isoformat()


# -- Content hashing --------------------------------------------------------- #


def world_content_hash(world: Mapping[str, object]) -> str:
    """Return the SHA-256 of a world's canonical declarative content.

    This is the content hash of the ``.lsworld`` model: the executable subset
    (states, controls, parameters, equations) serialized canonically so that two
    worlds with identical content -- regardless of name or storage id -- hash
    identically.  That equality is what lets :func:`merge_indexes` tell a benign
    duplicate from a genuine content conflict.
    """

    def _floats(value: object) -> dict[str, float]:
        if not isinstance(value, Mapping):
            return {}
        return {str(name): float(number) for name, number in value.items()}

    def _strs(value: object) -> dict[str, str]:
        if not isinstance(value, Mapping):
            return {}
        return {str(name): str(expr) for name, expr in value.items()}

    def _list(value: object) -> list[str]:
        if not isinstance(value, (list, tuple)):
            return []
        return [str(item) for item in value]

    spec = {
        "states": _list(world.get("states")),
        "controls": _list(world.get("controls")),
        "parameters": _floats(world.get("parameters")),
        "equations": _strs(world.get("equations")),
    }
    canonical = json.dumps(spec, sort_keys=True, separators=(",", ":"), allow_nan=False)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


# -- Derivation validation --------------------------------------------------- #

DERIVATION_KINDS = frozenset({"discovered", "edited", "composed", "imported"})


def _parent_ref(value: object) -> dict[str, object]:
    if not isinstance(value, Mapping):
        raise ValidationError("a parent reference must be an object with world_id and number")
    world_id = value.get("world_id")
    number = value.get("number")
    if not isinstance(world_id, str) or not world_id:
        raise ValidationError("parent world_id is required")
    if not isinstance(number, int) or isinstance(number, bool) or number < 1:
        raise ValidationError("parent number must be a positive integer")
    return {"world_id": world_id, "number": number}


def normalize_derivation(raw: object, *, content_hash: str) -> tuple[dict[str, object], list[dict[str, object]]]:
    """Validate a derivation tag and return ``(derivation, parent_refs)``.

    Mirrors the four derivation tags in the spec: ``discovered``
    (dataset hash + config), ``edited`` (parent revision + edit ops),
    ``composed`` (parent revisions + optional namespacing), and ``imported``
    (source archive hash).  An absent derivation defaults to ``imported`` with
    the world's own content hash as its source -- the honest tag for a directly
    stored declarative world.
    """

    if raw is None:
        return {"kind": "imported", "source_hash": content_hash}, []
    if not isinstance(raw, Mapping):
        raise ValidationError("derivation must be an object")
    kind = raw.get("kind")
    if kind not in DERIVATION_KINDS:
        raise ValidationError("derivation kind must be one of discovered, edited, composed, or imported")
    if kind == "discovered":
        dataset_hash = raw.get("dataset_hash")
        if not isinstance(dataset_hash, str) or not dataset_hash:
            raise ValidationError("discovered derivation requires a dataset_hash")
        config = raw.get("config", {})
        if not isinstance(config, Mapping):
            raise ValidationError("discovered derivation config must be an object")
        return {"kind": "discovered", "dataset_hash": dataset_hash, "config": dict(config)}, []
    if kind == "imported":
        source_hash = raw.get("source_hash", content_hash)
        if not isinstance(source_hash, str) or not source_hash:
            raise ValidationError("imported derivation requires a source_hash")
        return {"kind": "imported", "source_hash": source_hash}, []
    if kind == "edited":
        parent = _parent_ref(raw.get("parent"))
        ops = raw.get("ops", [])
        if not isinstance(ops, list):
            raise ValidationError("edited derivation ops must be a list")
        return {"kind": "edited", "parent": parent, "ops": list(ops)}, [parent]
    parents_raw = raw.get("parents")
    if not isinstance(parents_raw, list) or not parents_raw:
        raise ValidationError("composed derivation requires a non-empty parents list")
    parents = [_parent_ref(item) for item in parents_raw]
    derivation: dict[str, object] = {"kind": "composed", "parents": parents}
    if "namespacing" in raw:
        namespacing = raw["namespacing"]
        if not isinstance(namespacing, Mapping):
            raise ValidationError("composed derivation namespacing must be an object")
        derivation["namespacing"] = dict(namespacing)
    return derivation, parents


# -- Review state machine ---------------------------------------------------- #

REVIEW_STATES = ("draft", "in_review", "approved", "rejected")
_ALLOWED_REVIEW: dict[str, frozenset[str]] = {
    "draft": frozenset({"in_review"}),
    "in_review": frozenset({"approved", "rejected"}),
    "rejected": frozenset({"in_review"}),
    "approved": frozenset(),
}


# -- Annotations ------------------------------------------------------------- #

ANNOTATION_TARGETS = frozenset({"world", "law", "revision"})
MAX_ANNOTATION_BYTES = 4096


def validate_annotation(body: object) -> dict[str, object]:
    """Validate an annotation body (target, optional ref, bounded UTF-8 text)."""

    if not isinstance(body, Mapping):
        raise ValidationError("annotation body must be an object")
    text = body.get("text")
    if not isinstance(text, str) or not text.strip():
        raise ValidationError("annotation text is required")
    if "\x00" in text:
        raise ValidationError("annotation text must not contain NUL")
    if len(text.encode("utf-8")) > MAX_ANNOTATION_BYTES:
        raise ValidationError(
            "annotation text exceeds the maximum size",
            details={"maximum": MAX_ANNOTATION_BYTES},
        )
    target = body.get("target", "world")
    if target not in ANNOTATION_TARGETS:
        raise ValidationError("annotation target must be world, law, or revision")
    ref = body.get("ref")
    if target == "revision":
        if not isinstance(ref, int) or isinstance(ref, bool) or ref < 1:
            raise ValidationError("a revision annotation requires a positive revision ref")
    elif target == "law":
        if not isinstance(ref, str) or not ref:
            raise ValidationError("a law annotation requires a law ref")
    else:
        ref = None
    return {"target": target, "ref": ref, "text": text.strip()}


# -- Stores ------------------------------------------------------------------ #


class MembershipStore:
    """Per-project role bindings, keyed by ``(organization_id, project_id)``."""

    def __init__(self) -> None:
        self._lock = RLock()
        self._roles: dict[tuple[str, str], dict[str, str]] = {}

    def establish_owner(self, organization_id: str, project_id: str, principal: str) -> None:
        """Bind ``principal`` as owner iff the project has no members yet.

        Called when a project is created so its creator gets full control -- this
        is what keeps single-user projects working: the sole actor is the owner.
        """

        if not organization_id or not project_id or not principal:
            return
        with self._lock:
            members = self._roles.setdefault((organization_id, project_id), {})
            if not members:
                members[principal] = OWNER

    def set_role(self, organization_id: str, project_id: str, principal: str, role: str) -> dict[str, object]:
        """Upsert ``principal``'s role for the project (caller must be authorized)."""

        if role not in ROLES:
            raise ValidationError("role must be owner, editor, or viewer")
        if not isinstance(principal, str) or not principal:
            raise ValidationError("member principal is required")
        with self._lock:
            members = self._roles.setdefault((organization_id, project_id), {})
            members[principal] = role
            return {"principal": principal, "role": role}

    def role(self, organization_id: str, project_id: str, principal: str) -> str | None:
        with self._lock:
            return self._roles.get((organization_id, project_id), {}).get(principal)

    def has_members(self, organization_id: str, project_id: str) -> bool:
        with self._lock:
            return bool(self._roles.get((organization_id, project_id)))

    def members(self, organization_id: str, project_id: str) -> list[dict[str, object]]:
        with self._lock:
            members = self._roles.get((organization_id, project_id), {})
            return [{"principal": principal, "role": role} for principal, role in sorted(members.items())]

    def owners(self, organization_id: str, project_id: str) -> list[str]:
        with self._lock:
            members = self._roles.get((organization_id, project_id), {})
            return sorted(principal for principal, role in members.items() if role == OWNER)

    def remove(self, organization_id: str, project_id: str, principal: str) -> None:
        """Remove a member; refuse to orphan the project by removing its last owner."""

        with self._lock:
            members = self._roles.get((organization_id, project_id))
            if not members or principal not in members:
                raise NotFoundError("member not found")
            if members[principal] == OWNER and [p for p, r in members.items() if r == OWNER] == [principal]:
                raise ConflictError("cannot remove the last owner of a project")
            del members[principal]


class RevisionLog:
    """Append-only, per-world revision chain keyed by ``(organization_id, world_id)``."""

    def __init__(self) -> None:
        self._lock = RLock()
        self._chains: dict[tuple[str, str], list[dict[str, object]]] = {}

    def append(
        self,
        organization_id: str,
        world_id: str,
        *,
        content_hash: str,
        derivation: dict[str, object],
        parents: list[dict[str, object]],
        actor: str,
    ) -> dict[str, object]:
        """Append an immutable revision; the revision number is monotonic per world."""

        with self._lock:
            chain = self._chains.setdefault((organization_id, world_id), [])
            record = {
                "world_id": world_id,
                "number": len(chain) + 1,
                "content_hash": content_hash,
                "derivation": deepcopy(derivation),
                "parents": deepcopy(parents),
                "actor": actor,
                "created_at": _now(),
                "review_state": "draft",
                "review_history": [],
            }
            chain.append(record)
            return deepcopy(record)

    def content_hash_of(self, organization_id: str, world_id: str, number: int) -> str | None:
        with self._lock:
            chain = self._chains.get((organization_id, world_id), [])
            if 1 <= number <= len(chain):
                return str(chain[number - 1]["content_hash"])
            return None

    def list(self, organization_id: str, world_id: str) -> list[dict[str, object]]:
        with self._lock:
            return [deepcopy(record) for record in self._chains.get((organization_id, world_id), [])]

    def get(self, organization_id: str, world_id: str, number: int) -> dict[str, object]:
        with self._lock:
            chain = self._chains.get((organization_id, world_id), [])
            if not (1 <= number <= len(chain)):
                raise NotFoundError("revision not found")
            return deepcopy(chain[number - 1])

    def transition(
        self,
        organization_id: str,
        world_id: str,
        number: int,
        target: str,
        *,
        actor: str,
        is_owner: bool,
    ) -> dict[str, object]:
        """Transition a revision's review state, enforcing the approval rule.

        Only an owner may record ``approved``.  Illegal transitions are a 409.
        Returns the updated (deep-copied) revision record.
        """

        if target not in REVIEW_STATES:
            raise ValidationError("review state must be draft, in_review, approved, or rejected")
        with self._lock:
            chain = self._chains.get((organization_id, world_id), [])
            if not (1 <= number <= len(chain)):
                raise NotFoundError("revision not found")
            record = chain[number - 1]
            current = str(record["review_state"])
            if target not in _ALLOWED_REVIEW[current]:
                raise ConflictError(f"cannot transition review from {current} to {target}")
            if target == "approved" and not is_owner:
                from lawsynth_server.errors import AuthorizationError

                raise AuthorizationError("only an owner may approve a revision")
            record["review_state"] = target
            record["review_history"].append({"from": current, "to": target, "actor": actor, "at": _now()})
            return deepcopy(record)

    def world_trusted(self, organization_id: str, world_id: str) -> bool:
        """A world is trusted iff any of its revisions is approved."""

        with self._lock:
            chain = self._chains.get((organization_id, world_id), [])
            return any(record["review_state"] == "approved" for record in chain)


class AnnotationStore:
    """Append-only, per-world annotation log keyed by ``(organization_id, world_id)``."""

    def __init__(self) -> None:
        self._lock = RLock()
        self._logs: dict[tuple[str, str], list[dict[str, object]]] = {}

    def append(self, organization_id: str, world_id: str, annotation: dict[str, object], *, actor: str) -> dict[str, object]:
        with self._lock:
            log = self._logs.setdefault((organization_id, world_id), [])
            record = {
                "world_id": world_id,
                "ordinal": len(log) + 1,
                "target": annotation["target"],
                "ref": annotation["ref"],
                "text": annotation["text"],
                "actor": actor,
                "created_at": _now(),
            }
            log.append(record)
            return deepcopy(record)

    def list(self, organization_id: str, world_id: str) -> list[dict[str, object]]:
        with self._lock:
            return [deepcopy(record) for record in self._logs.get((organization_id, world_id), [])]


# -- Deterministic workspace merge ------------------------------------------- #


def _canonical_row(name: str, value: object) -> dict[str, object]:
    if not isinstance(value, Mapping):
        raise ValidationError(f"workspace row {name!r} must be an object")
    content_hash = value.get("content_hash")
    if not isinstance(content_hash, str) or not content_hash:
        raise ValidationError(f"workspace row {name!r} requires a content_hash")
    revision = value.get("revision", 0)
    if not isinstance(revision, int) or isinstance(revision, bool) or revision < 0:
        raise ValidationError(f"workspace row {name!r} revision must be a non-negative integer")
    row = dict(value)
    row["name"] = name
    row["content_hash"] = content_hash
    row["revision"] = revision
    return row


def _index(rows: object, side: str) -> dict[str, dict[str, object]]:
    if not isinstance(rows, list):
        raise ValidationError(f"{side} workspace index must be a list of rows")
    index: dict[str, dict[str, object]] = {}
    for entry in rows:
        if not isinstance(entry, Mapping):
            raise ValidationError(f"{side} workspace rows must be objects")
        name = entry.get("name")
        if not isinstance(name, str) or not name:
            raise ValidationError(f"{side} workspace rows require a name")
        if name in index:
            raise ValidationError(f"{side} workspace index has a duplicate name {name!r}")
        index[name] = _canonical_row(name, entry)
    return index


def _pick(left: dict[str, object], right: dict[str, object]) -> dict[str, object]:
    """Deterministically pick between two same-content rows (order-independent)."""

    left_rev, right_rev = int(left["revision"]), int(right["revision"])
    if left_rev != right_rev:
        return left if left_rev > right_rev else right
    left_key = json.dumps(left, sort_keys=True, separators=(",", ":"))
    right_key = json.dumps(right, sort_keys=True, separators=(",", ":"))
    return left if left_key <= right_key else right


def merge_indexes(base: object, incoming: object) -> dict[str, object]:
    """Merge two ``library.tsv``-style workspace indexes deterministically.

    Merge by world name, then by revision lineage.  A name present on both sides
    with the SAME content hash is merged (the higher-revision row wins, ties
    broken by canonical order so the result is order-independent).  A name whose
    content hash DIFFERS is a **conflict** returned with both revisions -- never
    silently overwritten.  Disjoint names union, so merge is associative and
    commutative over disjoint names and offline replicas converge.
    """

    left = _index(base, "base")
    right = _index(incoming, "incoming")
    merged: list[dict[str, object]] = []
    conflicts: list[dict[str, object]] = []
    for name in sorted(set(left) | set(right)):
        left_row = left.get(name)
        right_row = right.get(name)
        if left_row is not None and right_row is not None:
            if left_row["content_hash"] == right_row["content_hash"]:
                merged.append(_pick(left_row, right_row))
            else:
                conflicts.append({"name": name, "base": left_row, "incoming": right_row})
        else:
            merged.append(left_row if left_row is not None else right_row)  # type: ignore[arg-type]
    return {
        "merged": merged,
        "conflicts": conflicts,
        "merged_count": len(merged),
        "conflict_count": len(conflicts),
    }
