"""Append-only, per-tenant metering log for usage reporting.

Every billable action a tenant takes (submitting a discovery run, storing dataset
bytes) is recorded here as an immutable :class:`MeteringRecord`.  The log is the
usage-reporting substrate the hosted-platform spec requires
(``specs/hosted-platform/README.md`` -> "Quota, rate & metering"): it is
*append-only* (records are never mutated or deleted), *tenant-partitioned* (a
read for tenant A can never observe tenant B's records), and ordered by a
per-tenant **content-ordinal** rather than a wall clock.

Why content-ordinals and not timestamps
    Determinism.  The single-node engine is the reference for correctness and is
    fully deterministic and offline; a metering log keyed on ``time.time()``
    would make usage reports depend on when a test ran.  Instead each tenant
    carries a monotonic counter that starts at 1 and increments by one per
    appended record, so the *ordering* of billable events is reproducible from
    the inputs alone.  A deployment that needs civil timestamps layers them on
    top (e.g. at export) without changing this ordering contract.
"""

from __future__ import annotations

from dataclasses import dataclass
from threading import RLock

from lawsynth_server.errors import ValidationError

# The billable action vocabulary.  Kept small and explicit so a usage report can
# aggregate by a closed set of meters rather than free-form strings.
RUN_SUBMITTED = "run_submitted"
BYTES_STORED = "bytes_stored"
ACTIONS = frozenset({RUN_SUBMITTED, BYTES_STORED})


@dataclass(frozen=True, slots=True)
class MeteringRecord:
    """One immutable, billable line item scoped to a single tenant."""

    organization_id: str
    ordinal: int
    action: str
    quantity: int
    subject: str

    def __post_init__(self) -> None:
        if not isinstance(self.organization_id, str) or not self.organization_id:
            raise ValidationError("metering organization_id is required")
        if not isinstance(self.ordinal, int) or isinstance(self.ordinal, bool) or self.ordinal < 1:
            raise ValidationError("metering ordinal must be a positive integer")
        if self.action not in ACTIONS:
            raise ValidationError("metering action must be a known billable action")
        if not isinstance(self.quantity, int) or isinstance(self.quantity, bool) or self.quantity < 0:
            raise ValidationError("metering quantity must be a non-negative integer")
        if not isinstance(self.subject, str) or not self.subject:
            raise ValidationError("metering subject is required")

    def to_wire(self) -> dict[str, object]:
        """Return the JSON-serializable body used in a usage report."""

        return {
            "ordinal": self.ordinal,
            "action": self.action,
            "quantity": self.quantity,
            "subject": self.subject,
        }


class MeteringLog:
    """Thread-safe, in-process, tenant-partitioned append-only meter store."""

    def __init__(self) -> None:
        self._lock = RLock()
        self._records: dict[str, list[MeteringRecord]] = {}
        self._next_ordinal: dict[str, int] = {}

    def record(self, organization_id: str, action: str, quantity: int, subject: str) -> MeteringRecord:
        """Append one billable action, assigning the next per-tenant ordinal.

        The record is never mutated after this call; there is no update or delete
        surface, which is what makes the log a trustworthy billing substrate.
        """

        if not isinstance(organization_id, str) or not organization_id:
            raise ValidationError("metering organization_id is required")
        with self._lock:
            ordinal = self._next_ordinal.get(organization_id, 1)
            record = MeteringRecord(
                organization_id=organization_id,
                ordinal=ordinal,
                action=action,
                quantity=quantity,
                subject=subject,
            )
            self._records.setdefault(organization_id, []).append(record)
            self._next_ordinal[organization_id] = ordinal + 1
            return record

    def records(self, organization_id: str) -> list[MeteringRecord]:
        """Return this tenant's records in ordinal order (never another tenant's)."""

        with self._lock:
            return list(self._records.get(organization_id, ()))

    def usage(self, organization_id: str) -> dict[str, object]:
        """Aggregate this tenant's meters into a usage-report envelope."""

        records = self.records(organization_id)
        totals: dict[str, int] = {action: 0 for action in sorted(ACTIONS)}
        for record in records:
            totals[record.action] += record.quantity
        return {
            "organization_id": organization_id,
            "totals": totals,
            "records": [record.to_wire() for record in records],
        }
