#!/usr/bin/env python3
"""Drive the LawSynth P6 collaboration surface from Python --- offline.

This stands up the real ``lawsynth_api`` WSGI application in-process (a temp
SQLite metadata store, a temp object root, and THREE bearer tokens in one tenant
that resolve to three distinct principals), then uses :class:`lawsynth.Client`
--- one instance per principal, all talking to the same app object through the
in-process WSGI transport --- to exercise the shared, multi-user workspace:

    owner creates a project (becomes owner) -> owner adds an editor + a viewer ->
    editor submits a world (records revision 1) -> a viewer is refused (role
    gate) -> list the revision lineage -> editor annotates -> editor requests
    review, is refused approval (owner-only) -> owner approves (world trusted) ->
    a deterministic merge surfaces a content-hash conflict.

Roles are decided by project membership, not token scope: every token carries
read+write, so a viewer's 403 proves role enforcement is evaluated server-side
against the caller's role for that specific project. No socket is opened, no
native engine is required, and the run is deterministic. Run it with::

    PYTHONPATH="python/lawsynth/src:services/api/src:python/lawsynth-server/src" \
        python3 python/lawsynth/examples/collaboration_client.py
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import lawsynth
from lawsynth import ApiError
from lawsynth_api import ApiSettings, create_wsgi_app
from lawsynth_server.settings import Settings as ServerSettings

# Three 32-char demo tokens (>= 16 required) in ONE tenant ("acme"). Distinct
# 8-char prefixes -> distinct Principal.subject values, so they are three people.
OWNER_TOKEN = "0123456789abcdef0123456789abcdef"
EDITOR_TOKEN = "editoraaaaaaaaaaaaaaaaaaaaaaaaaa"
VIEWER_TOKEN = "viewerbbbbbbbbbbbbbbbbbbbbbbbbbb"

# The service derives a principal's identity as ``token:<first 8 chars>``.
EDITOR = f"token:{EDITOR_TOKEN[:8]}"
VIEWER = f"token:{VIEWER_TOKEN[:8]}"

_SCOPES = frozenset({"read", "write"})


def build_app(root: Path):
    """Construct the real API WSGI app over a temp SQLite store and object root."""
    server = ServerSettings(
        database_url=f"sqlite:///{root / 'metadata.sqlite3'}",
        object_root=root / "objects",
        tokens={
            OWNER_TOKEN: ("acme", _SCOPES),
            EDITOR_TOKEN: ("acme", _SCOPES),
            VIEWER_TOKEN: ("acme", _SCOPES),
        },
        max_upload_bytes=8 * 1024 * 1024,
    )
    return create_wsgi_app(
        ApiSettings(server=server, environment="test", max_request_bytes=8 * 1024 * 1024)
    )


def _expect_forbidden(label: str, action) -> None:
    """Run ``action`` expecting a 403 role-gate rejection; print the outcome."""
    try:
        action()
    except ApiError as error:
        print(f"gate:      {label} -> refused [{error.status} {error.code}] {error.message}")
        return
    raise SystemExit(f"expected {label!r} to be refused, but it succeeded")


def main() -> None:
    workdir = Path(tempfile.mkdtemp(prefix="lawsynth-collab-"))
    app = build_app(workdir)

    # One client per principal, all against the same in-process app.
    owner = lawsynth.Client(wsgi_app=app, token=OWNER_TOKEN)
    editor = lawsynth.Client(wsgi_app=app, token=EDITOR_TOKEN)
    viewer = lawsynth.Client(wsgi_app=app, token=VIEWER_TOKEN)

    print("LawSynth collaboration client --- offline in-process transcript")
    print("=" * 64)

    # 1. The owner creates a shared project and becomes its sole owner.
    project = owner.create_project("shared-lab")
    project_id = str(project["id"])
    members = owner.list_members(project_id)
    print(f"project:   {project_id} created; members {[m['role'] for m in members]}")

    # 2. The owner binds an editor and a viewer.
    owner.add_member(project_id, EDITOR, "editor")
    owner.add_member(project_id, VIEWER, "viewer")
    roles = {m["principal"]: m["role"] for m in owner.list_members(project_id)}
    print(f"members:   {roles}")

    # 3. A viewer may NOT add a world; an editor may (role gate, server-side).
    _expect_forbidden(
        "viewer submits world",
        lambda: viewer.create_world(
            name="viewer-world", states=["x"], equations={"x": "-rate * x"},
            parameters={"rate": 0.2}, project_id=project_id,
        ),
    )
    world = editor.create_world(
        name="decay", states=["x"], equations={"x": "-rate * x"},
        parameters={"rate": 0.2}, project_id=project_id,
    )
    world_id = str(world["id"])
    print(f"submit:    editor stored world {world_id!r} (revision recorded)")

    # 4. The revision lineage is immutable and starts in 'draft'.
    revisions = editor.list_revisions(world_id)
    rev = revisions["items"][0]
    print(
        f"revisions: {len(revisions['items'])} revision(s); #{rev['number']} "
        f"derivation={rev['derivation']['kind']} state={rev['review_state']} "
        f"trusted={revisions['trusted']}"
    )

    # 5. Annotations respect roles: a viewer is refused; an editor annotates.
    _expect_forbidden(
        "viewer annotates",
        lambda: viewer.add_annotation(world_id, "can I edit?", target="world"),
    )
    annotation = editor.add_annotation(world_id, "looks linear in x", target="law", target_ref="x")
    print(f"annotate:  editor added annotation #{annotation['ordinal']} on target={annotation['target']}({annotation['ref']})")
    listed = viewer.list_annotations(world_id)
    print(f"read:      viewer sees {len(listed)} annotation(s): {[a['text'] for a in listed]}")

    # 6. Review state machine: editor requests review, cannot approve; owner approves.
    moved = editor.review_revision(world_id, 1, "in_review")
    print(f"review:    editor moved revision 1 -> {moved['review_state']!r}")
    _expect_forbidden(
        "editor approves",
        lambda: editor.review_revision(world_id, 1, "approved"),
    )
    approved = owner.review_revision(world_id, 1, "approved")
    print(f"approve:   owner moved revision 1 -> {approved['review_state']!r}")
    trusted_rev = owner.get_revision(world_id, 1)
    print(f"trusted:   revision 1 trusted={trusted_rev['trusted']}")

    # 7. Deterministic workspace merge surfaces a content-hash conflict.
    base = [
        {"name": "decay", "content_hash": "h-decay", "revision": 1},
        {"name": "oscillator", "content_hash": "h-osc", "revision": 2},
    ]
    incoming = [
        {"name": "decay", "content_hash": "h-decay-EDITED", "revision": 1},  # same name, differing hash
        {"name": "growth", "content_hash": "h-growth", "revision": 1},
    ]
    merge = owner.merge_project(project_id, base, incoming)
    print(
        f"merge:     merged={[r['name'] for r in merge['merged']]} "
        f"conflicts={[c['name'] for c in merge['conflicts']]} "
        f"(conflict_count={merge['conflict_count']})"
    )
    for conflict in merge["conflicts"]:
        print(
            f"           conflict on {conflict['name']!r}: "
            f"base={conflict['base']['content_hash']} vs incoming={conflict['incoming']['content_hash']}"
        )

    print("=" * 64)
    print("done: roles -> submit -> revisions -> annotate -> review/approve -> merge, fully offline.")

    app.close()


if __name__ == "__main__":
    main()
