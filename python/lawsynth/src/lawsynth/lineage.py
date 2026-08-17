"""Content-addressed, reproducible **lineage** chains for governed models (P9).

A governed model must carry a queryable, exportable lineage chain::

    dataset(hash, columns) → preparation(ops) → discovery(config, engine version)
      → world(revision hash) → evaluation(validate/backtest/ensemble)
      → report(hash) → decision(actor, action, ordinal)

Every link is content-addressed: a link's digest is a SHA-256 over its payload
*and its parent's digest*, so the chain is immutable and tamper-evident, and any
link reconstructs the full ancestry back to the source dataset. Because discovery
and evaluation are deterministic and offline, a lineage is independently
reproducible — :meth:`Lineage.verify_reproducible` re-runs the recorded dataset +
config and asserts the same world revision hash.

The :class:`Lineage` is captured by a :class:`~lawsynth.study.Study` as it
progresses; nothing is mutated in place — every ``record_*`` returns a new,
extended chain and the study swaps its reference.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Mapping, Sequence

from ._content import SHORT, content_digest, dataset_columns, dataset_digest, world_hash
from ._version import __version__

__all__ = ["LineageLink", "Lineage"]

# The genesis parent for the first link in a chain (no ancestor).
_GENESIS = "0" * 64


@dataclass(frozen=True, slots=True)
class LineageLink:
    """One immutable, content-addressed link in a lineage chain."""

    kind: str
    payload: Mapping[str, object]
    parent: str
    digest: str

    def to_dict(self) -> dict[str, object]:
        return {
            "kind": self.kind,
            "payload": dict(self.payload),
            "parent": self.parent,
            "digest": self.digest,
        }


def _link(kind: str, payload: Mapping[str, object], parent: str) -> LineageLink:
    """Build a content-addressed link: digest = H(kind, payload, parent)."""
    digest = content_digest({"kind": kind, "payload": payload, "parent": parent})
    return LineageLink(kind=kind, payload=dict(payload), parent=parent, digest=digest)


class Lineage:
    """An append-only, content-addressed provenance chain for one model.

    Immutable by construction: every ``record_*`` returns a *new* :class:`Lineage`
    with one more link. The originating dataset, config and states are retained
    (process-local) so the chain can be independently re-run and verified.
    """

    __slots__ = ("_links", "_dataset", "_states", "_config")

    def __init__(
        self,
        links: Sequence[LineageLink] = (),
        *,
        dataset: object | None = None,
        states: Sequence[str] = (),
        config: object | None = None,
    ) -> None:
        self._links = tuple(links)
        self._dataset = dataset
        self._states = tuple(states)
        self._config = config

    # -- construction ------------------------------------------------------- #

    @classmethod
    def from_dataset(cls, dataset: object, states: Sequence[str]) -> "Lineage":
        """Start a chain rooted at ``dataset`` (its content hash + columns)."""
        payload = {
            "dataset_hash": dataset_digest(dataset),
            "columns": dataset_columns(dataset),
            "states": list(states),
            "samples": len(dataset.time),  # type: ignore[attr-defined]
        }
        root = _link("dataset", payload, _GENESIS)
        return cls([root], dataset=dataset, states=states)

    # -- accessors ---------------------------------------------------------- #

    @property
    def links(self) -> tuple[LineageLink, ...]:
        return self._links

    @property
    def head(self) -> LineageLink:
        if not self._links:
            raise ValueError("empty lineage has no head")
        return self._links[-1]

    def link_of(self, kind: str) -> LineageLink | None:
        """The most recent link of ``kind``, or ``None`` if absent."""
        for link in reversed(self._links):
            if link.kind == kind:
                return link
        return None

    @property
    def world_revision(self) -> str | None:
        """The recorded world revision hash, if discovery has been recorded."""
        link = self.link_of("world")
        return str(link.payload["world_hash"]) if link is not None else None

    # -- immutable recording ------------------------------------------------ #

    def _extended(self, link: LineageLink) -> "Lineage":
        return Lineage(
            (*self._links, link),
            dataset=self._dataset,
            states=self._states,
            config=self._config,
        )

    def record_preparation(self, ops: Sequence[Mapping[str, object]]) -> "Lineage":
        payload = {"ops": [dict(op) for op in ops]}
        return self._extended(_link("preparation", payload, self.head.digest))

    def record_discovery(self, config: object, world: object) -> "Lineage":
        """Record the discovery config + engine version, then the world revision."""
        config_payload = {
            "config": _config_payload(config),
            "engine_version": __version__,
        }
        chain = self._extended(_link("discovery", config_payload, self.head.digest))
        world_payload = {"world_hash": world_hash(world)}
        chain = chain._extended(_link("world", world_payload, chain.head.digest))
        # Retain the config for reproducibility re-runs.
        return Lineage(
            chain._links,
            dataset=chain._dataset,
            states=chain._states,
            config=config,
        )

    def record_evaluation(self, kind: str, summary: Mapping[str, object]) -> "Lineage":
        """Record an evaluation link (e.g. ``validate``/``backtest``/``ensemble``)."""
        payload = {"evaluation": kind, "summary": dict(summary)}
        return self._extended(_link("evaluation", payload, self.head.digest))

    def record_report(self, report_hash: str, *, kind: str = "model_card") -> "Lineage":
        payload = {"report_kind": kind, "report_hash": report_hash}
        return self._extended(_link("report", payload, self.head.digest))

    def record_decision(self, actor: str, action: str, ordinal: int) -> "Lineage":
        """Record a governance decision (no clock — a content ordinal, not a time)."""
        payload = {"actor": actor, "action": action, "ordinal": int(ordinal)}
        return self._extended(_link("decision", payload, self.head.digest))

    # -- integrity ---------------------------------------------------------- #

    def verify_chain(self) -> bool:
        """True iff every link's digest recomputes and its parent linkage holds."""
        parent = _GENESIS
        for link in self._links:
            expected = content_digest(
                {"kind": link.kind, "payload": dict(link.payload), "parent": link.parent}
            )
            if link.digest != expected or link.parent != parent:
                return False
            parent = link.digest
        return True

    def verify_reproducible(self) -> bool:
        """Re-run the recorded dataset + config and assert the same world hash.

        Deterministic reproducibility check (P9): re-discovering from the exact
        recorded inputs must yield an identical world revision hash. Returns
        ``False`` if no world was recorded or the inputs are unavailable.
        """
        recorded = self.world_revision
        if recorded is None or self._dataset is None or self._config is None:
            return False
        from .study import _discover_world

        replayed = _discover_world(self._dataset, self._states, self._config)
        return world_hash(replayed) == recorded

    # -- serialisation ------------------------------------------------------ #

    def to_dict(self) -> dict[str, object]:
        return {
            "engine_version": __version__,
            "world_revision": self.world_revision,
            "chain_valid": self.verify_chain(),
            "links": [link.to_dict() for link in self._links],
        }

    def to_json(self, *, indent: int | None = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent, sort_keys=True)

    def to_text(self) -> str:
        lines = [f"Lineage ({len(self._links)} links, engine {__version__}):"]
        for i, link in enumerate(self._links):
            arrow = "  " if i == 0 else "  ↓ "
            lines.append(f"{arrow}{link.kind:<12} {link.digest[:SHORT]}")
        head = self.world_revision
        if head is not None:
            lines.append(f"  world revision: {head[:SHORT]}")
        return "\n".join(lines)

    def __str__(self) -> str:
        return self.to_text()

    def __repr__(self) -> str:
        return f"Lineage(links={len(self._links)}, world={self.world_revision and self.world_revision[:SHORT]!r})"


def _config_payload(config: object) -> dict[str, object]:
    """A canonical, JSON-safe mapping of a DiscoveryConfig's declared fields."""
    fields = getattr(type(config), "__dataclass_fields__", None)
    if fields is None:
        return {}
    return {name: getattr(config, name) for name in sorted(fields)}
