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
from .errors import ConflictError, NotFoundError, ValidationError
from .health import check
from .middleware import invoke
from .native import discover_world, simulate_world
from .analysis import analyze_stability, validate_stability_request
from .pagination import page
from .settings import Settings
from .simulations import validate_simulation_spec
from ._version import __version__

# The explicit HTTP protocol version this dispatcher speaks.  It tracks the
# ``/v1`` route prefix and is published by ``GET /v1/version`` so clients never
# interpret responses under ambiguous semantics (specs/service-api/versioning.md).
PROTOCOL_VERSION = "1"


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
        if method == "GET" and path in {"/health", "/v1/health"}:
            return {"status": 200, "body": asdict(check(self.services.database, self.services.storage))}
        if method == "GET" and path in {"/version", "/v1/version"}:
            return {"status": 200, "body": {"version": __version__, "protocol": PROTOCOL_VERSION}}
        headers = request.get("headers", {})
        if not isinstance(headers, dict):
            raise ValidationError("headers must be an object")
        principal = self.services.auth.authenticate(headers.get("Authorization"))
        parts = [part for part in path.split("/") if part]
        if parts[:1] == ["v1"]:
            parts = parts[1:]
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
        if parts[:1] == ["artifacts"] and method == "GET" and len(parts) == 2:
            require_scope(principal, "read")
            data = self.services.storage.get(parts[1])
            return {"status": 200, "body": {"sha256": parts[1], "size": len(data), "data_base64": base64.b64encode(data).decode("ascii")}}
        if (
            method == "POST"
            and len(parts) == 3
            and parts[0] == "worlds"
            and parts[2] == "simulate"
        ):
            require_scope(principal, "write")
            body = request.get("body")
            if not isinstance(body, dict):
                raise ValidationError("body must be an object")
            simulation = validate_simulation_spec(body)
            key = headers.get("Idempotency-Key")
            if not isinstance(key, str):
                raise ValidationError("Idempotency-Key is required for writes")

            def execute_simulation() -> tuple[int, dict[str, object]]:
                world = self.services.worlds.get(principal.organization_id, parts[1])
                trajectory = simulate_world(world, simulation)
                self.services.events.append(
                    principal.organization_id,
                    "worlds.simulated",
                    {"id": parts[1], "samples": len(trajectory["time"])},
                )
                return 200, trajectory

            status, response, replayed = self.services.idempotency.execute(
                principal.organization_id,
                key,
                {"method": method, "path": path, "body": simulation},
                execute_simulation,
            )
            return {
                "status": status,
                "headers": {"Idempotency-Replayed": str(replayed).lower()},
                "body": response,
            }
        if (
            method == "POST"
            and len(parts) == 4
            and parts[0] == "worlds"
            and parts[2] == "analysis"
            and parts[3] == "stability"
        ):
            require_scope(principal, "write")
            body = request.get("body")
            if not isinstance(body, dict):
                raise ValidationError("body must be an object")
            stability_request = validate_stability_request(body)
            key = headers.get("Idempotency-Key")
            if not isinstance(key, str):
                raise ValidationError("Idempotency-Key is required for writes")

            def execute_stability() -> tuple[int, dict[str, object]]:
                world = self.services.worlds.get(principal.organization_id, parts[1])
                report = analyze_stability(world, stability_request)
                self.services.events.append(
                    principal.organization_id,
                    "worlds.analyzed",
                    {"id": parts[1], "analysis": "stability", "fixed_points": len(report["fixed_points"])},
                )
                return 200, report

            status, response, replayed = self.services.idempotency.execute(
                principal.organization_id,
                key,
                {"method": method, "path": path, "body": stability_request},
                execute_stability,
            )
            return {
                "status": status,
                "headers": {"Idempotency-Replayed": str(replayed).lower()},
                "body": response,
            }
        if method == "POST" and len(parts) == 3 and parts[0] == "runs" and parts[2] == "cancel":
            require_scope(principal, "write")
            key = headers.get("Idempotency-Key")
            if not isinstance(key, str):
                raise ValidationError("Idempotency-Key is required for writes")

            def cancel_run() -> tuple[int, dict[str, object]]:
                run = self.services.runs.get(principal.organization_id, parts[1])
                if run["status"] in {"succeeded", "failed", "cancelled"}:
                    raise ConflictError("run is already in a terminal state")
                item = self.services.runs.update(principal.organization_id, parts[1], {"status": "cancelled"})
                self.services.events.append(principal.organization_id, "runs.cancelled", {"id": parts[1]})
                return 200, item

            status, response, replayed = self.services.idempotency.execute(
                principal.organization_id, key, {"method": method, "path": path}, cancel_run
            )
            return {"status": status, "headers": {"Idempotency-Replayed": str(replayed).lower()}, "body": response}
        if method == "GET" and len(parts) == 3 and parts[0] == "runs" and parts[2] == "events":
            require_scope(principal, "read")
            self.services.runs.get(principal.organization_id, parts[1])
            events = [
                event
                for event in self.services.events.list(principal.organization_id)
                if isinstance(event.get("payload"), dict) and event["payload"].get("id") == parts[1]
            ]
            return {"status": 200, "body": {"items": events}}
        if not parts or parts[0] not in {"projects", "datasets", "worlds", "runs"} or len(parts) > 2:
            raise NotFoundError("route not found")
        repository = getattr(self.services, parts[0])
        if method == "GET" and len(parts) == 1:
            require_scope(principal, "read")
            query = request.get("query", {})
            if not isinstance(query, dict):
                raise ValidationError("query must be an object")
            items = repository.list(principal.organization_id)
            limit = int(query.get("limit", 20))
            result = page(items, cursor=query.get("cursor"), limit=limit, maximum=self.services.settings.max_page_size)
            return {"status": 200, "body": {"items": result.items, "next_cursor": result.next_cursor, "total": len(items), "limit": limit}}
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
                project_id = body.get("project_id")
                if project_id is not None:
                    if not isinstance(project_id, str):
                        raise ValidationError("project_id must be a string")
                    self.services.projects.get(principal.organization_id, project_id)
                if parts[0] == "runs" and "dataset_id" in body:
                    dataset_id = body["dataset_id"]
                    if not isinstance(dataset_id, str):
                        raise ValidationError("dataset_id must be a string")
                    dataset = self.services.datasets.get(principal.organization_id, dataset_id)
                    _, discovered_spec = discover_world(dataset, body.get("states"), body.get("discovery", {}))
                    world_name = body.get("world_name", f"{body.get('name', 'run')}-world")
                    if not isinstance(world_name, str) or not world_name.strip():
                        raise ValidationError("world_name must be a non-empty string")
                    world = self.services.worlds.create(
                        principal.organization_id,
                        {
                            "name": world_name,
                            "project_id": project_id,
                            "dataset_id": dataset_id,
                            **discovered_spec,
                        },
                    )
                    self.services.events.append(
                        principal.organization_id,
                        "worlds.discovered",
                        {"id": world["id"], "dataset_id": dataset_id},
                    )
                    item = repository.create(
                        principal.organization_id,
                        {**body, "world_id": world["id"], "status": "succeeded"},
                    )
                else:
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
