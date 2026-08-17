"""Transport wiring for the P6 collaboration surface.

This module is to :mod:`collaboration` what :mod:`discovery` is to the domain
run repository: the composition of an in-process, tenant-scoped store with the
service's authentication, scope, role enforcement, and event emission.  It owns

* route matching + handling for the nine collaboration endpoints;
* the server-side role gate wired into the existing project/world mutation
  paths (a viewer can never mutate; only an owner manages membership);
* revision-lineage recording as a side effect of storing a world;
* the approval audit event emitted to the existing :class:`EventBus`.

Local single-user behavior is preserved: a world with no shared project, and the
sole creator/owner of a project, are always treated as fully authorized.  Role
enforcement activates only once a project carries membership -- which happens the
moment a project is created (its creator becomes owner) and members are added.
"""

from __future__ import annotations

import json
import time
from typing import Mapping, Sequence

from lawsynth_server.auth import Principal
from lawsynth_server.errors import AuthorizationError, ServerError

from . import collaboration as collab
from .auth import ApiAuthenticator
from .authorization import READ, WRITE, require_scope_or_problem
from .events import EventBus, EventKind
from .middleware import RequestProblem, error_envelope
from .repositories import ApiRepositories


class CollaborationService:
    """Owns membership, revision lineage, annotations, review, and merge."""

    def __init__(self, auth: ApiAuthenticator, repositories: ApiRepositories, events: EventBus) -> None:
        self._auth = auth
        self._repositories = repositories
        self._events = events
        self._members = collab.MembershipStore()
        self._revisions = collab.RevisionLog()
        self._annotations = collab.AnnotationStore()

    # -- route classification ---------------------------------------------- #

    @staticmethod
    def matches(request: Mapping[str, object], parts: Sequence[str]) -> bool:
        """True when ``parts`` names a collaboration endpoint this module owns."""

        method = request["method"]
        if parts[:1] == ["projects"] and len(parts) >= 3:
            if parts[2] == "members":
                return (len(parts) == 3 and method in {"POST", "GET"}) or (len(parts) == 4 and method == "DELETE")
            if parts[2] == "merge":
                return len(parts) == 3 and method == "POST"
        if parts[:1] == ["worlds"] and len(parts) >= 3:
            if parts[2] == "revisions":
                if len(parts) == 3 and method == "GET":
                    return True
                if len(parts) == 4 and method == "GET":
                    return True
                if len(parts) == 5 and parts[4] == "review" and method == "POST":
                    return True
            if parts[2] == "annotations" and len(parts) == 3:
                return method in {"POST", "GET"}
        return False

    def handle(self, request: Mapping[str, object], parts: Sequence[str], request_id: str) -> dict[str, object]:
        """Authenticate, scope, and dispatch a collaboration request."""

        try:
            principal = self._auth.authenticate_or_problem(request["headers"])
            method = request["method"]
            if parts[0] == "projects":
                return self._handle_project(request, principal, parts, method, request_id)
            return self._handle_world(request, principal, parts, method, request_id)
        except ServerError as error:
            return error_envelope(error.status_code, error.code, error.message, request_id)

    # -- project-scoped routes --------------------------------------------- #

    def _handle_project(
        self, request: Mapping[str, object], principal: Principal, parts: Sequence[str], method: str, request_id: str
    ) -> dict[str, object]:
        project_id = parts[1]
        org = principal.organization_id
        # Existence + tenant isolation come from the domain repository.
        self._repositories.get("projects").get(org, project_id)  # type: ignore[attr-defined]
        if parts[2] == "merge":
            require_scope_or_problem(principal, WRITE)
            self._require_role(org, project_id, principal.subject, collab.role_can_write, "merge requires editor or owner")
            body = request.get("body")
            if not isinstance(body, Mapping):
                raise RequestProblem(422, "validation_error", "merge body must be an object")
            base = body.get("base", body.get("left"))
            incoming = body.get("incoming", body.get("right"))
            result = collab.merge_indexes(base, incoming)
            return {"status": 200, "headers": {"X-Request-ID": request_id}, "body": result}
        # members
        if method == "GET":
            require_scope_or_problem(principal, READ)
            self._require_membership(org, project_id, principal.subject)
            return {
                "status": 200,
                "headers": {"X-Request-ID": request_id},
                "body": {"items": self._members.members(org, project_id)},
            }
        if method == "POST":
            require_scope_or_problem(principal, WRITE)
            self._require_owner(org, project_id, principal.subject)
            body = request.get("body")
            if not isinstance(body, Mapping):
                raise RequestProblem(422, "validation_error", "member body must be an object")
            member = self._members.set_role(org, project_id, body.get("principal"), body.get("role"))
            return {"status": 200, "headers": {"X-Request-ID": request_id}, "body": member}
        # DELETE /members/{principal}
        require_scope_or_problem(principal, WRITE)
        self._require_owner(org, project_id, principal.subject)
        self._members.remove(org, project_id, parts[3])
        return {"status": 204, "headers": {"X-Request-ID": request_id}, "body": {}}

    # -- world-scoped routes ----------------------------------------------- #

    def _handle_world(
        self, request: Mapping[str, object], principal: Principal, parts: Sequence[str], method: str, request_id: str
    ) -> dict[str, object]:
        world_id = parts[1]
        org = principal.organization_id
        world = self._repositories.get("worlds").get(org, world_id)  # type: ignore[attr-defined]
        if parts[2] == "annotations":
            if method == "GET":
                require_scope_or_problem(principal, READ)
                self._require_world_reader(org, principal.subject, world)
                return {
                    "status": 200,
                    "headers": {"X-Request-ID": request_id},
                    "body": {"items": self._annotations.list(org, world_id)},
                }
            require_scope_or_problem(principal, WRITE)
            self._require_world_writer(org, principal.subject, world, "annotating requires editor or owner")
            annotation = collab.validate_annotation(request.get("body"))
            record = self._annotations.append(org, world_id, annotation, actor=principal.subject)
            return {"status": 201, "headers": {"X-Request-ID": request_id}, "body": record}
        # revisions
        if len(parts) == 3:  # GET /revisions
            require_scope_or_problem(principal, READ)
            self._require_world_reader(org, principal.subject, world)
            return {
                "status": 200,
                "headers": {"X-Request-ID": request_id},
                "body": {
                    "items": self._revisions.list(org, world_id),
                    "trusted": self._revisions.world_trusted(org, world_id),
                },
            }
        number = self._revision_number(parts[3])
        if len(parts) == 4:  # GET /revisions/{n}
            require_scope_or_problem(principal, READ)
            self._require_world_reader(org, principal.subject, world)
            record = self._revisions.get(org, world_id, number)
            record["trusted"] = record["review_state"] == "approved"
            return {"status": 200, "headers": {"X-Request-ID": request_id}, "body": record}
        # POST /revisions/{n}/review
        require_scope_or_problem(principal, WRITE)
        role = self._effective_world_role(org, principal.subject, world)
        if not collab.role_can_write(role):
            raise RequestProblem(403, "forbidden", "reviewing requires editor or owner")
        body = request.get("body")
        if not isinstance(body, Mapping):
            raise RequestProblem(422, "validation_error", "review body must be an object")
        target = body.get("state", body.get("to"))
        record = self._revisions.transition(
            org, world_id, number, target if isinstance(target, str) else "", actor=principal.subject, is_owner=role == collab.OWNER
        )
        self._emit_review_audit(org, world_id, record)
        record["trusted"] = record["review_state"] == "approved"
        return {"status": 200, "headers": {"X-Request-ID": request_id}, "body": record}

    # -- role gate wired into the existing mutation paths ------------------- #

    def authorize_mutation_or_problem(self, request: Mapping[str, object], parts: Sequence[str]) -> None:
        """Server-side role check for project/world CRUD before domain dispatch.

        Silent auth: an unauthenticated or foreign-tenant caller is left to the
        domain (which returns 401/404); enforcement only fires for an
        authenticated caller acting on a shared resource in their own tenant.
        """

        principal = self._auth.silent(request["headers"])
        if principal is None:
            return
        method = request["method"]
        org, actor = principal.organization_id, principal.subject
        if parts[:1] == ["projects"] and len(parts) == 2 and method in {"PATCH", "DELETE"}:
            if not self._members.has_members(org, parts[1]):
                return
            if self._members.role(org, parts[1], actor) != collab.OWNER:
                raise RequestProblem(403, "forbidden", "only an owner may modify or delete a project")
            return
        if parts[:1] == ["worlds"]:
            if len(parts) == 1 and method == "POST":
                self._gate_world_create(request, org, actor)
            elif len(parts) == 2 and method in {"PATCH", "DELETE"}:
                self._gate_world_item(org, actor, parts[1])

    def _gate_world_create(self, request: Mapping[str, object], org: str, actor: str) -> None:
        body = request.get("body")
        project_id = body.get("project_id") if isinstance(body, Mapping) else None
        if isinstance(project_id, str) and self._members.has_members(org, project_id):
            role = self._members.role(org, project_id, actor)
            if not collab.role_can_write(role):
                raise RequestProblem(403, "forbidden", "adding a world requires editor or owner")
        # Validate an optional derivation up front so a bad tag is a 422 before
        # the world is created (the revision is recorded post-create).
        if isinstance(body, Mapping) and body.get("derivation") is not None:
            try:
                collab.normalize_derivation(body.get("derivation"), content_hash="0" * 64)
            except ServerError as error:
                raise RequestProblem(error.status_code, error.code, error.message) from error

    def _gate_world_item(self, org: str, actor: str, world_id: str) -> None:
        try:
            world = self._repositories.get("worlds").get(org, world_id)  # type: ignore[attr-defined]
        except ServerError:
            return  # unknown/foreign world -> let the domain return 404
        project_id = world.get("project_id")
        if isinstance(project_id, str) and self._members.has_members(org, project_id):
            role = self._members.role(org, project_id, actor)
            if not collab.role_can_write(role):
                raise RequestProblem(403, "forbidden", "modifying a world requires editor or owner")

    # -- post-dispatch recorders (never break the response) ---------------- #

    def establish_owner(self, request: Mapping[str, object], response: Mapping[str, object]) -> None:
        """After a successful project create, bind the creator as owner."""

        try:
            if request["method"] != "POST" or response.get("status") != 201:
                return
            body = response.get("body")
            principal = self._auth.silent(request["headers"])
            if principal is None or not isinstance(body, Mapping) or not isinstance(body.get("id"), str):
                return
            self._members.establish_owner(principal.organization_id, str(body["id"]), principal.subject)
        except Exception:
            pass

    def record_world_revision(self, request: Mapping[str, object], response: Mapping[str, object]) -> None:
        """Record a revision when a world is stored (create) or re-stored (patch)."""

        try:
            method, status = request["method"], response.get("status")
            body = response.get("body")
            principal = self._auth.silent(request["headers"])
            if principal is None or not isinstance(body, Mapping) or not isinstance(body.get("id"), str):
                return
            org, actor, world_id = principal.organization_id, principal.subject, str(body["id"])
            content_hash = collab.world_content_hash(body)
            if method == "POST" and status == 201:
                raw = request.get("body")
                raw_derivation = raw.get("derivation") if isinstance(raw, Mapping) else None
                derivation, parents = collab.normalize_derivation(raw_derivation, content_hash=content_hash)
            elif method == "PATCH" and status == 200:
                previous = self._revisions.list(org, world_id)
                parent_number = len(previous)
                changed = sorted(request["body"].keys()) if isinstance(request.get("body"), Mapping) else []
                if parent_number == 0:
                    derivation, parents = collab.normalize_derivation(None, content_hash=content_hash)
                else:
                    parent = {"world_id": world_id, "number": parent_number}
                    derivation = {"kind": "edited", "parent": parent, "ops": changed}
                    parents = [parent]
            else:
                return
            parents = self._link_parents(org, parents)
            self._revisions.append(
                org, world_id, content_hash=content_hash, derivation=derivation, parents=parents, actor=actor
            )
        except Exception:
            pass

    def _link_parents(self, org: str, parents: list[dict[str, object]]) -> list[dict[str, object]]:
        linked: list[dict[str, object]] = []
        for parent in parents:
            enriched = dict(parent)
            world_id, number = parent.get("world_id"), parent.get("number")
            if isinstance(world_id, str) and isinstance(number, int):
                resolved = self._revisions.content_hash_of(org, world_id, number)
                if resolved is not None:
                    enriched["content_hash"] = resolved
            linked.append(enriched)
        return linked

    # -- authorization helpers --------------------------------------------- #

    def _effective_world_role(self, org: str, actor: str, world: Mapping[str, object]) -> str | None:
        """The caller's role for a world.

        A world inside a shared project resolves to the caller's project role
        (``None`` when they are not a member).  A standalone world -- the local
        single-user case -- treats the actor as owner of their own tenant data.
        """

        project_id = world.get("project_id")
        if isinstance(project_id, str) and self._members.has_members(org, project_id):
            return self._members.role(org, project_id, actor)
        return collab.OWNER

    def _require_world_reader(self, org: str, actor: str, world: Mapping[str, object]) -> None:
        if self._effective_world_role(org, actor, world) is None:
            raise RequestProblem(403, "forbidden", "not a member of this world's project")

    def _require_world_writer(self, org: str, actor: str, world: Mapping[str, object], message: str) -> None:
        if not collab.role_can_write(self._effective_world_role(org, actor, world)):
            raise RequestProblem(403, "forbidden", message)

    def _require_owner(self, org: str, project_id: str, actor: str) -> None:
        if self._members.role(org, project_id, actor) != collab.OWNER:
            raise RequestProblem(403, "forbidden", "only an owner may manage membership")

    def _require_membership(self, org: str, project_id: str, actor: str) -> None:
        if self._members.role(org, project_id, actor) is None:
            raise RequestProblem(403, "forbidden", "not a member of this project")

    def _require_role(self, org: str, project_id: str, actor: str, predicate, message: str) -> None:
        if not predicate(self._members.role(org, project_id, actor)):
            raise RequestProblem(403, "forbidden", message)

    # -- helpers ------------------------------------------------------------ #

    @staticmethod
    def _revision_number(raw: str) -> int:
        if not raw.isdigit():
            raise RequestProblem(422, "validation_error", "revision number must be a positive integer")
        number = int(raw)
        if number < 1:
            raise RequestProblem(422, "validation_error", "revision number must be a positive integer")
        return number

    def _emit_review_audit(self, org: str, world_id: str, record: Mapping[str, object]) -> None:
        history = record.get("review_history")
        last = history[-1] if isinstance(history, list) and history else {}
        payload = {
            "world_id": world_id,
            "revision": record.get("number"),
            "from": last.get("from"),
            "to": last.get("to"),
            "actor": last.get("actor"),
        }
        self._events.append(
            org,
            int(time.time() * 1000),
            EventKind.REVISION_REVIEWED,
            json.dumps(payload, separators=(",", ":"), allow_nan=False),
        )
