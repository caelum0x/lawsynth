"""Tenant-isolated repositories. All calls require an organization id."""

from __future__ import annotations

from copy import deepcopy
from datetime import UTC, datetime
from threading import RLock
from uuid import uuid4

from .errors import ConflictError, NotFoundError, ValidationError


class Repository:
    def __init__(self, kind: str) -> None:
        self.kind, self._items, self._lock = kind, {}, RLock()

    def create(self, organization_id: str, values: dict[str, object]) -> dict[str, object]:
        if not organization_id:
            raise ValidationError("organization_id is required")
        name = values.get("name")
        if not isinstance(name, str) or not name.strip():
            raise ValidationError("name is required")
        with self._lock:
            if any(x["organization_id"] == organization_id and x["name"] == name and not x.get("deleted_at") for x in self._items.values()):
                raise ConflictError(f"{self.kind} name already exists")
            record = {**deepcopy(values), "id": str(uuid4()), "organization_id": organization_id, "created_at": datetime.now(UTC).isoformat(), "deleted_at": None}
            self._items[record["id"]] = record
            return deepcopy(record)

    def get(self, organization_id: str, identifier: str) -> dict[str, object]:
        with self._lock:
            value = self._items.get(identifier)
            if not value or value["organization_id"] != organization_id or value.get("deleted_at"):
                raise NotFoundError(f"{self.kind} not found")
            return deepcopy(value)

    def list(self, organization_id: str) -> list[dict[str, object]]:
        with self._lock:
            values = [deepcopy(x) for x in self._items.values() if x["organization_id"] == organization_id and not x.get("deleted_at")]
        return sorted(values, key=lambda x: (str(x["created_at"]), str(x["id"])))

    def update(self, organization_id: str, identifier: str, values: dict[str, object]) -> dict[str, object]:
        allowed = {"name", "metadata", "status", "artifact_hash", "world_id", "dataset_id"}
        illegal = set(values) - allowed
        if illegal:
            raise ValidationError("immutable or unknown repository fields", details={"fields": sorted(illegal)})
        with self._lock:
            record = self.get(organization_id, identifier)
            record.update(deepcopy(values))
            self._items[identifier] = record
            return deepcopy(record)

    def delete(self, organization_id: str, identifier: str) -> None:
        with self._lock:
            record = self.get(organization_id, identifier)
            record["deleted_at"] = datetime.now(UTC).isoformat()
            self._items[identifier] = record
