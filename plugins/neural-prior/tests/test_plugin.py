"""Tests for the dependency-free dense neural prior."""

from __future__ import annotations

import math
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from neural_prior.plugin import NeuralPrior, _activation


def _linear_identity_model() -> dict:
    # 2 -> 2 identity map: weights are the identity matrix, zero bias.
    return {"layers": [{"weights": [[1.0, 0.0], [0.0, 1.0]], "bias": [0.0, 0.0], "activation": "identity"}]}


def test_identity_layer_passes_features_through() -> None:
    prior = NeuralPrior(_linear_identity_model())
    assert prior.predict([2.5, -1.0]) == [2.5, -1.0]


def test_relu_hidden_layer_then_linear_readout() -> None:
    model = {
        "layers": [
            {"weights": [[1.0, 0.0], [0.0, 1.0]], "bias": [0.0, 0.0], "activation": "relu"},
            {"weights": [[1.0, 1.0]], "bias": [0.0], "activation": "identity"},
        ]
    }
    prior = NeuralPrior(model)
    # relu([-1, 2]) = [0, 2]; readout sum = 2.
    assert prior.predict([-1.0, 2.0]) == [2.0]


def test_invoke_batches_predictions() -> None:
    prior = NeuralPrior(_linear_identity_model())
    result = prior.invoke({"features": [[1.0, 2.0], [3.0, 4.0]]})
    assert result["row_count"] == 2
    assert result["predictions"] == [[1.0, 2.0], [3.0, 4.0]]


def test_activation_functions() -> None:
    assert _activation("relu", -3.0) == 0.0
    assert _activation("relu", 3.0) == 3.0
    assert math.isclose(_activation("tanh", 0.0), 0.0)
    assert math.isclose(_activation("sigmoid", 0.0), 0.5)


def test_sigmoid_input_is_clamped_against_overflow() -> None:
    # Very negative input must not raise OverflowError; result approaches 0.
    assert _activation("sigmoid", -10_000.0) == pytest.approx(0.0, abs=1e-9)


def test_feature_width_mismatch_is_rejected() -> None:
    prior = NeuralPrior(_linear_identity_model())
    with pytest.raises(ValueError, match="feature width"):
        prior.predict([1.0, 2.0, 3.0])


def test_disconnected_layer_widths_are_rejected() -> None:
    model = {
        "layers": [
            {"weights": [[1.0, 0.0], [0.0, 1.0]], "bias": [0.0, 0.0], "activation": "identity"},
            # expects 3 inputs but previous layer emits 2.
            {"weights": [[1.0, 1.0, 1.0]], "bias": [0.0], "activation": "identity"},
        ]
    }
    with pytest.raises(ValueError, match="do not connect"):
        NeuralPrior(model)


def test_non_finite_parameter_is_rejected() -> None:
    model = {"layers": [{"weights": [[float("inf")]], "bias": [0.0], "activation": "identity"}]}
    with pytest.raises(ValueError, match="finite"):
        NeuralPrior(model)


def test_unsupported_activation_is_rejected() -> None:
    model = {"layers": [{"weights": [[1.0]], "bias": [0.0], "activation": "gelu"}]}
    prior = NeuralPrior(model)
    with pytest.raises(ValueError, match="activation"):
        prior.predict([1.0])


def test_parameter_limit_is_enforced() -> None:
    model = {"layers": [{"weights": [[1.0, 1.0]], "bias": [0.0], "activation": "identity"}]}
    with pytest.raises(ValueError, match="parameter limit"):
        NeuralPrior(model, max_parameters=1)
