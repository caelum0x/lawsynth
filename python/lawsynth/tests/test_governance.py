"""Model governance (P9): model card, lineage, and audit log.

Pure-Python tests (audit chain, content addressing) always run. The model-card
and lineage-reproducibility tests require the native discovery extension and skip
cleanly when it is unavailable.
"""

from __future__ import annotations

import dataclasses

import pytest

from lawsynth._content import content_digest, dataset_digest
from lawsynth.audit import AuditLog
from lawsynth.dataset import Dataset
from lawsynth.lineage import Lineage


def _native_available() -> bool:
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name == "lawsynth._native":
            return False
        raise
    return True


def _oscillator_study(steps: int = 500, dt: float = 0.02):
    from lawsynth.study import Study

    k, c = 1.0, 0.3
    x, v = 1.0, 0.0
    time: list[float] = []
    xs: list[float] = []
    vs: list[float] = []
    for i in range(steps):
        time.append(i * dt)
        xs.append(x)
        vs.append(v)

        def deriv(a: float, b: float) -> tuple[float, float]:
            return b, -k * a - c * b

        k1x, k1v = deriv(x, v)
        k2x, k2v = deriv(x + 0.5 * dt * k1x, v + 0.5 * dt * k1v)
        k3x, k3v = deriv(x + 0.5 * dt * k2x, v + 0.5 * dt * k2v)
        k4x, k4v = deriv(x + dt * k3x, v + dt * k3v)
        x += dt / 6 * (k1x + 2 * k2x + 2 * k3x + k4x)
        v += dt / 6 * (k1v + 2 * k2v + 2 * k3v + k4v)
    return Study.from_columns(time, {"x": xs, "v": vs}, state=["x", "v"], name="osc")


# --------------------------------------------------------------------------- #
# Content addressing (pure Python)                                            #
# --------------------------------------------------------------------------- #


def test_dataset_digest_is_deterministic_and_column_order_independent():
    a = Dataset.from_columns([0.0, 1.0, 2.0], {"x": [1.0, 2.0, 3.0], "y": [3.0, 2.0, 1.0]})
    b = Dataset.from_columns([0.0, 1.0, 2.0], {"y": [3.0, 2.0, 1.0], "x": [1.0, 2.0, 3.0]})
    assert dataset_digest(a) == dataset_digest(b)


def test_dataset_digest_changes_with_values():
    a = Dataset.from_columns([0.0, 1.0], {"x": [1.0, 2.0]})
    b = Dataset.from_columns([0.0, 1.0], {"x": [1.0, 2.5]})
    assert dataset_digest(a) != dataset_digest(b)


def test_content_digest_is_stable():
    assert content_digest({"a": 1, "b": [2, 3]}) == content_digest({"b": [2, 3], "a": 1})


# --------------------------------------------------------------------------- #
# Audit log (pure Python)                                                     #
# --------------------------------------------------------------------------- #


def test_audit_log_appends_and_verifies():
    log = AuditLog()
    log.append("alice", "submit")
    log.append("bob", "approve", report="abc")
    assert len(log) == 2
    assert [e.ordinal for e in log.entries] == [0, 1]
    assert log.verify() is True


def test_audit_log_detects_alteration():
    log = AuditLog()
    log.append("alice", "submit")
    log.append("bob", "approve")
    forged = dataclasses.replace(log.entries[0], action="reject")
    log._entries = (forged, log.entries[1])
    assert log.verify() is False


def test_audit_log_detects_gap():
    log = AuditLog()
    log.append("alice", "a")
    log.append("bob", "b")
    log.append("carol", "c")
    # Drop the middle entry: ordinals 0,2 => gap.
    log._entries = (log.entries[0], log.entries[2])
    assert log.verify() is False


def test_audit_log_persists_and_reloads(tmp_path):
    path = tmp_path / "audit.jsonl"
    log = AuditLog(path)
    log.append("alice", "submit")
    log.append("bob", "approve")
    assert AuditLog.verify_file(path) is True
    # A reloaded log carries the same intact chain.
    assert AuditLog(path).verify() is True
    assert len(AuditLog(path)) == 2


def test_audit_file_tampering_is_detected(tmp_path):
    import json

    path = tmp_path / "audit.jsonl"
    log = AuditLog(path)
    log.append("alice", "submit")
    log.append("bob", "approve")
    lines = path.read_text().splitlines()
    obj = json.loads(lines[0])
    obj["action"] = "hacked"
    path.write_text("\n".join([json.dumps(obj), *lines[1:]]) + "\n")
    assert AuditLog.verify_file(path) is False


def test_audit_entry_requires_actor_and_action():
    log = AuditLog()
    with pytest.raises(ValueError):
        log.append("", "submit")
    with pytest.raises(ValueError):
        log.append("alice", "")


# --------------------------------------------------------------------------- #
# Lineage chain integrity (pure Python)                                       #
# --------------------------------------------------------------------------- #


def test_lineage_chain_is_content_addressed_and_verifiable():
    dataset = Dataset.from_columns([0.0, 1.0, 2.0], {"x": [1.0, 2.0, 3.0]})
    chain = Lineage.from_dataset(dataset, ["x"])
    chain = chain.record_evaluation("validate", {"mean_r_squared": 0.9})
    chain = chain.record_report("deadbeef")
    assert chain.verify_chain() is True
    # Each link chains to its predecessor's digest.
    links = chain.links
    for prev, nxt in zip(links, links[1:]):
        assert nxt.parent == prev.digest


def test_lineage_reproducible_returns_false_without_a_world():
    dataset = Dataset.from_columns([0.0, 1.0, 2.0], {"x": [1.0, 2.0, 3.0]})
    chain = Lineage.from_dataset(dataset, ["x"])
    assert chain.verify_reproducible() is False


# --------------------------------------------------------------------------- #
# Model card + lineage reproducibility (native)                              #
# --------------------------------------------------------------------------- #


def test_study_captures_lineage_and_is_reproducible():
    if not _native_available():
        pytest.skip("native extension unavailable")
    study = _oscillator_study()
    study.discover(threshold=0.05)
    lineage = study.lineage
    kinds = [link.kind for link in lineage.links]
    assert kinds == ["dataset", "discovery", "world"]
    assert lineage.world_revision is not None
    assert lineage.verify_reproducible() is True


def test_model_card_populates_out_of_sample_sections():
    if not _native_available():
        pytest.skip("native extension unavailable")
    study = _oscillator_study()
    study.discover(threshold=0.05)
    card = study.model_card(holdout=0.25, origins=4, ensemble_n=6, ensemble_seed=0)
    assert card.validation is not None
    assert card.backtest is not None
    assert card.ensemble is not None
    html = card.to_html()
    assert html.startswith("<!doctype html>")
    assert "holdout validation" in html
    assert "rolling-origin backtest" in html
    assert "robust" in html or "unstable" in html
    assert "Known limitations / not validated" in html
    # The card's lineage carries evaluation + report links.
    assert card.lineage is not None
    assert card.lineage.link_of("report") is not None


def test_model_card_honestly_omits_disabled_sections():
    if not _native_available():
        pytest.skip("native extension unavailable")
    study = _oscillator_study()
    study.discover(threshold=0.05)
    card = study.model_card(run_validate=False, run_backtest=False, run_ensemble=False)
    assert card.validation is None
    assert card.backtest is None
    assert card.ensemble is None
    html = card.to_html()
    assert html.count("Not measured") >= 3
    assert "Not measured" in card.to_text()


def test_model_card_html_is_deterministic():
    if not _native_available():
        pytest.skip("native extension unavailable")
    a = _oscillator_study()
    a.discover(threshold=0.05)
    b = _oscillator_study()
    b.discover(threshold=0.05)
    card_a = a.model_card(origins=4, ensemble_n=6, ensemble_seed=0)
    card_b = b.model_card(origins=4, ensemble_n=6, ensemble_seed=0)
    assert card_a.to_html() == card_b.to_html()
    assert card_a.digest == card_b.digest


def test_holdout_validation_scores_out_of_sample():
    if not _native_available():
        pytest.skip("native extension unavailable")
    study = _oscillator_study()
    study.discover(threshold=0.05)
    validation = study.validate(holdout=0.25)
    assert validation.train_samples + validation.test_samples == len(study.dataset.time)
    assert set(validation.states) == {"x", "v"}
    assert validation.mean_r_squared > 0.9  # a clean synthetic system generalizes
    assert validation.verdict == "strong generalization"
