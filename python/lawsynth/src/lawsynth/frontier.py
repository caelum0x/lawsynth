"""Deterministic Pareto filtering for Python-side candidate metadata."""

from collections.abc import Sequence

from .candidate import CandidateMetrics


def pareto_front(metrics: Sequence[CandidateMetrics]) -> tuple[int, ...]:
    """Return input indices not dominated on error and complexity."""
    return tuple(index for index, candidate in enumerate(metrics) if not any(other_index != index and other.dominates(candidate) for other_index, other in enumerate(metrics)))
