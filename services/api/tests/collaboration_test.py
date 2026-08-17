"""Live WSGI tests for the P6 collaboration surface (specs/collaboration).

These drive the real transport with THREE principals in one tenant (``acme``)
holding different project roles -- owner, editor, viewer -- plus an outsider and
a foreign tenant.  Roles are decided by project membership, not token scope: all
tokens carry read+write, so a viewer's 403 proves role enforcement is evaluated
server-side against the caller's role for that specific project.
"""

from __future__ import annotations

import io
import json

from _harness import TOKEN, TOKEN_GLOBEX, auth, make_app, request

# Distinct 8-char prefixes -> distinct Principal.subject values in one tenant.
OWNER_TOKEN = TOKEN  # acme; the project creator becomes owner automatically
EDITOR_TOKEN = "editoraaaaaaaaaaaaaaaaaaaaaaaaaa"
VIEWER_TOKEN = "viewerbbbbbbbbbbbbbbbbbbbbbbbbbb"
OUTSIDER_TOKEN = "outsidercccccccccccccccccccccccc"

OWNER = f"token:{OWNER_TOKEN[:8]}"
EDITOR = f"token:{EDITOR_TOKEN[:8]}"
VIEWER = f"token:{VIEWER_TOKEN[:8]}"

_ACME = frozenset({"read", "write"})
_EXTRA = {
    EDITOR_TOKEN: ("acme", _ACME),
    VIEWER_TOKEN: ("acme", _ACME),
    OUTSIDER_TOKEN: ("acme", _ACME),
    TOKEN_GLOBEX: ("globex", _ACME),
}


def _world(name: str, project_id: str | None = None, derivation: object | None = None) -> dict:
    body: dict = {
        "name": name,
        "states": ["x"],
        "controls": [],
        "parameters": {"rate": 0.2},
        "equations": {"x": "-rate * x"},
    }
    if project_id is not None:
        body["project_id"] = project_id
    if derivation is not None:
        body["derivation"] = derivation
    return body


def _app(tmp_path):
    return make_app(tmp_path, extra_tokens=_EXTRA, max_bytes=100_000)


def _tok(token, key=None):
    return auth(token=token, key=key)


def _project(app, name="collab"):
    status, _, project = request(app, "POST", "/v1/projects", body={"name": name}, headers=_tok(OWNER_TOKEN, "p-1"))
    assert status == 201
    return project["id"]


def _add(app, project_id, principal, role, key):
    return request(
        app,
        "POST",
        f"/v1/projects/{project_id}/members",
        body={"principal": principal, "role": role},
        headers=_tok(OWNER_TOKEN, key),
    )


# --------------------------------------------------------------------------- #
# Membership & roles                                                          #
# --------------------------------------------------------------------------- #


def test_creator_becomes_owner_and_can_manage_membership(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app)
        # The creator is registered as the sole owner.
        status, _, members = request(app, "GET", f"/v1/projects/{pid}/members", headers=_tok(OWNER_TOKEN))
        assert status == 200 and members["items"] == [{"principal": OWNER, "role": "owner"}]

        assert _add(app, pid, EDITOR, "editor", "m-1")[0] == 200
        assert _add(app, pid, VIEWER, "viewer", "m-2")[0] == 200
        status, _, members = request(app, "GET", f"/v1/projects/{pid}/members", headers=_tok(OWNER_TOKEN))
        roles = {m["principal"]: m["role"] for m in members["items"]}
        assert roles == {OWNER: "owner", EDITOR: "editor", VIEWER: "viewer"}
    finally:
        app.close()


def test_only_owner_manages_membership(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app)
        _add(app, pid, EDITOR, "editor", "m-1")
        _add(app, pid, VIEWER, "viewer", "m-2")

        # Editor and viewer cannot add/remove members.
        status, _, body = request(
            app, "POST", f"/v1/projects/{pid}/members", body={"principal": "x", "role": "editor"}, headers=_tok(EDITOR_TOKEN, "e-1")
        )
        assert status == 403 and body["error"]["code"] == "forbidden"
        status, _, _ = request(
            app, "POST", f"/v1/projects/{pid}/members", body={"principal": "x", "role": "viewer"}, headers=_tok(VIEWER_TOKEN, "v-1")
        )
        assert status == 403
        status, _, _ = request(app, "DELETE", f"/v1/projects/{pid}/members/{VIEWER}", headers=_tok(EDITOR_TOKEN, "e-2"))
        assert status == 403

        # Owner removes the viewer.
        status, _, _ = request(app, "DELETE", f"/v1/projects/{pid}/members/{VIEWER}", headers=_tok(OWNER_TOKEN, "o-1"))
        assert status == 204
        status, _, members = request(app, "GET", f"/v1/projects/{pid}/members", headers=_tok(OWNER_TOKEN))
        assert VIEWER not in {m["principal"] for m in members["items"]}
    finally:
        app.close()


def test_cannot_remove_last_owner(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app)
        status, _, body = request(app, "DELETE", f"/v1/projects/{pid}/members/{OWNER}", headers=_tok(OWNER_TOKEN, "o-2"))
        assert status == 409 and body["error"]["code"] == "conflict"
    finally:
        app.close()


def test_non_member_cannot_read_membership(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app)
        status, _, body = request(app, "GET", f"/v1/projects/{pid}/members", headers=_tok(OUTSIDER_TOKEN))
        assert status == 403 and body["error"]["code"] == "forbidden"
    finally:
        app.close()


def test_role_gate_on_project_and_world_mutation(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app)
        _add(app, pid, EDITOR, "editor", "m-1")
        _add(app, pid, VIEWER, "viewer", "m-2")

        # Viewer cannot add a world to the shared project.
        status, _, body = request(app, "POST", "/v1/worlds", body=_world("v-world", pid), headers=_tok(VIEWER_TOKEN, "vw-1"))
        assert status == 403 and body["error"]["code"] == "forbidden"

        # Editor can add a world.
        status, _, world = request(app, "POST", "/v1/worlds", body=_world("e-world", pid), headers=_tok(EDITOR_TOKEN, "ew-1"))
        assert status == 201
        world_id = world["id"]

        # Viewer cannot modify or delete it; editor can patch it.
        status, _, _ = request(app, "PATCH", f"/v1/worlds/{world_id}", body={"metadata": {"n": 1}}, headers=_tok(VIEWER_TOKEN, "vp-1"))
        assert status == 403
        status, _, _ = request(app, "PATCH", f"/v1/worlds/{world_id}", body={"metadata": {"n": 2}}, headers=_tok(EDITOR_TOKEN, "ep-1"))
        assert status == 200

        # Only the owner may modify/delete the project itself.
        status, _, _ = request(app, "PATCH", f"/v1/projects/{pid}", body={"metadata": {"k": 1}}, headers=_tok(EDITOR_TOKEN, "epr-1"))
        assert status == 403
        status, _, _ = request(app, "PATCH", f"/v1/projects/{pid}", body={"metadata": {"k": 2}}, headers=_tok(OWNER_TOKEN, "opr-1"))
        assert status == 200
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# Revision lineage                                                            #
# --------------------------------------------------------------------------- #


def test_revision_lineage_records_derivation_and_parents(tmp_path):
    app = _app(tmp_path)
    try:
        _, _, world_a = request(app, "POST", "/v1/worlds", body=_world("decay-a"), headers=_tok(OWNER_TOKEN, "wa-1"))
        a_id = world_a["id"]

        status, _, revs = request(app, "GET", f"/v1/worlds/{a_id}/revisions", headers=_tok(OWNER_TOKEN))
        assert status == 200 and len(revs["items"]) == 1
        rev1 = revs["items"][0]
        assert rev1["number"] == 1 and rev1["derivation"]["kind"] == "imported"
        a_hash = rev1["content_hash"]

        # A derived (edited) world references A's revision 1 as its parent.
        derivation = {"kind": "edited", "parent": {"world_id": a_id, "number": 1}, "ops": ["retune"]}
        _, _, world_b = request(app, "POST", "/v1/worlds", body=_world("decay-b", derivation=derivation), headers=_tok(OWNER_TOKEN, "wb-1"))
        b_id = world_b["id"]

        status, _, b_rev = request(app, "GET", f"/v1/worlds/{b_id}/revisions/1", headers=_tok(OWNER_TOKEN))
        assert status == 200
        assert b_rev["derivation"]["kind"] == "edited"
        assert b_rev["parents"][0]["world_id"] == a_id
        assert b_rev["parents"][0]["content_hash"] == a_hash  # parent linked by content hash

        # Prior revision remains retrievable and immutable.
        status, _, a_rev = request(app, "GET", f"/v1/worlds/{a_id}/revisions/1", headers=_tok(OWNER_TOKEN))
        assert status == 200 and a_rev["content_hash"] == a_hash
    finally:
        app.close()


def test_patch_appends_monotonic_revision(tmp_path):
    app = _app(tmp_path)
    try:
        _, _, world = request(app, "POST", "/v1/worlds", body=_world("decay-p"), headers=_tok(OWNER_TOKEN, "wp-1"))
        world_id = world["id"]
        request(app, "PATCH", f"/v1/worlds/{world_id}", body={"metadata": {"note": "x"}}, headers=_tok(OWNER_TOKEN, "wp-2"))

        status, _, revs = request(app, "GET", f"/v1/worlds/{world_id}/revisions", headers=_tok(OWNER_TOKEN))
        assert status == 200 and [r["number"] for r in revs["items"]] == [1, 2]
        assert revs["items"][1]["derivation"]["kind"] == "edited"
        assert revs["items"][1]["derivation"]["parent"]["number"] == 1

        # Unknown revision number is a 404; malformed number is a 422.
        assert request(app, "GET", f"/v1/worlds/{world_id}/revisions/9", headers=_tok(OWNER_TOKEN))[0] == 404
        assert request(app, "GET", f"/v1/worlds/{world_id}/revisions/abc", headers=_tok(OWNER_TOKEN))[0] == 422
    finally:
        app.close()


def test_bad_derivation_is_rejected_before_world_is_created(tmp_path):
    app = _app(tmp_path)
    try:
        status, _, body = request(
            app, "POST", "/v1/worlds", body=_world("bad", derivation={"kind": "edited"}), headers=_tok(OWNER_TOKEN, "bd-1")
        )
        assert status == 422 and body["error"]["code"] == "validation_error"
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# Annotations & review                                                        #
# --------------------------------------------------------------------------- #


def _shared_world(app):
    pid = _project(app, "review-proj")
    _add(app, pid, EDITOR, "editor", "m-1")
    _add(app, pid, VIEWER, "viewer", "m-2")
    _, _, world = request(app, "POST", "/v1/worlds", body=_world("review-world", pid), headers=_tok(OWNER_TOKEN, "rw-1"))
    return pid, world["id"]


def test_annotations_respect_roles(tmp_path):
    app = _app(tmp_path)
    try:
        _, world_id = _shared_world(app)
        # Viewer cannot annotate.
        status, _, body = request(
            app, "POST", f"/v1/worlds/{world_id}/annotations", body={"text": "nope"}, headers=_tok(VIEWER_TOKEN, "an-v")
        )
        assert status == 403 and body["error"]["code"] == "forbidden"

        # Editor annotates a specific law; viewer may read.
        status, _, ann = request(
            app,
            "POST",
            f"/v1/worlds/{world_id}/annotations",
            body={"target": "law", "ref": "x", "text": "looks linear"},
            headers=_tok(EDITOR_TOKEN, "an-e"),
        )
        assert status == 201 and ann["ordinal"] == 1 and ann["actor"] == EDITOR and ann["target"] == "law"

        status, _, listed = request(app, "GET", f"/v1/worlds/{world_id}/annotations", headers=_tok(VIEWER_TOKEN))
        assert status == 200 and len(listed["items"]) == 1 and listed["items"][0]["text"] == "looks linear"
    finally:
        app.close()


def test_review_state_machine_and_owner_only_approval(tmp_path):
    app = _app(tmp_path)
    try:
        _, world_id = _shared_world(app)
        base = f"/v1/worlds/{world_id}/revisions/1/review"

        # Editor moves draft -> in_review.
        status, _, rev = request(app, "POST", base, body={"state": "in_review"}, headers=_tok(EDITOR_TOKEN, "rv-1"))
        assert status == 200 and rev["review_state"] == "in_review"

        # Editor cannot approve; only the owner may.
        status, _, body = request(app, "POST", base, body={"state": "approved"}, headers=_tok(EDITOR_TOKEN, "rv-2"))
        assert status == 403 and body["error"]["code"] == "forbidden"

        status, _, rev = request(app, "POST", base, body={"state": "approved"}, headers=_tok(OWNER_TOKEN, "rv-3"))
        assert status == 200 and rev["review_state"] == "approved"

        # A trusted world references an approved revision.
        status, _, got = request(app, "GET", f"/v1/worlds/{world_id}/revisions/1", headers=_tok(OWNER_TOKEN))
        assert got["trusted"] is True
        status, _, revs = request(app, "GET", f"/v1/worlds/{world_id}/revisions", headers=_tok(OWNER_TOKEN))
        assert revs["trusted"] is True

        # Approved is terminal: an illegal transition is a 409.
        status, _, body = request(app, "POST", base, body={"state": "in_review"}, headers=_tok(OWNER_TOKEN, "rv-4"))
        assert status == 409 and body["error"]["code"] == "conflict"
    finally:
        app.close()


def _sse(app, token):
    environ = {
        "REQUEST_METHOD": "GET",
        "PATH_INFO": "/v1/events",
        "QUERY_STRING": "",
        "CONTENT_LENGTH": "0",
        "wsgi.input": io.BytesIO(b""),
        "HTTP_ACCEPT": "text/event-stream",
        "HTTP_AUTHORIZATION": f"Bearer {token}",
    }
    captured: dict = {}
    payload = b"".join(app(environ, lambda status, hs: captured.update(status=status)))
    frames = []
    for block in payload.decode("utf-8").split("\n\n"):
        block = block.strip()
        if not block or block.startswith(":"):
            continue
        frame = {}
        for line in block.splitlines():
            field, _, value = line.partition(": ")
            frame[field] = value
        if "data" in frame:
            frame["data"] = json.loads(frame["data"])
        frames.append(frame)
    return frames


def test_review_transitions_emit_audit_events(tmp_path):
    app = _app(tmp_path)
    try:
        _, world_id = _shared_world(app)
        base = f"/v1/worlds/{world_id}/revisions/1/review"
        request(app, "POST", base, body={"state": "in_review"}, headers=_tok(EDITOR_TOKEN, "au-1"))
        request(app, "POST", base, body={"state": "approved"}, headers=_tok(OWNER_TOKEN, "au-2"))

        frames = _sse(app, OWNER_TOKEN)
        audits = [f for f in frames if f.get("event") == "revision_reviewed"]
        assert len(audits) == 2
        # Strictly increasing per-tenant sequence.
        ids = [int(f["id"]) for f in audits]
        assert ids == sorted(ids) and len(set(ids)) == 2
        assert audits[-1]["data"]["project_id"] == "acme"  # tenant scope
        last_audit = json.loads(audits[-1]["data"]["payload"])
        assert last_audit["to"] == "approved" and last_audit["actor"] == OWNER

        # Tenant isolation: globex never observes acme's audit stream.
        assert _sse(app, TOKEN_GLOBEX) == []
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# Deterministic workspace merge                                              #
# --------------------------------------------------------------------------- #


def _merge(app, pid, base, incoming, token=OWNER_TOKEN, key="mg"):
    return request(app, "POST", f"/v1/projects/{pid}/merge", body={"base": base, "incoming": incoming}, headers=_tok(token, key))


def test_merge_is_deterministic_and_surfaces_conflicts(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app, "merge-proj")
        _add(app, pid, EDITOR, "editor", "m-1")
        _add(app, pid, VIEWER, "viewer", "m-2")

        a = {"name": "a", "content_hash": "h1", "revision": 1}
        b = {"name": "b", "content_hash": "h2", "revision": 1}

        # Disjoint names union, sorted, no conflicts -- commutative.
        status, _, left = _merge(app, pid, [a], [b], key="mg-1")
        assert status == 200 and [r["name"] for r in left["merged"]] == ["a", "b"] and left["conflicts"] == []
        _, _, right = _merge(app, pid, [b], [a], key="mg-2")
        assert right["merged"] == left["merged"]  # order-independent

        # Same name + same content hash -> higher revision wins, no conflict.
        a2 = {"name": "a", "content_hash": "h1", "revision": 2}
        _, _, same = _merge(app, pid, [a], [a2], key="mg-3")
        assert same["conflict_count"] == 0 and same["merged"][0]["revision"] == 2

        # Same name + differing content hash -> conflict with both revisions.
        a_conflict = {"name": "a", "content_hash": "hX", "revision": 1}
        _, _, conflicted = _merge(app, pid, [a], [a_conflict], key="mg-4")
        assert conflicted["conflict_count"] == 1
        assert conflicted["merged"] == []
        conflict = conflicted["conflicts"][0]
        assert conflict["name"] == "a"
        assert conflict["base"]["content_hash"] == "h1" and conflict["incoming"]["content_hash"] == "hX"
    finally:
        app.close()


def test_merge_requires_editor_or_owner(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app, "merge-roles")
        _add(app, pid, VIEWER, "viewer", "m-2")
        status, _, body = _merge(app, pid, [], [], token=VIEWER_TOKEN, key="mg-v")
        assert status == 403 and body["error"]["code"] == "forbidden"
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# Offline / local single-user guarantee                                       #
# --------------------------------------------------------------------------- #


def test_local_single_user_world_flow_still_works(tmp_path):
    app = _app(tmp_path)
    try:
        # A lone user with a standalone world (no shared project) is treated as
        # owner of their own tenant data: they annotate and approve freely.
        _, _, world = request(app, "POST", "/v1/worlds", body=_world("solo"), headers=_tok(OWNER_TOKEN, "s-1"))
        world_id = world["id"]
        assert request(
            app, "POST", f"/v1/worlds/{world_id}/annotations", body={"text": "mine"}, headers=_tok(OWNER_TOKEN, "s-2")
        )[0] == 201
        assert request(
            app, "POST", f"/v1/worlds/{world_id}/revisions/1/review", body={"state": "in_review"}, headers=_tok(OWNER_TOKEN, "s-3")
        )[0] == 200
        status, _, rev = request(
            app, "POST", f"/v1/worlds/{world_id}/revisions/1/review", body={"state": "approved"}, headers=_tok(OWNER_TOKEN, "s-4")
        )
        assert status == 200 and rev["review_state"] == "approved"
    finally:
        app.close()


def test_collaboration_is_tenant_isolated(tmp_path):
    app = _app(tmp_path)
    try:
        pid = _project(app, "iso")
        _, _, world = request(app, "POST", "/v1/worlds", body=_world("iso-world"), headers=_tok(OWNER_TOKEN, "iw-1"))
        # A foreign tenant sees neither the project's membership nor world revisions.
        assert request(app, "GET", f"/v1/projects/{pid}/members", headers=_tok(TOKEN_GLOBEX))[0] == 404
        assert request(app, "GET", f"/v1/worlds/{world['id']}/revisions", headers=_tok(TOKEN_GLOBEX))[0] == 404
    finally:
        app.close()
