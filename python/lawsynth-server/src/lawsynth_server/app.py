"""In-process request dispatcher; deploy it behind a WSGI/ASGI adapter.

The core deliberately has no unreviewed HTTP framework dependency. It accepts
plain request dictionaries, which makes authorization and tenant boundaries
testable independently of an edge server.
"""

from __future__ import annotations

import base64
import binascii
from dataclasses import asdict

from .auth import require_scope
from .dependencies import Services, build_services
from .errors import NotFoundError, ValidationError
from .health import check
from .middleware import invoke
from .pagination import page
from .settings import Settings
from .simulations import validate_simulation_spec


class Application:
    def __init__(self, settings: Settings | None = None, *, services: Services | None = None) -> None:
        self.services = services or build_services(settings or Settings())

    def dispatch(self, request: dict[str, object]) -> dict[str, object]:
        return invoke(self._dispatch, request)

    def _dispatch(self, request: dict[str, object]) -> dict[str, object]:
        method = request.get("method")
        path = request.get("path")
        if not isinstance(method, str) or not isinstance(path, str):
            raise ValidationError("request requires method and path")
        if method == "GET" and path == "/health":
            return {"status": 200, "body": asdict(check(self.services.database, self.services.storage))}
        headers = request.get("headers", {})
        if not isinstance(headers, dict):
            raise ValidationError("headers must be an object")
        principal = self.services.auth.authenticate(headers.get("Authorization"))
        parts = [part for part in path.split("/") if part]
        if method == "GET" and parts == ["events"]:
            require_scope(principal, "read")
            return {"status": 200, "body": {"items": self.services.events.list(principal.organization_id, after=request.get("after"))}}
        if parts == ["artifacts"] and method == "POST":
            require_scope(principal, "write")
            body = request.get("body")
            if not isinstance(body, dict) or not isinstance(body.get("data_base64"), str):
                raise ValidationError("artifact upload requires base64 data")
            try:
                data = base64.b64decode(body["data_base64"], validate=True)
            except (ValueError, UnicodeEncodeError, binascii.Error) as exc:
                raise ValidationError("artifact data is not valid base64") from exc
            key = headers.get("Idempotency-Key")
            if not isinstance(key, str):
                raise ValidationError("Idempotency-Key is required for writes")
            def put() -> tuple[int, dict[str, object]]:
                artifact = self.services.storage.put(data, str(body.get("media_type", "application/octet-stream")))
                item = asdict(artifact)
                self.services.events.append(principal.organization_id, "artifacts.created", {"sha256": artifact.sha256})
                return 201, item
            status, response, replayed = self.services.idempotency.execute(principal.organization_id, key, {"method": method, "path": path, "body": body}, put)
            return {"status": status, "headers": {"Idempotency-Replayed": str(replayed).lower()}, "body": response}
        if not parts or parts[0] not in {"projects", "datasets", "worlds", "runs"} or len(parts) > 2:
            raise NotFoundError("route not found")
        repository = getattr(self.services, parts[0])
        if method == "GET" and len(parts) == 1:
            require_scope(principal, "read")
            query = request.get("query", {})
            if not isinstance(query, dict):
                raise ValidationError("query must be an object")
            result = page(repository.list(principal.organization_id), cursor=query.get("cursor"), limit=int(query.get("limit", 20)), maximum=self.services.settings.max_page_size)
            return {"status": 200, "body": {"items": result.items, "next_cursor": result.next_cursor}}
        if method == "GET" and len(parts) == 2:
            require_scope(principal, "read")
            return {"status": 200, "body": repository.get(principal.organization_id, parts[1])}
        if method == "POST" and len(parts) == 1:
            require_scope(principal, "write")
            body = request.get("body")
            if not isinstance(body, dict):
                raise ValidationError("body must be an object")
            if parts[0] == "runs" and "simulation" in body:
                body = {**body, "simulation": validate_simulation_spec(body["simulation"])}
            key = headers.get("Idempotency-Key")
            if not isinstance(key, str):
                raise ValidationError("Idempotency-Key is required for writes")
            def create() -> tuple[int, dict[str, object]]:
                item = repository.create(principal.organization_id, body)
                self.services.events.append(principal.organization_id, f"{parts[0]}.created", {"id": item["id"]})
                return 201, item
            status, response, replayed = self.services.idempotency.execute(principal.organization_id, key, {"method": method, "path": path, "body": body}, create)
            return {"status": status, "headers": {"Idempotency-Replayed": str(replayed).lower()}, "body": response}
        if method == "PATCH" and len(parts) == 2:
            require_scope(principal, "write")
            body = request.get("body")
            if not isinstance(body, dict):
                raise ValidationError("body must be an object")
            key = headers.get("Idempotency-Key")
            if not isinstance(key, str):
                raise ValidationError("Idempotency-Key is required for writes")
            def update() -> tuple[int, dict[str, object]]:
                item = repository.update(principal.organization_id, parts[1], body)
                self.services.events.append(principal.organization_id, f"{parts[0]}.updated", {"id": item["id"]})
                return 200, item
            status, response, replayed = self.services.idempotency.execute(principal.organization_id, key, {"method": method, "path": path, "body": body}, update)
            return {"status": status, "headers": {"Idempotency-Replayed": str(replayed).lower()}, "body": response}
        if method == "DELETE" and len(parts) == 2:
            require_scope(principal, "write")
            key = headers.get("Idempotency-Key")
            if not isinstance(key, str):
                raise ValidationError("Idempotency-Key is required for writes")
            def delete() -> tuple[int, dict[str, object]]:
                repository.delete(principal.organization_id, parts[1])
                self.services.events.append(principal.organization_id, f"{parts[0]}.deleted", {"id": parts[1]})
                return 204, {}
            status, response, replayed = self.services.idempotency.execute(principal.organization_id, key, {"method": method, "path": path}, delete)
            return {"status": status, "headers": {"Idempotency-Replayed": str(replayed).lower()}, "body": response}
        raise NotFoundError("route not found")


def create_app(settings: Settings | None = None) -> Application:
    return Application(settings)
