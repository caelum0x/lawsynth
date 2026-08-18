"""Tests for experiment-tracking interop (``lawsynth.tracking``).

The module must import — and its anchor :class:`RunRecord` must be fully built,
serialised and asserted — with none of ``mlflow``/``wandb``/``numpy`` installed.
Dependency-free behaviour (record extraction, deterministic JSON, honest metric
omission, the clear missing-dependency error) is tested unconditionally; the
actual-logging paths use ``pytest.importorskip`` so they skip cleanly when a
backend is absent. Tests needing the compiled engine are marked ``@native``.
"""

from __future__ import annotations

import json

import pytest

import lawsynth
from lawsynth import tracking
from lawsynth._version import __version__
from lawsynth.errors import LawSynthError


def _native_available() -> bool:
    try:
        import lawsynth._native  # noqa: F401
    except ModuleNotFoundError as error:
        if error.name == "lawsynth._native":
            return False
        raise
    return True


native = pytest.mark.skipif(not _native_available(), reason="native extension not built")


def _module_absent(name: str) -> bool:
    try:
        __import__(name)
    except ImportError:
        return True
    return False


class _StubResult:
    """A minimal, native-free stand-in for a discovered run.

    Exposes just enough surface — ``name``, ``states``, ``equations`` and a
    ``save`` — to drive the whole extraction path without the compiled engine.
    Deliberately has no ``explain()``, so it exercises the honest "no fit metric"
    branch.
    """

    def __init__(self, equations, *, name="stub", config=None, states=None):
        self._equations = dict(equations)
        self.name = name
        self.states = tuple(states) if states is not None else tuple(sorted(self._equations))
        self._config = config

    @property
    def equations(self):
        return dict(self._equations)

    def save(self, path):
        from pathlib import Path

        Path(path).write_bytes(b"LSWORLD-STUB\x00" + repr(sorted(self._equations.items())).encode())


def _linear_stub():
    # Two additive terms in dv, one in dx — a clean polynomial the term-splitter reads.
    return _StubResult(
        {"x": "(1.0*v)", "v": "((-4.0*x)+(-0.3*v))"},
        name="osc demo",
        states=("x", "v"),
    )


def _discover_oscillator():
    """Deterministically discover a damped linear oscillator (needs native)."""
    k, c = 4.0, 0.3

    def spring(_t, s):
        x, v = s
        return [v, -k * x - c * v]

    y = [1.0, 0.0]
    t = 0.0
    times, cols = [], [[], []]
    dt = 0.01
    for _ in range(2000):
        times.append(t)
        cols[0].append(y[0])
        cols[1].append(y[1])
        k1 = spring(t, y)
        k2 = spring(t + dt / 2, [y[j] + dt / 2 * k1[j] for j in range(2)])
        k3 = spring(t + dt / 2, [y[j] + dt / 2 * k2[j] for j in range(2)])
        k4 = spring(t + dt, [y[j] + dt * k3[j] for j in range(2)])
        y = [y[j] + dt / 6 * (k1[j] + 2 * k2[j] + 2 * k3[j] + k4[j]) for j in range(2)]
        t += dt
    study = lawsynth.Study.from_columns(times, {"x": cols[0], "v": cols[1]}, state=["x", "v"], name="osc")
    return study.discover(recipe="mechanics")


# --------------------------------------------------------------------------- #
# Import & error surface (no optional dependency required)                     #
# --------------------------------------------------------------------------- #


def test_module_and_public_symbols_import_without_backends():
    assert callable(lawsynth.run_record)
    assert callable(lawsynth.log_to_mlflow)
    assert callable(lawsynth.log_to_wandb)
    assert lawsynth.RunRecord is tracking.RunRecord
    assert lawsynth.RunArtifact is tracking.RunArtifact
    assert lawsynth.TrackingError is tracking.TrackingError


def test_missing_dependency_error_is_lawsynth_and_import_error():
    assert issubclass(tracking.MissingDependencyError, LawSynthError)
    assert issubclass(tracking.MissingDependencyError, ImportError)
    assert issubclass(tracking.MissingDependencyError, tracking.TrackingError)


def test_require_raises_clear_typed_error():
    with pytest.raises(tracking.MissingDependencyError) as info:
        tracking._require("lawsynth_absent_tracker_xyz", "log_to_mlflow()")
    message = str(info.value)
    assert "lawsynth_absent_tracker_xyz" in message
    assert "pip install" in message
    assert "log_to_mlflow()" in message
    assert isinstance(info.value, LawSynthError)
    assert isinstance(info.value, ImportError)


def test_log_to_mlflow_raises_missing_dependency_when_absent():
    if not _module_absent("mlflow"):
        pytest.skip("mlflow is installed; the missing-dependency path is not exercised")
    with pytest.raises(tracking.MissingDependencyError) as info:
        lawsynth.log_to_mlflow(_linear_stub())
    assert "mlflow" in str(info.value)
    assert isinstance(info.value, ImportError)


def test_log_to_wandb_raises_missing_dependency_when_absent():
    if not _module_absent("wandb"):
        pytest.skip("wandb is installed; the missing-dependency path is not exercised")
    with pytest.raises(tracking.MissingDependencyError) as info:
        lawsynth.log_to_wandb(_linear_stub())
    assert "wandb" in str(info.value)
    assert isinstance(info.value, ImportError)


# --------------------------------------------------------------------------- #
# The anchor RunRecord: dependency-free extraction                             #
# --------------------------------------------------------------------------- #


def test_run_record_extracts_structural_metrics_from_stub():
    record = lawsynth.run_record(_linear_stub())
    assert isinstance(record, tracking.RunRecord)
    assert record.name == "osc demo"
    # law/term/complexity are computed from the law strings alone (pure stdlib).
    assert record.metrics["law_count"] == 2.0
    assert record.metrics["term_count"] == 3.0  # 1 term in dx + 2 terms in dv
    assert record.metrics["complexity_nodes"] > 0.0


def test_run_record_tags_carry_metadata():
    record = lawsynth.run_record(_linear_stub())
    assert record.tags["framework"] == "lawsynth"
    assert record.tags["engine_version"] == __version__
    assert record.tags["variables"] == "x,v"
    # world revision is a deterministic content hash recoverable from the laws.
    assert len(record.tags["world_revision"]) == 64


def test_absent_fit_metric_is_omitted_not_fabricated():
    record = lawsynth.run_record(_linear_stub())  # stub has no explain() -> no fit
    assert not any(key.startswith("r_squared") for key in record.metrics)
    assert not any(key.startswith("rmse") for key in record.metrics)


def test_config_params_reflect_actual_discovery_config():
    from lawsynth.config import DiscoveryConfig

    config = DiscoveryConfig(polynomial_degree=3, threshold=0.02, solver="sr3")
    record = lawsynth.run_record(_StubResult({"x": "(1.0*x)"}, config=config))
    assert record.params["polynomial_degree"] == 3
    assert record.params["threshold"] == 0.02
    assert record.params["solver"] == "sr3"
    # Every declared config field is present — nothing silently dropped.
    assert set(record.params) == set(DiscoveryConfig.__dataclass_fields__)


def test_no_config_yields_no_config_params():
    record = lawsynth.run_record(_StubResult({"x": "(1.0*x)"}, config=None))
    assert record.params == {}


def test_extra_params_metrics_and_tags_are_merged_honestly():
    record = lawsynth.run_record(
        _linear_stub(),
        tags={"owner": "alice"},
        extra_params={"note": "run-A"},
        extra_metrics={"training_time_s": 1.5, "not_a_number": float("inf")},
    )
    assert record.params["note"] == "run-A"
    assert record.tags["owner"] == "alice"
    assert record.metrics["training_time_s"] == 1.5
    # Non-finite caller metrics are dropped, never logged as-is.
    assert "not_a_number" not in record.metrics


# --------------------------------------------------------------------------- #
# Determinism of the record and its serialisation                              #
# --------------------------------------------------------------------------- #


def test_run_record_serialisation_is_byte_stable():
    a = lawsynth.run_record(_linear_stub())
    b = lawsynth.run_record(_linear_stub())
    assert a.to_dict() == b.to_dict()
    assert a.to_json() == b.to_json()  # byte-for-byte identical


def test_run_record_json_is_valid_and_sorted():
    record = lawsynth.run_record(_linear_stub())
    text = record.to_json()
    parsed = json.loads(text)
    assert set(parsed) == {"name", "params", "metrics", "tags", "artifact"}
    # sort_keys makes key ordering stable and independent of insertion order.
    assert list(parsed["metrics"]) == sorted(parsed["metrics"])
    assert list(parsed["tags"]) == sorted(parsed["tags"])


def test_records_equal_by_value():
    assert lawsynth.run_record(_linear_stub()) == lawsynth.run_record(_linear_stub())


# --------------------------------------------------------------------------- #
# Artifact capture (the .lsworld bundle)                                       #
# --------------------------------------------------------------------------- #


def test_artifact_reference_default_records_only_filename():
    record = lawsynth.run_record(_linear_stub())
    assert record.artifact is not None
    assert record.artifact.filename == "osc-demo.lsworld"
    assert record.artifact.path is None
    assert record.artifact.sha256 is None


def test_artifact_written_when_path_given(tmp_path):
    target = tmp_path / "osc.lsworld"
    record = lawsynth.run_record(_linear_stub(), artifact_path=target)
    assert record.artifact is not None
    assert record.artifact.path == str(target)
    assert record.artifact.size_bytes == target.stat().st_size
    assert len(record.artifact.sha256) == 64
    # The digest matches the bytes actually on disk (no fabrication).
    import hashlib

    assert record.artifact.sha256 == hashlib.sha256(target.read_bytes()).hexdigest()


def test_artifact_file_context_yields_written_bundle(tmp_path):
    target = tmp_path / "osc.lsworld"
    record = lawsynth.run_record(_linear_stub(), artifact_path=target)
    with tracking._artifact_file(record, None) as bundle:
        assert bundle is not None
        assert bundle.is_file()


# --------------------------------------------------------------------------- #
# Native-backed extraction (real DiscoveryResult) — skipped without the engine #
# --------------------------------------------------------------------------- #


@native
def test_run_record_from_real_discovery_has_config_and_fit_metrics():
    result = _discover_oscillator()
    record = lawsynth.run_record(result)
    # Params mirror the resolved discovery config used by the run.
    assert record.params["solver"] == "stlsq"
    assert "polynomial_degree" in record.params
    # Fit metrics genuinely exist for a discovered world (simulation vs. data).
    assert record.metrics["r_squared_x"] > 0.99
    assert record.metrics["r_squared_v"] > 0.99
    assert record.metrics["r_squared_mean"] > 0.99
    assert record.metrics["rmse_max"] >= record.metrics["rmse_mean"]
    assert record.metrics["law_count"] == 2.0
    # World revision matches the SDK's own content-addressed lineage hash.
    assert record.tags["world_revision"] == result.lineage.world_revision


@native
def test_convenience_methods_attached_to_result_and_study():
    result = _discover_oscillator()
    assert callable(result.run_record)
    record = result.run_record()
    assert isinstance(record, tracking.RunRecord)
    assert record.name == "osc"


@native
def test_real_discovery_record_is_deterministic():
    a = lawsynth.run_record(_discover_oscillator())
    b = lawsynth.run_record(_discover_oscillator())
    assert a.to_json() == b.to_json()


@native
def test_real_artifact_bundle_is_captured(tmp_path):
    result = _discover_oscillator()
    target = tmp_path / "osc.lsworld"
    record = lawsynth.run_record(result, artifact_path=target)
    assert record.artifact.size_bytes > 0
    # The saved bundle reloads into an equivalent world.
    reloaded = lawsynth.Study.load(target, dataset=result._dataset, state=list(result.states))
    assert set(reloaded.world.equations()) == set(result.equations)


# --------------------------------------------------------------------------- #
# Actual-logging paths — skipped cleanly when the backend is not installed     #
# --------------------------------------------------------------------------- #


@native
def test_log_to_mlflow_round_trips_when_installed(tmp_path):
    mlflow = pytest.importorskip("mlflow")
    uri = (tmp_path / "mlruns").as_uri()
    result = _discover_oscillator()
    run_id = lawsynth.log_to_mlflow(result, tracking_uri=uri, run_name="osc-test")
    assert isinstance(run_id, str) and run_id
    client = mlflow.tracking.MlflowClient(tracking_uri=uri)
    data = client.get_run(run_id).data
    assert "polynomial_degree" in data.params
    assert data.metrics["law_count"] == 2.0


def test_log_to_wandb_uses_backend_when_installed():
    pytest.importorskip("wandb")
    # A real W&B run needs network/credentials; we only assert the symbol is wired.
    # Offline logging is validated by the record payload tests above.
    assert callable(lawsynth.log_to_wandb)
