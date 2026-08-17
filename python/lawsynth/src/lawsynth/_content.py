"""Deterministic content-addressing helpers for governance (P9).

Every governance artifact — dataset, discovery config, world revision, evaluation
and report — is addressed by a SHA-256 over a *canonical* JSON encoding. Canonical
means: keys sorted, compact separators, and round-trippable float reprs. The same
logical value therefore always hashes to the same digest, on any run, offline.
"""

from __future__ import annotations

import hashlib
import json
from typing import Any

__all__ = ["canonical_json", "content_digest", "dataset_digest", "world_hash"]

# A stable prefix length for short, human-readable digest displays.
SHORT = 12


def canonical_json(payload: Any) -> str:
    """Return a canonical JSON string: sorted keys, compact, round-trippable."""
    return json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def content_digest(payload: Any) -> str:
    """SHA-256 hex digest of the canonical JSON encoding of ``payload``."""
    return hashlib.sha256(canonical_json(payload).encode("utf-8")).hexdigest()


def dataset_digest(dataset: object) -> str:
    """Content digest of a :class:`~lawsynth.dataset.Dataset` (time + columns).

    The digest is order-independent across columns (names are sorted) and captures
    every observed value at full round-trippable precision.
    """
    time = [float(v) for v in dataset.time]  # type: ignore[attr-defined]
    columns = {
        str(name): [float(v) for v in values]
        for name, values in sorted(dataset.columns.items())  # type: ignore[attr-defined]
    }
    return content_digest({"time": time, "columns": columns})


def dataset_columns(dataset: object) -> list[str]:
    """The sorted column names of a dataset (recorded alongside its digest)."""
    return sorted(str(name) for name in dataset.columns)  # type: ignore[attr-defined]


def world_hash(world: object) -> str:
    """Content-addressed **world revision hash** of a discovered/native world.

    Hashes the world's canonical declarative structure (states, parameters,
    controls and equations, each sorted) recovered via
    :func:`~lawsynth.worldspec.spec_of`. Because discovery is deterministic, an
    identical dataset + config reproduces identical equations and therefore an
    identical hash — the property :meth:`Lineage.verify_reproducible` relies on.
    """
    from .worldspec import spec_of

    spec = spec_of(world)
    payload = {
        "states": sorted(spec.states),
        "parameters": sorted((str(k), float(v)) for k, v in spec.parameters),
        "controls": sorted(spec.controls),
        "equations": sorted((str(t), str(e)) for t, e in spec.equations),
    }
    return content_digest(payload)
