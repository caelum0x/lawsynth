"""Per-tenant quota enforcement at the discovery-submit boundary (P10).

Quota is a hard pre-admission gate on ``POST /v1/runs``: it is checked before the
native probe, so these tests are deterministic whether or not the native runtime
is installed.  Exceeding a quota returns a documented ``429 quota_exceeded`` and
never silently drops the work.
"""

from __future__ import annotations

from _harness import TOKEN, TOKEN_GLOBEX, auth, request

from lawsynth_api import ApiSettings, create_wsgi_app
from lawsynth_api.quota import QuotaPolicy, QuotaGuard, dataset_bytes
from lawsynth_server.settings import Settings as ServerSettings


def _app(tmp_path, *, max_active_runs=1000, max_dataset_bytes=1024 * 1024 * 1024):
    server = ServerSettings(
        database_url=f"sqlite:///{tmp_path / 'metadata.sqlite3'}",
        object_root=tmp_path / "objects",
        tokens={
            TOKEN: ("acme", frozenset({"read", "write"})),
            TOKEN_GLOBEX: ("globex", frozenset({"read", "write"})),
        },
        max_upload_bytes=200_000,
    )
    settings = ApiSettings(
        server=server,
        environment="test",
        max_request_bytes=200_000,
        quota=QuotaPolicy(max_active_runs=max_active_runs, max_dataset_bytes=max_dataset_bytes),
    )
    return create_wsgi_app(settings)


def _discovery_body(name="quota-run"):
    return {
        "name": name,
        "dataset": {"time": [0.0, 1.0, 2.0, 3.0], "columns": {"x": [1.0, 0.5, 0.25, 0.125]}},
        "states": ["x"],
    }


# --------------------------------------------------------------------------- #
# Unit: policy validation + deterministic dataset sizing                       #
# --------------------------------------------------------------------------- #


def test_quota_policy_rejects_non_positive_limits():
    import pytest

    with pytest.raises(Exception):
        QuotaPolicy(max_active_runs=0)
    with pytest.raises(Exception):
        QuotaPolicy(max_dataset_bytes=0)


def test_dataset_bytes_is_deterministic_and_offline():
    a = dataset_bytes([0.0, 1.0], {"x": [1.0, 2.0]})
    b = dataset_bytes([0.0, 1.0], {"x": [1.0, 2.0]})
    assert a == b and a > 0
    # A reference (no inline observations) measures as zero.
    assert dataset_bytes(None, None) == 0


# --------------------------------------------------------------------------- #
# Active-run concurrency ceiling                                               #
# --------------------------------------------------------------------------- #


def test_active_run_quota_returns_429_before_native(tmp_path):
    # Two plain (non-discovery) runs stay queued and occupy the active-run ceiling.
    app = _app(tmp_path, max_active_runs=2)
    try:
        for index in range(2):
            status, _, run = request(app, "POST", "/v1/runs", body={"name": f"plain-{index}"}, headers=auth(key=f"plain-{index}"))
            assert status == 201 and run["status"] == "queued"

        # A third submission (a discovery run) is over quota and is refused.
        status, _, body = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(key="over-quota"))
        assert status == 429 and body["error"]["code"] == "quota_exceeded"
    finally:
        app.close()


def test_active_run_quota_is_per_tenant(tmp_path):
    # acme saturates its own ceiling; globex is unaffected (isolation).
    app = _app(tmp_path, max_active_runs=1)
    try:
        status, _, _ = request(app, "POST", "/v1/runs", body={"name": "acme-active"}, headers=auth(token=TOKEN, key="a-1"))
        assert status == 201
        status, _, body = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(token=TOKEN, key="a-over"))
        assert status == 429 and body["error"]["code"] == "quota_exceeded"

        # globex has consumed none of its own quota: its submit is not quota-blocked
        # (it proceeds to the native boundary -- 201 if native present, else 503).
        status, _, _ = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(token=TOKEN_GLOBEX, key="g-1"))
        assert status != 429
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# Dataset storage ceiling                                                      #
# --------------------------------------------------------------------------- #


def test_dataset_bytes_quota_returns_429(tmp_path):
    # A tiny storage ceiling that the inline dataset in the submit exceeds.
    app = _app(tmp_path, max_dataset_bytes=10)
    try:
        status, _, body = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(key="big-ds"))
        assert status == 429 and body["error"]["code"] == "quota_exceeded"
    finally:
        app.close()


def test_submit_within_quota_is_not_quota_blocked(tmp_path):
    # Generous limits: the submit clears quota and reaches the native boundary.
    # Native-agnostic: it is admitted (201) when native is present, or a 503
    # native_unavailable otherwise -- but never a 429.
    app = _app(tmp_path)
    try:
        status, _, _ = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(key="ok-1"))
        assert status in {201, 503}
    finally:
        app.close()
