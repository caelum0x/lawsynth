"""Tests for the simulate action module (a sub-resource of worlds)."""

from __future__ import annotations

import pytest

from lawsynth_api import simulations
from lawsynth_server.errors import ValidationError


def test_is_simulate_matches_only_the_world_action():
    assert simulations.is_simulate(["worlds", "id", "simulate"])
    assert not simulations.is_simulate(["worlds", "id"])
    assert not simulations.is_simulate(["runs", "id", "simulate"])


def test_classify_labels_the_simulate_post():
    assert simulations.classify("POST", ["worlds", "id", "simulate"]) == "worlds.simulate"
    assert simulations.classify("GET", ["worlds", "id", "simulate"]) is None
    assert simulations.classify("POST", ["worlds", "id"]) is None


def test_normalize_request_accepts_valid_spec():
    spec = simulations.normalize_request({"initial": {"x": 1.0}, "horizon": 1.0, "step": 0.1})
    assert spec["horizon"] == 1.0 and spec["step"] == 0.1 and spec["method"] == "rk4"


def test_normalize_request_rejects_non_positive_step():
    with pytest.raises(ValidationError):
        simulations.normalize_request({"horizon": 1.0, "step": 0.0})


def test_normalize_request_rejects_unsupported_method():
    with pytest.raises(ValidationError):
        simulations.normalize_request({"horizon": 1.0, "step": 0.1, "method": "euler"})
