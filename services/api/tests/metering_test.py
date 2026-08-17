"""Append-only, per-tenant metering log + the ``GET /v1/usage`` report (P10).

Unit coverage of the log's ordering/isolation/append-only contract runs
unconditionally.  The end-to-end wiring on the discovery-submit path (a submit
records ``run_submitted`` and ``bytes_stored``, a replayed key does not
double-bill) is guarded on the native runtime, matching the discovery tests.
"""

from __future__ import annotations

import pytest
from _harness import TOKEN, TOKEN_GLOBEX, auth, make_app, request

from lawsynth_api import discovery
from lawsynth_api.metering import BYTES_STORED, RUN_SUBMITTED, MeteringLog, MeteringRecord

_GLOBEX = {TOKEN_GLOBEX: ("globex", frozenset({"read", "write"}))}


def _native_present() -> bool:
    return discovery.native_available()


def _app(tmp_path):
    return make_app(tmp_path, extra_tokens=_GLOBEX, max_bytes=200_000)


def _discovery_body(name="metered-run"):
    return {
        "name": name,
        "dataset": {"time": [0.0, 1.0, 2.0, 3.0], "columns": {"x": [1.0, 0.5, 0.25, 0.125]}},
        "states": ["x"],
    }


# --------------------------------------------------------------------------- #
# MeteringRecord value contract                                               #
# --------------------------------------------------------------------------- #


def test_record_rejects_unknown_action_and_bad_ordinal():
    with pytest.raises(Exception):
        MeteringRecord("acme", 1, "not_an_action", 1, "s")
    with pytest.raises(Exception):
        MeteringRecord("acme", 0, RUN_SUBMITTED, 1, "s")
    with pytest.raises(Exception):
        MeteringRecord("acme", 1, RUN_SUBMITTED, -1, "s")


# --------------------------------------------------------------------------- #
# MeteringLog: ordinals, append-only ordering, isolation, aggregation         #
# --------------------------------------------------------------------------- #


def test_log_assigns_monotonic_per_tenant_ordinals_from_one():
    log = MeteringLog()
    first = log.record("acme", RUN_SUBMITTED, 1, "run-1")
    second = log.record("acme", BYTES_STORED, 512, "ds-1")
    assert (first.ordinal, second.ordinal) == (1, 2)
    # Ordinals are per tenant: a fresh tenant also starts at 1 (content-ordinals,
    # no wall clock, so ordering is reproducible from the inputs alone).
    other = log.record("globex", RUN_SUBMITTED, 1, "run-9")
    assert other.ordinal == 1


def test_log_is_tenant_partitioned_on_read():
    log = MeteringLog()
    log.record("acme", RUN_SUBMITTED, 1, "run-1")
    log.record("globex", RUN_SUBMITTED, 1, "run-2")
    assert [r.subject for r in log.records("acme")] == ["run-1"]
    assert [r.subject for r in log.records("globex")] == ["run-2"]
    # An unknown tenant leaks nothing.
    assert log.records("stranger") == []


def test_usage_aggregates_totals_per_action():
    log = MeteringLog()
    log.record("acme", RUN_SUBMITTED, 1, "run-1")
    log.record("acme", RUN_SUBMITTED, 1, "run-2")
    log.record("acme", BYTES_STORED, 300, "ds-1")
    usage = log.usage("acme")
    assert usage["organization_id"] == "acme"
    assert usage["totals"][RUN_SUBMITTED] == 2
    assert usage["totals"][BYTES_STORED] == 300
    assert [r["ordinal"] for r in usage["records"]] == [1, 2, 3]


# --------------------------------------------------------------------------- #
# GET /v1/usage: auth, empty envelope, isolation (native-agnostic)            #
# --------------------------------------------------------------------------- #


def test_usage_endpoint_requires_authentication(tmp_path):
    app = _app(tmp_path)
    try:
        status, _, _ = request(app, "GET", "/v1/usage")
        assert status == 401
    finally:
        app.close()


def test_usage_endpoint_returns_empty_envelope_for_fresh_tenant(tmp_path):
    app = _app(tmp_path)
    try:
        status, _, usage = request(app, "GET", "/v1/usage", headers=auth())
        assert status == 200
        assert usage["organization_id"] == "acme"
        assert usage["totals"] == {BYTES_STORED: 0, RUN_SUBMITTED: 0}
        assert usage["records"] == []
    finally:
        app.close()


# --------------------------------------------------------------------------- #
# End-to-end metering on the submit path (native-present)                     #
# --------------------------------------------------------------------------- #


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_submit_meters_run_and_bytes_and_is_queryable(tmp_path):
    app = _app(tmp_path)
    try:
        status, _, run = request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(key="m-1"))
        assert status == 201

        status, _, usage = request(app, "GET", "/v1/usage", headers=auth())
        assert status == 200
        assert usage["totals"][RUN_SUBMITTED] == 1
        assert usage["totals"][BYTES_STORED] > 0
        actions = [record["action"] for record in usage["records"]]
        assert actions == [RUN_SUBMITTED, BYTES_STORED]
        assert usage["records"][0]["subject"] == run["id"]
        # Ordinals are dense and start at 1.
        assert [record["ordinal"] for record in usage["records"]] == [1, 2]
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_idempotent_replay_does_not_double_bill(tmp_path):
    app = _app(tmp_path)
    try:
        body = _discovery_body(name="idem-meter")
        request(app, "POST", "/v1/runs", body=body, headers=auth(key="same-meter"))
        # Same idempotency key replays the stored response; it must NOT re-bill.
        status, headers, _ = request(app, "POST", "/v1/runs", body=body, headers=auth(key="same-meter"))
        assert status == 201 and headers.get("Idempotency-Replayed") == "true"

        _, _, usage = request(app, "GET", "/v1/usage", headers=auth())
        assert usage["totals"][RUN_SUBMITTED] == 1
    finally:
        app.close()


@pytest.mark.skipif(not _native_present(), reason="native runtime is absent")
def test_usage_is_isolated_across_tenants(tmp_path):
    app = _app(tmp_path)
    try:
        request(app, "POST", "/v1/runs", body=_discovery_body(), headers=auth(token=TOKEN, key="acme-m"))

        # globex submitted nothing: its usage is empty, never acme's.
        _, _, globex_usage = request(app, "GET", "/v1/usage", headers=auth(token=TOKEN_GLOBEX))
        assert globex_usage["totals"][RUN_SUBMITTED] == 0
        assert globex_usage["records"] == []
    finally:
        app.close()
