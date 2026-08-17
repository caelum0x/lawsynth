"""Cross-tenant isolation on every resource path (hosted-platform P10).

The load-bearing guarantee of a hosted deployment is that a token for tenant A
can never observe or mutate tenant B's resources.  These tests drive the real
WSGI transport with two tenants (``acme`` and ``globex``) and assert, for every
resource -- projects, datasets, worlds, runs, artifacts, events -- that a
foreign token is refused with 404/403 and never sees the other tenant's content.

Isolation is enforced in the domain repositories (every call keyed by
``organization_id``, a mismatch surfaces as ``NotFoundError`` -> 404); the
artifact path additionally required an API-side ownership gate, because the
object store is content-addressed and a hash alone is not a grant.
"""

from __future__ import annotations

from _harness import TOKEN, TOKEN_GLOBEX, auth, make_app, request

_GLOBEX = {TOKEN_GLOBEX: ("globex", frozenset({"read", "write"}))}
_WORLD = {
    "name": "decay",
    "states": ["x"],
    "controls": [],
    "parameters": {"rate": 0.2},
    "equations": {"x": "-rate * x"},
}
_ARTIFACT = {"data_base64": "c2VjcmV0LWJ5dGVz", "media_type": "text/plain"}


def _app(tmp_path):
    return make_app(tmp_path, extra_tokens=_GLOBEX, max_bytes=100_000)


def _acme(key: str | None = None):
    return auth(token=TOKEN, key=key)


def _globex(key: str | None = None):
    return auth(token=TOKEN_GLOBEX, key=key)


# --------------------------------------------------------------------------- #
# projects / datasets / worlds: item read, list, mutate isolation             #
# --------------------------------------------------------------------------- #


def test_projects_are_isolated_across_tenants(tmp_path):
    app = _app(tmp_path)
    try:
        _, _, project = request(app, "POST", "/v1/projects", body={"name": "acme-proj"}, headers=_acme("p-1"))
        identifier = project["id"]

        # Foreign read of the item is a 404, not a leak.
        status, _, body = request(app, "GET", f"/v1/projects/{identifier}", headers=_globex())
        assert status == 404 and body["error"]["code"] == "not_found"

        # Foreign list never contains it.
        status, _, listed = request(app, "GET", "/v1/projects", headers=_globex())
        assert status == 200 and listed["items"] == []

        # Foreign mutation and deletion are 404 as well.
        status, _, _ = request(app, "PATCH", f"/v1/projects/{identifier}", body={"name": "hijacked"}, headers=_globex("p-x"))
        assert status == 404
        status, _, _ = request(app, "DELETE", f"/v1/projects/{identifier}", headers=_globex("p-d"))
        assert status == 404

        # The owner still sees it intact.
        status, _, owner_view = request(app, "GET", f"/v1/projects/{identifier}", headers=_acme())
        assert status == 200 and owner_view["name"] == "acme-proj"
    finally:
        app.close()


def test_datasets_are_isolated_across_tenants(tmp_path):
    app = _app(tmp_path)
    try:
        _, _, dataset = request(
            app, "POST", "/v1/datasets", body={"name": "acme-ds", "schema": ["t", "x"]}, headers=_acme("d-1")
        )
        identifier = dataset["id"]

        status, _, body = request(app, "GET", f"/v1/datasets/{identifier}", headers=_globex())
        assert status == 404 and body["error"]["code"] == "not_found"

        status, _, listed = request(app, "GET", "/v1/datasets", headers=_globex())
        assert status == 200 and listed["items"] == []

        status, _, _ = request(app, "DELETE", f"/v1/datasets/{identifier}", headers=_globex("d-d"))
        assert status == 404
    finally:
        app.close()


def test_worlds_are_isolated_across_tenants(tmp_path):
    app = _app(tmp_path)
    try:
        _, _, world = request(app, "POST", "/v1/worlds", body=_WORLD, headers=_acme("w-1"))
        identifier = world["id"]

        status, _, body = request(app, "GET", f"/v1/worlds/{identifier}", headers=_globex())
        assert status == 404 and body["error"]["code"] == "not_found"

        status, _, listed = request(app, "GET", "/v1/worlds", headers=_globex())
        assert status == 200 and listed["items"] == []

        # Product surfaces that resolve a world are scoped too: explain is a 404.
        status, _, _ = request(app, "GET", f"/v1/worlds/{identifier}/explain", headers=_globex())
        assert status == 404
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# runs: item, list, cancel, run-events, run-world isolation                    #
# --------------------------------------------------------------------------- #


def test_runs_are_isolated_across_tenants(tmp_path):
    app = _app(tmp_path)
    try:
        _, _, run = request(app, "POST", "/v1/runs", body={"name": "acme-run"}, headers=_acme("r-1"))
        identifier = run["id"]

        status, _, body = request(app, "GET", f"/v1/runs/{identifier}", headers=_globex())
        assert status == 404 and body["error"]["code"] == "not_found"

        status, _, listed = request(app, "GET", "/v1/runs", headers=_globex())
        assert status == 200 and listed["items"] == []

        # A foreign cancel, run-event journal read, and run-world read are all 404.
        status, _, _ = request(app, "POST", f"/v1/runs/{identifier}/cancel", headers=_globex("c-x"))
        assert status == 404
        status, _, _ = request(app, "GET", f"/v1/runs/{identifier}/events", headers=_globex())
        assert status == 404
        status, _, _ = request(app, "GET", f"/v1/runs/{identifier}/world", headers=_globex())
        assert status == 404
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# artifacts: the content-addressed store must not cross a tenant boundary      #
# --------------------------------------------------------------------------- #


def test_artifact_download_is_tenant_scoped_even_though_storage_is_deduplicated(tmp_path):
    app = _app(tmp_path)
    try:
        # acme stores an artifact and can read it back.
        status, _, created = request(app, "POST", "/v1/artifacts", body=_ARTIFACT, headers=_acme("a-1"))
        assert status == 201
        sha = created["sha256"]
        status, _, fetched = request(app, "GET", f"/v1/artifacts/{sha}", headers=_acme())
        assert status == 200 and fetched["sha256"] == sha

        # globex knows the exact (valid, present) hash but never stored it: 404.
        # This is the leak the ownership gate closes -- a hash is not a grant.
        status, _, body = request(app, "GET", f"/v1/artifacts/{sha}", headers=_globex())
        assert status == 404 and body["error"]["code"] == "not_found"

        # Dedup is preserved and honest: if globex stores the SAME bytes it owns
        # the (shared) blob independently and may then read it.
        status, _, globex_created = request(app, "POST", "/v1/artifacts", body=_ARTIFACT, headers=_globex("a-2"))
        assert status == 201 and globex_created["sha256"] == sha
        status, _, globex_fetched = request(app, "GET", f"/v1/artifacts/{sha}", headers=_globex())
        assert status == 200 and globex_fetched["sha256"] == sha
    finally:
        app.close()


def test_artifact_download_unowned_absent_hash_is_404(tmp_path):
    app = _app(tmp_path)
    try:
        status, _, body = request(app, "GET", f"/v1/artifacts/{'0' * 64}", headers=_acme())
        assert status == 404 and body["error"]["code"] == "not_found"
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# events: neither the JSON journal nor the SSE stream may cross a tenant       #
# --------------------------------------------------------------------------- #


def test_event_journal_is_isolated_across_tenants(tmp_path):
    app = _app(tmp_path)
    try:
        # acme's run create appends a domain event; globex must never see it.
        _, _, run = request(app, "POST", "/v1/runs", body={"name": "acme-evented"}, headers=_acme("r-e"))

        status, _, acme_events = request(app, "GET", "/v1/events", headers=_acme())
        assert status == 200
        acme_ids = {event["payload"].get("id") for event in acme_events["items"]}
        assert run["id"] in acme_ids

        status, _, globex_events = request(app, "GET", "/v1/events", headers=_globex())
        assert status == 200 and globex_events["items"] == []
    finally:
        app.close()
