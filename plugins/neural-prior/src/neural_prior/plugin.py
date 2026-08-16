"""Dependency-free dense neural prior inference with strict shape bounds."""

from __future__ import annotations

import math
from collections.abc import Mapping, Sequence
from typing import Any


def _activation(name: str, value: float) -> float:
    if name == "identity": return value
    if name == "relu": return max(0.0, value)
    if name == "tanh": return math.tanh(value)
    if name == "sigmoid": return 1.0 / (1.0 + math.exp(-max(-700.0, min(700.0, value))))
    raise ValueError(f"unsupported activation: {name}")


class NeuralPrior:
    def __init__(self, model: Mapping[str, Any], *, max_parameters: int = 10_000_000) -> None:
        layers = model.get("layers")
        if not isinstance(layers, Sequence) or not layers:
            raise ValueError("model requires at least one layer")
        self.layers: list[tuple[list[list[float]], list[float], str]] = []
        parameters = 0
        previous_width: int | None = None
        for layer in layers:
            weights = [[float(value) for value in row] for row in layer["weights"]]
            bias = [float(value) for value in layer["bias"]]
            if not weights or len(weights) != len(bias) or any(len(row) != len(weights[0]) for row in weights):
                raise ValueError("neural layer shape is invalid")
            if previous_width is not None and len(weights[0]) != previous_width:
                raise ValueError("neural layer widths do not connect")
            if any(not math.isfinite(value) for row in weights for value in row + bias):
                raise ValueError("neural parameters must be finite")
            parameters += sum(map(len, weights)) + len(bias)
            if parameters > max_parameters:
                raise ValueError("model exceeds parameter limit")
            self.layers.append((weights, bias, str(layer.get("activation", "identity"))))
            previous_width = len(weights)

    def predict(self, features: Sequence[float]) -> list[float]:
        values = [float(value) for value in features]
        if len(values) != len(self.layers[0][0][0]):
            raise ValueError("feature width does not match model input")
        for weights, bias, activation in self.layers:
            values = [_activation(activation, sum(w * x for w, x in zip(row, values, strict=True)) + b) for row, b in zip(weights, bias, strict=True)]
        return values

    def invoke(self, request: Mapping[str, Any]) -> dict[str, Any]:
        rows = request.get("features", ())
        if not isinstance(rows, Sequence) or len(rows) > 1_000_000:
            raise ValueError("feature batch is invalid or too large")
        predictions = [self.predict(row) for row in rows]
        return {"predictions": predictions, "row_count": len(predictions)}
