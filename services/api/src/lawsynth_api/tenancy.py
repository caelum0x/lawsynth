"""Tenant-scoped capability index for content-addressed artifacts.

The object store is content-addressed: an artifact is keyed by the SHA-256 of its
bytes, and identical content from two tenants deduplicates to one blob.  That is
correct for storage efficiency, but it means the *key alone is not a grant* -- and
the hosted-platform spec is explicit that "identifiers are never grants" and that
"no ... artifact download may cross a tenant boundary"
(``specs/hosted-platform/README.md`` -> "Tenancy isolation").

Without this index, a tenant that learned another tenant's SHA-256 could download
its bytes, because the domain download path (``GET /v1/artifacts/{sha}``) resolves
purely by content hash with no owner check.  This module records, per tenant, the
set of artifact hashes that tenant has actually stored, so the API can authorize a
download against the caller's tenant before serving it.  Deduplicated content is
independently owned by each tenant that stored it, so a legitimate re-upload of
the same bytes is never blocked.
"""

from __future__ import annotations

from threading import RLock


class ArtifactOwnership:
    """Thread-safe, in-process record of which tenant stored which artifact hash."""

    def __init__(self) -> None:
        self._lock = RLock()
        self._owned: dict[str, set[str]] = {}

    def grant(self, organization_id: str, sha256: str) -> None:
        """Record that ``organization_id`` has stored the artifact ``sha256``."""

        if not organization_id or not sha256:
            return
        with self._lock:
            self._owned.setdefault(organization_id, set()).add(sha256)

    def owns(self, organization_id: str, sha256: str) -> bool:
        """Return whether ``organization_id`` may download the artifact ``sha256``."""

        if not organization_id or not sha256:
            return False
        with self._lock:
            return sha256 in self._owned.get(organization_id, ())
