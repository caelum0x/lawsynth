"""Per-tenant quota enforcement at the API admission boundary.

The hosted-platform spec (``specs/hosted-platform/README.md`` -> "Quota, rate &
metering") requires per-tenant policy to *bound concurrent runs, queued jobs, and
dataset size*, and that exceeding quota return a documented error rather than
silently dropping work.  This module owns that admission arithmetic for the
Python API -- the surface where a discovery run is actually submitted.

It mirrors, at the tenant granularity, the same "refuse over-commitment rather
than corrupt an invariant" discipline the scheduler's Rust ``quota`` module
applies at the pool granularity: capacity is checked *before* work is admitted,
and the caller gets an actionable rejection.

What is REAL enforcement here
    ``check_admission`` is called synchronously on ``POST /v1/runs`` before the
    run is recorded (and before the native probe), so a tenant over its active-run
    or stored-bytes ceiling is turned away with ``429 quota_exceeded``.  This is a
    hard gate, not advisory.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Mapping, Sequence

from lawsynth_server.dependencies import Services
from lawsynth_server.errors import ServerError, ValidationError

# Run statuses that count against the concurrency ceiling: a run that is queued or
# running occupies tenant capacity; terminal runs (succeeded/failed/cancelled) do
# not.  This is the tenant-level analogue of the scheduler's reserved/free split.
_ACTIVE_STATUSES = frozenset({"queued", "running"})


class QuotaExceededError(ServerError):
    """A tenant exceeded a provisioned quota; surfaced as a documented ``429``.

    Reuses the domain :class:`ServerError` envelope so it flows through the same
    ``except ServerError`` translation the discovery submit path already applies,
    rendering ``{"error": {"code": "quota_exceeded", ...}}``.
    """

    status_code, code = 429, "quota_exceeded"


@dataclass(frozen=True, slots=True)
class QuotaPolicy:
    """Immutable per-tenant limits applied uniformly to every tenant.

    Defaults are deliberately generous so the single-node, self-hosted product is
    unaffected; a hosted deployment tightens them via :class:`ApiSettings`.
    """

    max_active_runs: int = 1000
    max_dataset_bytes: int = 1024 * 1024 * 1024

    def __post_init__(self) -> None:
        if not isinstance(self.max_active_runs, int) or self.max_active_runs < 1:
            raise ValidationError("max_active_runs must be a positive integer")
        if not isinstance(self.max_dataset_bytes, int) or self.max_dataset_bytes < 1:
            raise ValidationError("max_dataset_bytes must be a positive integer")


def dataset_bytes(time: object, columns: object) -> int:
    """Return a stable byte size for a dataset's observations.

    The measure is the UTF-8 length of the canonical JSON of the ``time`` and
    ``columns`` arrays.  It is deterministic and offline (no wall clock, no
    platform-dependent sizing), which keeps metering and quota reproducible from
    the inputs alone.  A dataset with no observations measures as ``0``.
    """

    if not isinstance(time, (list, tuple)) or not isinstance(columns, Mapping):
        return 0
    payload = {
        "time": [float(value) for value in time],
        "columns": {str(name): [float(value) for value in series] for name, series in columns.items()},
    }
    return len(json.dumps(payload, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8"))


class QuotaGuard:
    """Applies a :class:`QuotaPolicy` against a tenant's live domain state."""

    def __init__(self, policy: QuotaPolicy) -> None:
        self._policy = policy

    @property
    def policy(self) -> QuotaPolicy:
        return self._policy

    def check_admission(
        self,
        services: Services,
        organization_id: str,
        *,
        new_dataset_bytes: int = 0,
    ) -> None:
        """Reject a run submission that would breach an active-run or storage cap.

        ``new_dataset_bytes`` is the size of any inline dataset the submission
        will materialise; a submission that references an existing dataset stores
        no new bytes and passes ``0``.
        """

        active = self._active_run_count(services.runs.list(organization_id))
        if active >= self._policy.max_active_runs:
            raise QuotaExceededError(
                "active run quota exceeded",
                details={"limit": self._policy.max_active_runs, "active": active, "quota": "active_runs"},
            )
        stored = self._stored_dataset_bytes(services.datasets.list(organization_id))
        projected = stored + max(0, int(new_dataset_bytes))
        if projected > self._policy.max_dataset_bytes:
            raise QuotaExceededError(
                "dataset storage quota exceeded",
                details={
                    "limit": self._policy.max_dataset_bytes,
                    "stored": stored,
                    "requested": max(0, int(new_dataset_bytes)),
                    "quota": "dataset_bytes",
                },
            )

    @staticmethod
    def _active_run_count(runs: Sequence[Mapping[str, object]]) -> int:
        return sum(1 for run in runs if run.get("status") in _ACTIVE_STATUSES)

    @staticmethod
    def _stored_dataset_bytes(datasets: Sequence[Mapping[str, object]]) -> int:
        return sum(dataset_bytes(record.get("time"), record.get("columns")) for record in datasets)
