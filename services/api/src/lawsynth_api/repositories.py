"""Typed repository facades over the domain service repositories.

Every persisted resource (projects, datasets, worlds, runs) is backed by a
tenant-isolated repository in ``lawsynth_server``.  This module gives the API
process a typed, name-indexed view of those repositories without re-declaring
their storage or ownership rules -- reads and writes still flow through the
domain, keyed by ``organization_id``.  It is the single place the transport maps
a public resource segment to its backing repository.
"""

from __future__ import annotations

from typing import Mapping

from lawsynth_server.dependencies import Services

# The public resource segments that map one-to-one to a domain repository.
SEGMENTS = ("projects", "datasets", "worlds", "runs")


class ApiRepositories:
    """A name-indexed facade over the domain's persisted-resource repositories."""

    def __init__(self, services: Services) -> None:
        self._repositories: Mapping[str, object] = {
            segment: getattr(services, segment) for segment in SEGMENTS
        }

    def segments(self) -> tuple[str, ...]:
        """Return the resource segments backed by a repository."""

        return SEGMENTS

    def has(self, segment: str) -> bool:
        """True when ``segment`` maps to a backing repository."""

        return segment in self._repositories

    def get(self, segment: str) -> object:
        """Return the domain repository for ``segment`` (raises ``KeyError``)."""

        return self._repositories[segment]
