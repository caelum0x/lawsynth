"""Trajectory accuracy metrics over aligned recorded samples."""
from collections.abc import Iterable
from .metrics import mae, rmse

def score(reference: Iterable[float], predicted: Iterable[float]) -> dict[str, float]:
    return {"mae": mae(reference, predicted), "rmse": rmse(reference, predicted)}
