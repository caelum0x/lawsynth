"""Runnable example: evaluate a dense neural prior over feature rows.

    python plugins/neural-prior/examples/basic.py
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src"))

from neural_prior.plugin import NeuralPrior  # noqa: E402


def main() -> None:
    # A 2 -> 2 -> 1 network: a ReLU hidden layer then a linear read-out.
    model = {
        "layers": [
            {"weights": [[0.5, -0.5], [1.0, 0.0]], "bias": [0.0, 0.1], "activation": "relu"},
            {"weights": [[1.0, 1.0]], "bias": [-0.25], "activation": "identity"},
        ]
    }

    prior = NeuralPrior(model)

    features = [[1.0, 2.0], [0.0, -1.0], [3.0, 3.0]]
    result = prior.invoke({"features": features})

    print("row_count:", result["row_count"])
    for row, prediction in zip(features, result["predictions"]):
        print(f"{row} -> {prediction}")


if __name__ == "__main__":
    main()
